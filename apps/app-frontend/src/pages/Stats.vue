<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { type StatsOverview, getStatsOverview } from '@/helpers/stats'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

const { formatMessage, locale } = useVIntl()

const messages = defineMessages({
    title: { id: 'app.stats.title', defaultMessage: 'Statistics' },
    instances: { id: 'app.stats.instances', defaultMessage: 'Instances' },
    screenshots: { id: 'app.stats.screenshots', defaultMessage: 'Screenshots' },
    heatmap: { id: 'app.stats.heatmap', defaultMessage: 'Playtime' },
    less: { id: 'app.stats.less', defaultMessage: 'Less' },
    more: { id: 'app.stats.more', defaultMessage: 'More' },
    legend1: { id: 'app.stats.legend1', defaultMessage: 'Up to 30 min' },
    legend2: { id: 'app.stats.legend2', defaultMessage: '30 min – 2 h' },
    legend3: { id: 'app.stats.legend3', defaultMessage: '2 – 5 h' },
    legend4: { id: 'app.stats.legend4', defaultMessage: 'Over 5 h' },
    ranking: { id: 'app.stats.ranking', defaultMessage: 'Playtime by instance' },
    timeline: { id: 'app.stats.timeline', defaultMessage: 'Achievement timeline' },
    timeUnknown: { id: 'app.stats.time-unknown', defaultMessage: 'Time unknown' },
    timeUnknownHint: {
        id: 'app.stats.time-unknown-hint',
        defaultMessage: 'Not counted in the yearly total',
    },
    empty: {
        id: 'app.stats.empty',
        defaultMessage: 'No statistics yet. Start the game and your records will show up here.',
    },
    loading: { id: 'app.stats.loading', defaultMessage: 'Collecting statistics…' },
    hours: { id: 'app.stats.hours', defaultMessage: '{hours} h {minutes} min' },
    hoursOnly: { id: 'app.stats.hours-only', defaultMessage: '{hours} h' },
    minutes: { id: 'app.stats.minutes', defaultMessage: '{minutes} min' },
    noPlay: { id: 'app.stats.no-play', defaultMessage: 'No playtime' },
    refresh: { id: 'app.stats.refresh', defaultMessage: 'Refresh' },
    retry: { id: 'app.stats.retry', defaultMessage: 'Try again' },
    loadError: {
        id: 'app.stats.load-error',
        defaultMessage: 'Could not load statistics.',
    },
    yearPlaytime: { id: 'app.stats.year-playtime', defaultMessage: 'Playtime in {year}' },
    yearAchievements: {
        id: 'app.stats.year-achievements',
        defaultMessage: 'Unlocked in {year}',
    },
    weekdayMon: { id: 'app.stats.weekday-mon', defaultMessage: 'Mon' },
    weekdayWed: { id: 'app.stats.weekday-wed', defaultMessage: 'Wed' },
    weekdayFri: { id: 'app.stats.weekday-fri', defaultMessage: 'Fri' },
})

const loading = ref(true)
const data = ref<StatsOverview | null>(null)
const error = ref<string | null>(null)
const year = ref(new Date().getFullYear())
let loadSequence = 0
let iconPollTimer: ReturnType<typeof setTimeout> | null = null
let iconPollAttempts = 0

async function load(force = false) {
    const sequence = ++loadSequence
    if (force) iconPollAttempts = 0
    loading.value = true
    error.value = null
    try {
        const result = await getStatsOverview(year.value, force)
        if (sequence !== loadSequence) return
        data.value = result
        scheduleIconPoll()
    } catch (cause) {
        if (sequence !== loadSequence) return
        error.value = cause instanceof Error ? cause.message : String(cause)
    } finally {
        if (sequence === loadSequence) loading.value = false
    }
}

/// Achievement icons download in the background; poll a few times so the
/// timeline picks them up without the user pressing refresh.
function scheduleIconPoll() {
    if (iconPollTimer || iconPollAttempts >= 6) return
    const missing = (data.value?.timeline ?? []).some((entry) => !entry.iconPath)
    if (!missing) return
    iconPollTimer = setTimeout(async () => {
        iconPollTimer = null
        iconPollAttempts += 1
        await load()
    }, 5000)
}

