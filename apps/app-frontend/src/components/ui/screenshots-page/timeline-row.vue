<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { MAX_SCREENSHOT_NOTE_LENGTH } from '@/helpers/screenshot-notes'
import { useImageThumbnail } from '@/composables/use-image-thumbnail'
import type { InstanceScreenshot } from '@/helpers/screenshots'
import { defineMessages, useFormatDateTime, useVIntl } from '@modrinth/ui'
import { nextTick, ref, watch } from 'vue'

const props = defineProps<{
    screenshot: InstanceScreenshot
    showInstance: boolean
    note?: string
    editing: boolean
    selected?: boolean
}>()

const emit = defineEmits<{
    (e: 'activate', screenshot: InstanceScreenshot, event: MouseEvent): void
    (e: 'contextmenu', screenshot: InstanceScreenshot, event: MouseEvent): void
    (e: 'save-note', screenshot: InstanceScreenshot, note: string): void
    (e: 'cancel-note'): void
    (e: 'toggle-select', screenshot: InstanceScreenshot): void
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
    addNote: { id: 'app.screenshots.note.placeholder', defaultMessage: 'Add a note…' },
})

const formatTime = useFormatDateTime({ timeStyle: 'short' })

const thumbnail = useImageThumbnail(
    () => ({
        instanceId: props.screenshot.instance_id,
        filePath: `screenshots/${props.screenshot.file_name}`,
        path: props.screenshot.path,
    }),
    512,
    () => props.screenshot.modified_at,
)

const draft = ref(props.note ?? '')
const input = ref<HTMLInputElement>()

watch(
    () => props.editing,
    async (editing) => {
        if (!editing) return
        draft.value = props.note ?? ''
        await nextTick()
        input.value?.focus()
        input.value?.select()
    },
)

function commit() {
    emit('save-note', props.screenshot, draft.value.trim())
}
</script>

<template>
    <div
        class="group/row relative flex h-[256px] cursor-pointer items-center gap-8 rounded-xl bg-surface-2 p-3 transition-colors hover:bg-surface-3"
        :class="{ 'ring-2 ring-brand': selected }"
        @click.stop="emit('activate', screenshot, $event)"
        @contextmenu.prevent.stop="emit('contextmenu', screenshot, $event)"
    >
        <span
            class="absolute -left-[19px] size-2 rounded-full bg-brand ring-2 ring-bg"
            aria-hidden="true"
        />
        <div class="relative shrink-0">
            <img
                v-if="thumbnail"
                :src="thumbnail"
                :alt="screenshot.file_name"
                class="aspect-video w-96 shrink-0 rounded-lg object-cover"
            />
            <div
                v-else
                class="aspect-video w-96 shrink-0 animate-pulse rounded-lg bg-surface-3"
            />
            <button
                class="absolute right-2 top-2 z-[3] flex size-6 cursor-pointer items-center justify-center rounded-full border-2 border-solid border-white/70 bg-black/30 text-transparent transition-all group-hover/row:opacity-100"
                :class="
                    selected
                        ? 'border-brand bg-brand text-white'
                        : 'opacity-0 hover:border-brand hover:bg-brand hover:text-white'
                "
                :aria-pressed="selected"
                aria-label="选择截图"
                @click.stop="emit('toggle-select', screenshot)"
            >
                <CheckIcon class="size-4" :stroke-width="3" />
            </button>
        </div>
        <div class="flex min-w-0 shrink-0 flex-col gap-0.5">
            <span class="text-xl font-semibold text-[var(--color-text-primary)]">
                {{ formatTime(new Date(screenshot.created_at)) }}
            </span>
            <span
                v-if="showInstance"
                class="truncate text-base text-[var(--color-text-secondary)]"
            >
                {{ screenshot.instance_name }}
            </span>
            <span class="truncate text-sm text-[var(--color-text-tertiary)]">
                {{ screenshot.file_name }}
            </span>
        </div>

        <template v-if="note || editing">
            <span
                class="h-16 w-px shrink-0 self-center bg-surface-5"
                aria-hidden="true"
            />
            <div class="flex min-w-0 flex-1 flex-col">
                <input
                    v-if="editing"
                    ref="input"
                    v-model="draft"
                    :maxlength="MAX_SCREENSHOT_NOTE_LENGTH"
                    class="w-full rounded-lg border border-solid border-brand bg-surface-1 px-3 py-1.5 text-base text-[var(--color-text-primary)] outline-none"
                    @click.stop
                    @keydown.enter.prevent="commit"
                    @keydown.esc.prevent="emit('cancel-note')"
                    @blur="commit"
                />
                <p
                    v-else
                    class="m-0 line-clamp-2 text-base text-[var(--color-text-secondary)]"
                    :title="note"
                >
                    {{ note }}
                </p>
            </div>
        </template>
    </div>
</template>
