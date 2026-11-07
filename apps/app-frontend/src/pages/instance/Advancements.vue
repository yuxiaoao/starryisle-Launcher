<script setup lang="ts">
import steveSkin from '@/assets/skins/steve.png?inline'
import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer'
import type { Skin } from '@/helpers/skins'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import {
    type AdvancementDef,
    type WorldAdvancements,
    getAdvancementCatalog,
    getWorldAdvancements,
    listWorlds,
} from '@/helpers/advancements'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
    heading: { id: 'app.instance.advancements.heading', defaultMessage: 'Advancements' },
    empty: {
        id: 'app.instance.advancements.empty',
        defaultMessage:
            'No single-player worlds yet. Start a world and its progress will show up here.',
    },
    loading: { id: 'app.instance.advancements.loading', defaultMessage: 'Reading worlds…' },
    lastPlayed: {
        id: 'app.instance.advancements.last-played',
        defaultMessage: 'Last played {when}',
    },
    refresh: { id: 'app.instance.advancements.refresh', defaultMessage: 'Refresh' },
    catalogMissing: {
        id: 'app.instance.advancements.catalog-missing',
        defaultMessage: 'Start the game once so the advancement list can be read from your files.',
    },
    hidden: { id: 'app.instance.advancements.hidden', defaultMessage: 'Hidden advancement' },
    hiddenHint: {
        id: 'app.instance.advancements.hidden-hint',
        defaultMessage: 'Keep playing to discover this one.',
    },
    completedOn: {
        id: 'app.instance.advancements.completed-on',
        defaultMessage: 'Unlocked {date}',
    },
    player: { id: 'app.instance.advancements.player', defaultMessage: 'Player' },
    noPlayer: {
        id: 'app.instance.advancements.no-player',
        defaultMessage: 'No player progress in this world',
    },
    unknownPlayer: {
        id: 'app.instance.advancements.unknown-player',
        defaultMessage: 'Unknown player ({id})',
    },
    categoryStory: { id: 'app.instance.advancements.category.story', defaultMessage: 'Story' },
    categoryNether: { id: 'app.instance.advancements.category.nether', defaultMessage: 'Nether' },
    categoryEnd: { id: 'app.instance.advancements.category.end', defaultMessage: 'The End' },
    categoryAdventure: {
        id: 'app.instance.advancements.category.adventure',
        defaultMessage: 'Adventure',
    },
    categoryHusbandry: {
        id: 'app.instance.advancements.category.husbandry',
        defaultMessage: 'Husbandry',
    },
})

const props = defineProps<{
    instance: { id: string }
}>()

type LauncherUser = {
    account_id: string
    profile: {
        id: string
        name: string
        skins?: Array<{
            state: string
            url: string
            variant?: string
            textureKey?: string
        }>
    }
}

/// Crops the 8x8 face out of a 64x64 skin into a small pixelated head image.
/// Drawing through a canvas keeps the result local, so no remote image is loaded
/// and the launcher's content policy never comes into play.
async function renderHead(texture: string): Promise<string | null> {
    try {
        const image = new Image()
        image.src = texture
        await image.decode()
        const canvas = document.createElement('canvas')
        canvas.width = 32
        canvas.height = 32
        const context = canvas.getContext('2d')
        if (!context) return null
        context.imageSmoothingEnabled = false
        context.drawImage(image, 8, 8, 8, 8, 0, 0, 32, 32)
        const blob = await new Promise<Blob | null>((resolve) =>
            canvas.toBlob(resolve, 'image/png'),
        )
        return blob ? URL.createObjectURL(blob) : null
    } catch {
        return null
    }
}