function stopIconPoll() {
    if (iconPollTimer) clearTimeout(iconPollTimer)
    iconPollTimer = null
}

function formatDuration(seconds: number): string {
    const total = Math.max(0, Math.round(seconds))
    const hours = Math.floor(total / 3600)
    const minutes = Math.floor((total % 3600) / 60)
    if (hours > 0 && minutes > 0) return formatMessage(messages.hours, { hours, minutes })
    if (hours > 0) return formatMessage(messages.hoursOnly, { hours })
    if (minutes > 0) return formatMessage(messages.minutes, { minutes })
    return formatMessage(messages.minutes, { minutes: 0 })
}

function formatDate(value: number | null): string {
    if (!value) return '—'
    const date = new Date(value)
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(
        date.getDate(),
    ).padStart(2, '0')}`
}

const dayMap = computed(() => {
    const map = new Map<string, { seconds: number; topInstance: string | null }>()
    for (const day of data.value?.days ?? []) {
        map.set(day.date, { seconds: day.seconds, topInstance: day.topInstance })
    }
    return map
})

function levelFor(seconds: number): number {
    if (seconds <= 0) return 0
    // Fixed thresholds: 30 min, 2 h and 5 h.
    if (seconds > 18_000) return 4
    if (seconds > 7_200) return 3
    if (seconds > 1_800) return 2
    return 1
}

const heatmapStart = computed(() => {
    const jan1 = new Date(Date.UTC(year.value, 0, 1))
    // Sunday-first weeks, like GitHub's contribution graph.
    const offset = jan1.getUTCDay()
    const start = new Date(jan1)
    start.setUTCDate(jan1.getUTCDate() - offset)
    return start
})

const heatmapWeeks = computed(() => {
    const start = heatmapStart.value
    const end = new Date(Date.UTC(year.value, 11, 31))
    const cursor = new Date(start)
    const weeks: Array<
        Array<{
            date: string
            inYear: boolean
            seconds: number
            topInstance: string | null
        }>
    > = []

    while (cursor <= end) {
        const week: Array<{
            date: string
            inYear: boolean
            seconds: number
            topInstance: string | null
        }> = []
        for (let index = 0; index < 7; index += 1) {
            const key = cursor.toISOString().slice(0, 10)
            const entry = dayMap.value.get(key)
            week.push({
                date: key,
                inYear: cursor.getUTCFullYear() === year.value,
                seconds: entry?.seconds ?? 0,
                topInstance: entry?.topInstance ?? null,
            })
            cursor.setUTCDate(cursor.getUTCDate() + 1)
        }
        weeks.push(week)
    }
    return weeks
})

const monthLabels = computed(() => {
    const start = heatmapStart.value
    const formatter = new Intl.DateTimeFormat(locale.value, { month: 'short' })
    const labels: Array<{ index: number; label: string }> = []
    for (let month = 0; month < 12; month += 1) {
        const first = new Date(Date.UTC(year.value, month, 1))
        const days = Math.round((first.getTime() - start.getTime()) / 86_400_000)
        labels.push({
            index: Math.floor(days / 7),
            label: formatter.format(first),
        })
    }
    return labels
})

/// Playtime for the selected year, matching what the heatmap shows.
const yearSeconds = computed(() =>
    (data.value?.days ?? []).reduce((total, day) => total + day.seconds, 0),
)

/// Unlocks that happened in the selected year.
const yearTimeline = computed(() =>
    (data.value?.timeline ?? []).filter(
        (entry) => entry.completedAt && new Date(entry.completedAt).getFullYear() === year.value,
    ),
)

const yearUnlocked = computed(() => new Set(yearTimeline.value.map((entry) => entry.id)).size)

const timelineGroups = computed(() => {
    const groups: Array<{ date: string; entries: StatsOverview['timeline'] }> = []
    for (const entry of yearTimeline.value) {
        const date = formatDate(entry.completedAt)
        const last = groups[groups.length - 1]
        if (last && last.date === date) last.entries.push(entry)
        else groups.push({ date, entries: [entry] })
    }
    return groups
})

/// Unlocks whose save file carries no completion time; they still count towards
/// the library but cannot be placed on a date.
const timeUnknownEntries = computed(() =>
    (data.value?.timeline ?? []).filter((entry) => !entry.completedAt),
)

const ranking = computed(() => {
    const list = data.value?.instances ?? []
    const max = list[0]?.seconds || 1
    return list.slice(0, 8).map((entry) => ({
        ...entry,
        percent: Math.max(2, Math.round((entry.seconds / max) * 100)),
    }))
})

function heatCellLabel(day: { date: string; seconds: number; topInstance: string | null }): string {
    if (day.seconds <= 0) return `${day.date} · ${formatMessage(messages.noPlay)}`
    const duration = formatDuration(day.seconds)
    return day.topInstance
        ? `${day.date} · ${duration} · ${day.topInstance}`
        : `${day.date} · ${duration}`
}

function iconUrl(path: string | null): string | null {
    return path ? convertFileSrc(path) : null
}

onMounted(() => {
    void load()
})

onBeforeUnmount(() => {
    stopIconPoll()
})
</script>

<template>
    <div class="mx-auto flex w-full max-w-[1100px] flex-col gap-5 p-6">
        <div class="flex items-center justify-between">
            <h1 class="m-0 text-2xl font-bold text-[var(--color-text-primary)]">
                {{ formatMessage(messages.title) }}
            </h1>
            <button
                type="button"
                class="rounded-lg bg-surface-3 px-3 py-1.5 text-sm text-[var(--color-text-primary)] hover:bg-surface-4"
                :disabled="loading"
                @click="load(true)"
            >
                {{ formatMessage(messages.refresh) }}
            </button>
        </div>

        <p v-if="loading" class="m-0 text-secondary">{{ formatMessage(messages.loading) }}</p>

        <div v-else-if="error" class="flex flex-col items-start gap-2 rounded-xl bg-surface-2 p-4">
            <p class="m-0 text-sm text-[var(--color-text-primary)]">
                {{ formatMessage(messages.loadError) }}
            </p>
            <p class="m-0 text-xs text-[var(--color-text-tertiary)]">{{ error }}</p>
            <button
                type="button"
                class="rounded-lg bg-surface-3 px-3 py-1.5 text-sm text-[var(--color-text-primary)] hover:bg-surface-4"
                @click="load()"
            >
                {{ formatMessage(messages.retry) }}
            </button>
        </div>

        <p v-else-if="!data" class="m-0 text-secondary">{{ formatMessage(messages.empty) }}</p>

        <template v-else>
            <div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
                <div class="flex flex-col gap-1 rounded-xl bg-surface-2 p-4">
                    <span class="text-xs text-[var(--color-text-tertiary)]">
                        {{ formatMessage(messages.yearPlaytime, { year }) }}
                    </span>
                    <span class="text-xl font-semibold text-[var(--color-text-primary)]">
                        {{ formatDuration(yearSeconds) }}
                    </span>
                </div>
                <div class="flex flex-col gap-1 rounded-xl bg-surface-2 p-4">
                    <span class="text-xs text-[var(--color-text-tertiary)]">
                        {{ formatMessage(messages.instances) }}
                    </span>
                    <span class="text-xl font-semibold text-[var(--color-text-primary)]">
                        {{ data.instanceCount }}
                    </span>
                </div>
                <div class="flex flex-col gap-1 rounded-xl bg-surface-2 p-4">
                    <span class="text-xs text-[var(--color-text-tertiary)]">
                        {{ formatMessage(messages.screenshots) }}
                    </span>
                    <span class="text-xl font-semibold text-[var(--color-text-primary)]">
                        {{ data.screenshotCount }}
                    </span>
                </div>
                <div class="flex flex-col gap-1 rounded-xl bg-surface-2 p-4">
                    <span class="text-xs text-[var(--color-text-tertiary)]">
                        {{ formatMessage(messages.yearAchievements, { year }) }}
                    </span>
                    <span class="text-xl font-semibold text-[var(--color-text-primary)]">
                        {{ yearUnlocked }}
                        <span class="text-sm font-normal text-[var(--color-text-tertiary)]">
                            / {{ data.achievementTotal }}
                        </span>
                    </span>
                </div>
            </div>

            <section class="flex flex-col gap-3 rounded-xl bg-surface-2 p-4">
                <div class="flex items-center justify-between">
                    <h2 class="m-0 text-base font-semibold text-[var(--color-text-primary)]">
                        {{ formatMessage(messages.heatmap) }}
                    </h2>
                    <select
                        v-model.number="year"
                        class="cursor-pointer rounded-lg bg-surface-3 px-2 py-1 text-xs text-[var(--color-text-primary)] outline-none"
                        @change="load()"
                    >
                        <option
                            v-for="option in data?.years ?? [year]"
                            :key="option"
                            :value="option"
                        >
                            {{ option }}
                        </option>
                    </select>
                </div>

                <div class="flex gap-2 overflow-x-auto pb-1">
                    <div
                        class="grid shrink-0 text-[10px] leading-none text-[var(--color-text-tertiary)]"
                        :style="{ gridTemplateRows: '16px repeat(7, 10px)', rowGap: '3px' }"
                    >
                        <span />
                        <span />
                        <span class="flex items-center">{{
                            formatMessage(messages.weekdayMon)
                        }}</span>
                        <span />
                        <span class="flex items-center">{{
                            formatMessage(messages.weekdayWed)
                        }}</span>
                        <span />
                        <span class="flex items-center">{{
                            formatMessage(messages.weekdayFri)
                        }}</span>
                        <span />
                    </div>
                    <div
                        class="grid"
                        :style="{
                            gridTemplateColumns: `repeat(${heatmapWeeks.length}, 10px)`,
                            gridTemplateRows: '16px repeat(7, 10px)',
                            gap: '3px',
                        }"
                    >
                        <span
                            v-for="label in monthLabels"
                            :key="label.index"
                            class="whitespace-nowrap text-[10px] leading-none text-[var(--color-text-tertiary)]"
                            :style="{ gridColumnStart: label.index + 1, gridRow: 1 }"
                        >
                            {{ label.label }}
                        </span>
                        <template v-for="(week, weekIndex) in heatmapWeeks" :key="weekIndex">
                            <div
                                v-for="(day, dayIndex) in week"
                                :key="day.date"
                                class="heat-cell rounded-[2px]"
                                :style="{ gridColumnStart: weekIndex + 1, gridRow: dayIndex + 2 }"
                                :class="
                                    !day.inYear
                                        ? 'opacity-0'
                                        : levelFor(day.seconds) === 0
                                          ? 'bg-surface-4'
                                          : `heat-level-${levelFor(day.seconds)}`
                                "
                                :title="day.inYear ? heatCellLabel(day) : ''"
                            />
                        </template>
                    </div>
                </div>

                <div
                    class="flex items-center justify-end gap-1 text-[10px] text-[var(--color-text-tertiary)]"
                >
                    <span>{{ formatMessage(messages.less) }}</span>
                    <span
                        class="heat-cell size-[10px] rounded-[2px] bg-surface-4"
                        :title="formatMessage(messages.noPlay)"
                    />
                    <span
                        class="heat-cell heat-level-1 size-[10px] rounded-[2px]"
                        :title="formatMessage(messages.legend1)"
                    />
                    <span
                        class="heat-cell heat-level-2 size-[10px] rounded-[2px]"
                        :title="formatMessage(messages.legend2)"
                    />
                    <span
                        class="heat-cell heat-level-3 size-[10px] rounded-[2px]"
                        :title="formatMessage(messages.legend3)"
                    />
                    <span
                        class="heat-cell heat-level-4 size-[10px] rounded-[2px]"
                        :title="formatMessage(messages.legend4)"
                    />
                    <span>{{ formatMessage(messages.more) }}</span>
                </div>
            </section>

            <div class="grid gap-4 lg:grid-cols-2">
                <section class="flex flex-col gap-3 rounded-xl bg-surface-2 p-4">
                    <h2 class="m-0 text-base font-semibold text-[var(--color-text-primary)]">
                        {{ formatMessage(messages.ranking) }}
                    </h2>
                    <div
                        v-if="ranking.length === 0"
                        class="text-sm text-[var(--color-text-tertiary)]"
                    >
                        {{ formatMessage(messages.empty) }}
                    </div>
                    <div v-for="entry in ranking" :key="entry.name" class="flex flex-col gap-1">
                        <div class="flex items-center justify-between gap-2 text-sm">
                            <span class="truncate text-[var(--color-text-primary)]">{{
                                entry.name
                            }}</span>
                            <span class="shrink-0 text-[var(--color-text-tertiary)]">
                                {{ formatDuration(entry.seconds) }}
                            </span>
                        </div>
                        <div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-4">
                            <div
                                class="h-full rounded-full bg-brand"
                                :style="{ width: `${entry.percent}%` }"
                            />
                        </div>
                    </div>
                </section>

                <section class="flex flex-col gap-3 rounded-xl bg-surface-2 p-4">
                    <h2 class="m-0 text-base font-semibold text-[var(--color-text-primary)]">
                        {{ formatMessage(messages.timeline) }}
                    </h2>
                    <div
                        v-if="timelineGroups.length === 0"
                        class="text-sm text-[var(--color-text-tertiary)]"
                    >
                        {{ formatMessage(messages.empty) }}
                    </div>
                    <div class="flex max-h-[360px] flex-col gap-3 overflow-y-auto pr-1">
                        <div
                            v-for="group in timelineGroups"
                            :key="group.date"
                            class="flex flex-col gap-1.5"
                        >
                            <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                                {{ group.date }}
                            </span>
                            <div
                                v-for="entry in group.entries"
                                :key="entry.id"
                                class="flex items-center gap-2"
                            >
                                <img
                                    v-if="iconUrl(entry.iconPath)"
                                    :src="iconUrl(entry.iconPath) ?? ''"
                                    :alt="entry.title"
                                    class="size-6 shrink-0 object-contain [image-rendering:pixelated]"
                                />
                                <div class="min-w-0 flex-1">
                                    <p
                                        class="m-0 truncate text-sm text-[var(--color-text-primary)]"
                                    >
                                        {{ entry.title }}
                                    </p>
                                    <p
                                        class="m-0 truncate text-[11px] text-[var(--color-text-tertiary)]"
                                    >
                                        {{ entry.instanceName }} / {{ entry.worldName }}
                                    </p>
                                </div>
                            </div>
                        </div>

                        <div v-if="timeUnknownEntries.length > 0" class="flex flex-col gap-1.5">
                            <span class="text-xs font-medium text-[var(--color-text-tertiary)]">
                                {{ formatMessage(messages.timeUnknown) }}
                                <span class="font-normal">
                                    · {{ formatMessage(messages.timeUnknownHint) }}
                                </span>
                            </span>
                            <div
                                v-for="entry in timeUnknownEntries"
                                :key="entry.id"
                                class="flex items-center gap-2"
                            >
                                <img
                                    v-if="iconUrl(entry.iconPath)"
                                    :src="iconUrl(entry.iconPath) ?? ''"
                                    :alt="entry.title"
                                    class="size-6 shrink-0 object-contain [image-rendering:pixelated]"
                                />
                                <div class="min-w-0 flex-1">
                                    <p
                                        class="m-0 truncate text-sm text-[var(--color-text-primary)]"
                                    >
                                        {{ entry.title }}
                                    </p>
                                    <p
                                        class="m-0 truncate text-[11px] text-[var(--color-text-tertiary)]"
                                    >
                                        {{ entry.instanceName }} / {{ entry.worldName }}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
            </div>
        </template>
    </div>
</template>
<style scoped>
/* GitHub-style contribution ramp */
.heat-cell {
    box-sizing: border-box;
}

.heat-level-1 {
    background: #a5e4b3;
    border-color: transparent;
}

.heat-level-2 {
    background: #48ba68;
    border-color: transparent;
}

.heat-level-3 {
    background: #2d9e4c;
    border-color: transparent;
}

.heat-level-4 {
    background: #126029;
    border-color: transparent;
}
</style>
