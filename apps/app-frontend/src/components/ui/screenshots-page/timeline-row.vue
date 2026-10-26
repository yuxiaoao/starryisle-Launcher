<script setup lang="ts">
import { useImageThumbnail } from '@/composables/use-image-thumbnail'
import type { InstanceScreenshot } from '@/helpers/screenshots'
import { useFormatDateTime } from '@modrinth/ui'

const props = defineProps<{
    screenshot: InstanceScreenshot
    showInstance: boolean
}>()

const emit = defineEmits<{
    (e: 'activate', screenshot: InstanceScreenshot, event: MouseEvent): void
}>()

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
</script>

<template>
    <div
        class="relative flex h-[252px] cursor-pointer items-center gap-8 rounded-xl bg-surface-2 p-3 transition-colors hover:bg-surface-3"
        @click="emit('activate', screenshot, $event)"
    >
        <span
            class="absolute -left-[19px] size-2 rounded-full bg-brand ring-2 ring-bg"
            aria-hidden="true"
        />
        <img
            v-if="thumbnail"
            :src="thumbnail"
            :alt="screenshot.file_name"
            class="aspect-video w-96 shrink-0 rounded-lg object-cover"
        />
        <div v-else class="aspect-video w-96 shrink-0 animate-pulse rounded-lg bg-surface-3" />
        <div class="flex min-w-0 flex-col gap-0.5">
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
    </div>
</template>