/// Names come straight from the accounts signed in to the launcher; heads are
/// rendered afterwards so a failed texture never hides the names.
async function loadAccounts() {
    let users: LauncherUser[] = []
    try {
        users = await invoke<LauncherUser[]>('plugin:auth|get_users', {
            offlineMode: false,
            cachedOnly: true,
        })
    } catch {
        users = []
    }

    const names: Record<string, string> = {}
    for (const user of users) {
        const uuid = normalizeUuid(user.account_id)
        const name = user.profile?.name
        if (name) names[uuid] = name
    }
    nameByUuid.value = names

    if (!defaultHeadUrl.value) {
        const head = await renderHead(steveSkin)
        if (head) {
            createdHeadUrls.add(head)
            defaultHeadUrl.value = head
        }
    }

    const heads: Record<string, string> = {}
    for (const user of users) {
        const uuid = normalizeUuid(user.account_id)
        const skin =
            user.profile?.skins?.find((entry) => entry.state === 'ACTIVE') ??
            user.profile?.skins?.[0]
        if (!skin?.url) continue

        // Prefer the head the account card already rendered: the renderer keeps a
        // texture-keyed cache, so a previously fetched skin works offline too.
        try {
            heads[uuid] = await getPlayerHeadUrl({
                texture_key: skin.textureKey ?? `${user.profile.id}:${skin.url}`,
                variant: (skin.variant as Skin['variant']) ?? 'UNKNOWN',
                texture: skin.url,
                source: 'custom_external',
                is_equipped: true,
            } satisfies Skin)
            continue
        } catch {
            // Fall through to a direct crop of the skin texture.
        }

        const head = await renderHead(skin.url)
        if (head) heads[uuid] = head
    }
    skinByUuid.value = heads
}

const loading = ref(true)
const worlds = ref<Awaited<ReturnType<typeof listWorlds>>>([])
const catalog = ref<AdvancementDef[]>([])
const counts = ref<Record<string, number>>({})

const openWorld = ref<string | null>(null)
const detail = ref<WorldAdvancements | null>(null)
const detailLoading = ref(false)
const activeCategory = ref<string>('story')
const activePlayer = ref<string | null>(null)

const total = computed(() => catalog.value.length)

/// Worlds can carry achievements from mods and data packs; only the ids in the
/// vanilla catalogue are counted, otherwise the progress could exceed the total.
const catalogIds = computed(() => new Set(catalog.value.map((entry) => entry.id)))

function countCompleted(completed: Array<{ id: string }>): number {
    if (catalogIds.value.size === 0) return completed.length
    return completed.filter((entry) => catalogIds.value.has(entry.id)).length
}

const completedCount = computed(() => countCompleted(detail.value?.completed ?? []))

/// A world without a catalogue (jar not downloaded yet) has no denominator.
function progressLabelFor(done: number): string {
    return total.value > 0 ? `${done} / ${total.value}` : String(done)
}

const progressLabel = computed(() => progressLabelFor(completedCount.value))

const playerName = computed(() => {
    const id = activePlayer.value
    if (!id) return ''
    const accountName = nameByUuid.value[normalizeUuid(id)]
    if (accountName) return accountName
    const found = detail.value?.players.find((player) => player.id === id)
    return found?.name ?? formatMessage(messages.unknownPlayer, { id: id.slice(0, 8) })
})

/// Player heads are rendered by the same helper the account card uses, so they
/// come from the shared head cache and never rely on remote images inside SVG.
const skinByUuid = ref<Record<string, string>>({})
const nameByUuid = ref<Record<string, string>>({})
const defaultHeadUrl = ref<string | null>(null)
const createdHeadUrls = new Set<string>()

function normalizeUuid(value: string): string {
    // Account ids arrive as `microsoft:<uuid>`; worlds only store the bare uuid.
    const bare = value.includes(':') ? value.slice(value.lastIndexOf(':') + 1) : value
    return bare.replaceAll('-', '').toLowerCase()
}

const playerAvatar = computed(() => {
    const id = activePlayer.value
    if (!id) return defaultHeadUrl.value
    return skinByUuid.value[normalizeUuid(id)] ?? defaultHeadUrl.value
})

/// Only accounts signed in to the launcher can be picked here; uuids from other
/// machines stay visible in the list but cannot be selected.
const accountPlayers = computed(() =>
    (detail.value?.players ?? []).filter(
        (player) => nameByUuid.value[normalizeUuid(player.id)] !== undefined,
    ),
)

const rows = computed(() =>
    worlds.value.map((world) => ({ world, done: counts.value[world.name] ?? 0 })),
)

const completedMap = computed(() => {
    const map = new Map<string, number | null>()
    for (const entry of detail.value?.completed ?? []) map.set(entry.id, entry.completedAt)
    return map
})

// The tab row is fixed so every world opens with the same layout.
const categories = ['story', 'nether', 'end', 'adventure', 'husbandry'] as const

