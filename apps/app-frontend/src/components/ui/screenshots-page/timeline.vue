<script setup lang="ts">
import { defineMessages, useFormatDateTime, useVIntl } from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed } from 'vue'

import type { InstanceScreenshot } from '@/helpers/screenshots'

const props = defineProps<{
    screenshots: InstanceScreenshot[]
    showInstance: boolean
}>()

const emit = defineEmits<{
    (e: 'activate', screenshot: InstanceScreenshot, event: MouseEvent): void
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
    dayLabel: { id: 'app.screenshots.timeline.day', defaultMessage: '{date} · {count}' },
    empty: { id: 'app.screenshots.timeline.empty', defaultMessage: 'Nothing to show yet' },
})

const formatDay = useFormatDateTime({ dateStyle: 'full' })
const formatTime = useFormatDateTime({ timeStyle: 'short' })

type TimelineDay = {
    id: string
    label: string
    entries: InstanceScreenshot[]
}

const days = computed<TimelineDay[]>(() => {
    const grouped = new Map<string, InstanceScreenshot[]>()
    for (const screenshot of props.screenshots) {
        const key = dayjs(screenshot.created_at).format('YYYY-MM-DD')
        const bucket = grouped.get(key) ?? []
        bucket.push(screenshot)
        grouped.set(key, bucket)
    }

    return [...grouped.entries()].map(([id, entries]) => ({
        id,
        label: formatMessage(messages.dayLabel, {
            date: formatDay(dayjs(entries[0].created_at).toDate()),
            count: entries.length,
        }),
        entries,
    }))
})
</script>

<template>
    <div class="flex w-full flex-col gap-6 pb-10">
        <p v-if="days.length === 0" class="m-0 text-secondary">
            {{ formatMessage(messages.empty) }}
        </p>
        <section v-for="day in days" :key="day.id" class="flex flex-col gap-2">
            <h2
                class="sticky top-0 z-[2] m-0 bg-[var(--color-raised-bg)] py-2 text-base font-semibold text-[var(--color-text-primary)]"
            >
                {{ day.label }}
            </h2>
            <ol class="relative m-0 flex list-none flex-col gap-2 p-0 pl-5">
                <span
                    class="pointer-events-none absolute inset-y-1 left-[5px] w-px bg-[var(--color-divider)]"
                    aria-hidden="true"
                />
                <li
                    v-for="screenshot in day.entries"
                    :key="screenshot.id"
                    class="relative flex cursor-pointer items-center gap-3 rounded-xl border border-[var(--color-divider)] bg-[var(--color-raised-bg)] p-2 transition-colors hover:border-brand"
                    @click="emit('activate', screenshot, $event)"
                >
                    <span
                        class="absolute -left-[17px] size-2 rounded-full border-2 border-[var(--color-raised-bg)] bg-brand"
                        aria-hidden="true"
                    />
                    <img
                        :src="screenshot.url"
                        :alt="screenshot.file_name"
                        loading="lazy"
                        class="aspect-video w-40 shrink-0 rounded-lg object-cover"
                    />
                    <div class="flex min-w-0 flex-col gap-0.5">
                        <span class="text-sm font-medium text-[var(--color-text-primary)]">
                            {{ formatTime(dayjs(screenshot.created_at).toDate()) }}
                        </span>
                        <span
                            v-if="showInstance"
                            class="truncate text-xs text-[var(--color-text-secondary)]"
                        >
                            {{ screenshot.instance_name }}
                        </span>
                        <span class="truncate text-xs text-[var(--color-text-tertiary)]">
                            {{ screenshot.file_name }}
                        </span>
                    </div>
                </li>
            </ol>
        </section>
    </div>
</template>