const categoryCounts = computed(() => {
    const result: Record<string, { done: number; total: number }> = {}
    for (const entry of catalog.value) {
        const bucket = (result[entry.category] ??= { done: 0, total: 0 })
        bucket.total += 1
        if (completedMap.value.has(entry.id)) bucket.done += 1
    }
    return result
})

const visibleEntries = computed(() =>
    catalog.value
        .filter((entry) => entry.category === activeCategory.value)
        .map((entry) => {
            const completed = completedMap.value.has(entry.id)
            return {
                entry,
                completed,
                completedAt: completedMap.value.get(entry.id) ?? null,
                masked: !completed && entry.hidden,
            }
        })
        .sort((left, right) => {
            if (left.completed !== right.completed) return left.completed ? -1 : 1
            return left.entry.title.localeCompare(right.entry.title)
        }),
)

const categoryLabels: Record<string, keyof typeof messages> = {
    story: 'categoryStory',
    nether: 'categoryNether',
    end: 'categoryEnd',
    adventure: 'categoryAdventure',
    husbandry: 'categoryHusbandry',
}

function categoryLabel(category: string): string {
    const key = categoryLabels[category]
    return key ? formatMessage(messages[key]) : category
}

const bundledIcons = import.meta.glob('/src/assets/advancement-icons/*.png', {
    eager: true,
    query: '?url',
    import: 'default',
}) as Record<string, string>
const bundledBySlug = new Map(
    Object.entries(bundledIcons).map(([file, url]) => [
        file
            .split('/')
            .pop()!
            .replace(/\.png$/, ''),
        url as string,
    ]),
)

// Bundled wiki icons win over the textures extracted from the game jar, so the
// page looks the same offline as it does with a warm wiki cache.
function iconUrl(entry: { id: string; iconPath: string | null }): string | null {
    const slug = entry.id.replace(/[^a-z0-9]/gi, '_')
    const bundled = bundledBySlug.get(slug)
    if (bundled) return bundled
    if (!entry.iconPath) return null
    return convertFileSrc(entry.iconPath)
}

function percent(done: number): number {
    if (total.value === 0) return 0
    return Math.min(100, Math.round((done / total.value) * 100))
}

function relativeTime(value: number | null): string {
    if (!value) return '—'
    const minutes = Math.round((Date.now() - value) / 60000)
    if (minutes < 60) return `${minutes} 分钟前`
    const hours = Math.round(minutes / 60)
    if (hours < 48) return `${hours} 小时前`
    return `${Math.round(hours / 24)} 天前`
}

function fullDate(value: number | null): string {
    if (!value) return '—'
    const date = new Date(value)
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(
        date.getDate(),
    ).padStart(2, '0')}`
}

async function load() {
    loading.value = true
    try {
        const [worldList, catalogList] = await Promise.all([
            listWorlds(props.instance.id),
            getAdvancementCatalog(props.instance.id),
        ])
        worlds.value = worldList
        catalog.value = catalogList

        const entries = await Promise.all(
            worldList
                .filter((world) => world.hasAdvancements)
                .map(async (world) => {
                    const worldDetail = await getWorldAdvancements(props.instance.id, world.name)
                    return [world.name, countCompleted(worldDetail.completed)] as const
                }),
        )
        counts.value = Object.fromEntries(entries)
    } finally {
        loading.value = false
    }
}

async function openDetail(world: string) {
    if (openWorld.value === world) return
    openWorld.value = world
    detail.value = null
    detailLoading.value = true
    activePlayer.value = null
    try {
        const result = await getWorldAdvancements(props.instance.id, world, activePlayer.value)
        detail.value = result
        // Prefer a player that belongs to a launcher account.
        const account = result.players.find(
            (player) => nameByUuid.value[normalizeUuid(player.id)] !== undefined,
        )
        activePlayer.value = account?.id ?? result.player
        // Always land on the story tab so every world opens the same way.
        activeCategory.value = 'story'
    } finally {
        detailLoading.value = false
    }
}

async function changePlayer(player: string) {
    if (!openWorld.value) return
    activePlayer.value = player
    detailLoading.value = true
    try {
        detail.value = await getWorldAdvancements(props.instance.id, openWorld.value, player)
    } finally {
        detailLoading.value = false
    }
}

function closeDetail() {
    openWorld.value = null
    detail.value = null
}

function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') closeDetail()
}

let wikiPollTimer: ReturnType<typeof setInterval> | null = null
let wikiPollAttempts = 0

function stopWikiPoll() {
    if (wikiPollTimer) clearInterval(wikiPollTimer)
    wikiPollTimer = null
}

/// The wiki icons arrive from a background download; poll until they show up so
/// the list switches over as soon as the files exist.
function startWikiPoll() {
    if (wikiPollTimer) return
    wikiPollAttempts = 0
    wikiPollTimer = setInterval(async () => {
        wikiPollAttempts += 1
        try {
            const list = await getAdvancementCatalog(props.instance.id)
            if (list.some((entry) => entry.iconPath?.includes('wiki-icons'))) {
                catalog.value = list
                stopWikiPoll()
                return
            }
        } catch {
            // Keep polling; a failed request is not fatal here.
        }
        if (wikiPollAttempts >= 30) stopWikiPoll()
    }, 3000)
}

onMounted(() => {
    void load().then(() => {
        const usesLocalIcons = catalog.value.some(
            (entry) => entry.iconPath && !entry.iconPath.includes('wiki-icons'),
        )
        if (usesLocalIcons) startWikiPoll()
    })
    void loadAccounts()
    window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
    stopWikiPoll()
    for (const url of createdHeadUrls) URL.revokeObjectURL(url)
    createdHeadUrls.clear()
    window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
    <div class="flex flex-col gap-4">
        <div class="flex items-center justify-between gap-3">
            <h2 class="m-0 text-xl font-semibold text-[var(--color-text-primary)]">
                {{ formatMessage(messages.heading) }}
            </h2>
            <button
                class="cursor-pointer rounded-lg border-0 bg-surface-3 px-3 py-1.5 text-sm text-[var(--color-text-default)] hover:bg-surface-4"
                @click="load"
            >
                {{ formatMessage(messages.refresh) }}
            </button>
        </div>

        <p v-if="loading" class="m-0 text-secondary">{{ formatMessage(messages.loading) }}</p>

        <p v-else-if="catalog.length === 0" class="m-0 text-secondary">
            {{ formatMessage(messages.catalogMissing) }}
        </p>

        <p v-else-if="rows.length === 0" class="m-0 text-secondary">
            {{ formatMessage(messages.empty) }}
        </p>

        <div v-else class="flex flex-col gap-2">
            <button
                v-for="row in rows"
                :key="row.world.name"
                class="flex cursor-pointer items-center gap-4 rounded-xl border-0 bg-surface-2 px-4 py-3 text-left hover:bg-surface-3"
                @click="openDetail(row.world.name)"
            >
                <div class="min-w-0 flex-1">
                    <p class="m-0 truncate text-base font-medium text-[var(--color-text-primary)]">
                        {{ row.world.name }}
                    </p>
                    <p class="m-0 mt-0.5 text-xs text-[var(--color-text-tertiary)]">
                        {{
                            formatMessage(messages.lastPlayed, {
                                when: relativeTime(row.world.lastPlayed),
                            })
                        }}
                    </p>
                </div>
                <div class="w-40 shrink-0">
                    <div class="h-2 w-full overflow-hidden rounded-full bg-surface-4">
                        <div
                            class="h-full rounded-full bg-brand"
                            :style="{ width: `${percent(row.done)}%` }"
                        />
                    </div>
                </div>
                <span
                    class="w-24 shrink-0 text-right text-sm tabular-nums text-[var(--color-text-secondary)]"
                >
                    {{ progressLabelFor(row.done) }}
                </span>
            </button>
        </div>

        <Teleport to="body">
            <div
                v-if="openWorld"
                class="pointer-events-none fixed inset-0 z-[9999] flex justify-end"
                @click.self="closeDetail"
            >
                <aside
                    class="pointer-events-auto flex h-full w-[560px] max-w-[100vw] flex-col border-0 border-l border-solid border-surface-4 bg-surface-1 shadow-2xl"
                >
                    <header
                        class="flex shrink-0 items-start justify-between gap-3 border-0 border-b border-solid border-surface-4 px-5 py-4"
                    >
                        <div class="min-w-0">
                            <p
                                class="m-0 truncate text-lg font-semibold text-[var(--color-text-primary)]"
                            >
                                {{ openWorld }}
                            </p>
                            <p class="m-0 mt-1 text-sm text-[var(--color-text-secondary)]">
                                {{ progressLabel }}
                            </p>
                        </div>
                        <button
                            class="cursor-pointer rounded-lg border-0 bg-transparent px-2 py-1 text-xl leading-none text-[var(--color-text-secondary)] hover:bg-surface-3"
                            @click="closeDetail"
                        >
                            ×
                        </button>
                    </header>

                    <div v-if="detailLoading" class="p-5 text-secondary">…</div>

                    <template v-else-if="detail">
                        <div class="flex items-center gap-2 px-5 py-3">
                            <span class="text-sm text-[var(--color-text-secondary)]">
                                {{ formatMessage(messages.player) }}
                            </span>
                            <span
                                v-if="detail.players.length === 0"
                                class="text-sm text-[var(--color-text-tertiary)]"
                            >
                                {{ formatMessage(messages.noPlayer) }}
                            </span>
                            <img
                                v-if="detail.players.length > 0"
                                :src="playerAvatar"
                                :alt="playerName"
                                :title="playerName"
                                class="size-6 shrink-0 rounded-[4px] [image-rendering:pixelated]"
                            />
                            <select
                                v-if="accountPlayers.length > 0"
                                class="rounded-lg border border-solid border-surface-4 bg-surface-2 px-2 py-1 text-sm text-[var(--color-text-primary)]"
                                :value="activePlayer ?? ''"
                                @change="changePlayer(($event.target as HTMLSelectElement).value)"
                            >
                                <option
                                    v-for="player in accountPlayers"
                                    :key="player.id"
                                    :value="player.id"
                                >
                                    {{ nameByUuid[normalizeUuid(player.id)] }}
                                </option>
                            </select>
                        </div>

                        <div class="flex w-full min-w-0 flex-wrap gap-1.5 px-5 pb-3">
                            <button
                                v-for="category in categories"
                                :key="category"
                                class="cursor-pointer whitespace-nowrap rounded-full border-0 px-3 py-1 text-sm"
                                :class="
                                    activeCategory === category
                                        ? 'bg-brand text-[var(--color-accent-contrast)]'
                                        : 'bg-surface-3 text-[var(--color-text-secondary)] hover:bg-surface-4'
                                "
                                @click="activeCategory = category"
                            >
                                {{ categoryLabel(category) }}
                                <span class="ml-1 tabular-nums">
                                    {{ categoryCounts[category]?.done ?? 0 }}/{{
                                        categoryCounts[category]?.total ?? 0
                                    }}
                                </span>
                            </button>
                        </div>

                        <div class="flex-1 overflow-y-auto px-5 pb-5">
                            <div
                                v-for="item in visibleEntries"
                                :key="item.entry.id"
                                class="mb-2 flex items-start gap-3 rounded-lg bg-surface-2 p-3"
                                :class="{ 'opacity-60': !item.completed }"
                            >
                                <div class="flex size-8 shrink-0 items-center justify-center">
                                    <img
                                        v-if="!item.masked && iconUrl(item.entry)"
                                        :src="iconUrl(item.entry) ?? ''"
                                        :alt="item.entry.title"
                                        class="size-8 object-contain [image-rendering:pixelated]"
                                    />
                                    <span v-else class="text-sm text-[var(--color-text-tertiary)]">
                                        ?
                                    </span>
                                </div>
                                <div class="min-w-0 flex-1">
                                    <p
                                        class="m-0 text-sm font-medium text-[var(--color-text-primary)]"
                                    >
                                        {{
                                            item.masked
                                                ? formatMessage(messages.hidden)
                                                : item.entry.title
                                        }}
                                    </p>
                                    <p
                                        class="m-0 mt-0.5 text-xs text-[var(--color-text-secondary)]"
                                    >
                                        {{
                                            item.masked
                                                ? formatMessage(messages.hiddenHint)
                                                : item.entry.description
                                        }}
                                    </p>
                                    <p v-if="item.completed" class="m-0 mt-1 text-xs text-brand">
                                        {{
                                            formatMessage(messages.completedOn, {
                                                date: fullDate(item.completedAt),
                                            })
                                        }}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </template>
                </aside>
            </div>
        </Teleport>
    </div>
</template>
