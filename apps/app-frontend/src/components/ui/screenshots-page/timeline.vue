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

const formatFullDate = useFormatDateTime({ dateStyle: 'full' })
const formatTime = useFormatDateTime({ timeStyle: 'short' })

/** Days with more screenshots than this get hour separators inside them. */
const HOUR_SEPARATOR_THRESHOLD = 5

type TimelineEntry =
    | { kind: 'hour'; key: string; label: string }
    | { kind: 'shot'; key: string; screenshot: InstanceScreenshot }

type TimelineDay = {
    id: string
    label: string
    entries: TimelineEntry[]
}

const days = computed<TimelineDay[]>(() => {
    const grouped = new Map<string, InstanceScreenshot[]>()
    for (const screenshot of props.screenshots) {
        const key = dayjs(screenshot.created_at).format('YYYY-MM-DD')
        const bucket = grouped.get(key) ?? []
        bucket.push(screenshot)
        grouped.set(key, bucket)
    }

    return [...grouped.entries()].map(([id, shots]) => {
        const entries: TimelineEntry[] = []
        const useHours = shots.length > HOUR_SEPARATOR_THRESHOLD
        let currentHour = ''

        for (const screenshot of shots) {
            const moment = dayjs(screenshot.created_at)
            const hour = moment.format('HH')
            if (useHours && hour !== currentHour) {
                currentHour = hour
                entries.push({
                    kind: 'hour',
                    key: `${id}-${hour}`,
                    label: moment.startOf('hour').format('HH:00'),
                })
            }
            entries.push({ kind: 'shot', key: screenshot.id, screenshot })
        }

        return {
            id,
            label: formatMessage(messages.dayLabel, {
                date: formatFullDate(dayjs(shots[0].created_at).toDate()),
                count: shots.length,
            }),
            entries,
        }
    })
})

</script>

<template>
    <div class="flex w-full flex-col gap-6 pb-10">
        <p v-if="days.length === 0" class="m-0 text-secondary">
            {{ formatMessage(messages.empty) }}
        </p>
        <section v-for="day in days" :key="day.id" class="flex flex-col gap-2">
            <h2
                class="m-0 flex items-center gap-2 rounded-xl bg-surface-2 px-4 py-2.5 text-base font-semibold text-[var(--color-text-default)]"
            >
                {{ day.label }}
            </h2>
            <ol class="relative m-0 flex list-none flex-col gap-2 p-0 pl-5">
                <span
                    class="pointer-events-none absolute inset-y-1 left-[5px] w-px bg-surface-5"
                    aria-hidden="true"
                />
                <template v-for="entry in day.entries" :key="entry.key">
                    <li
                        v-if="entry.kind === 'hour'"
                        class="ml-1 mt-1 text-sm font-medium text-[var(--color-text-tertiary)]"
                    >
                        {{ entry.label }}
                    </li>
                    <li
                        v-else
                        class="relative flex cursor-pointer items-center gap-5 rounded-xl bg-surface-2 p-3 transition-colors hover:bg-surface-3 [contain-intrinsic-size:0_240px] [content-visibility:auto]"
                        @click="emit('activate', entry.screenshot, $event)"
                    >
                        <span
                            class="absolute -left-[19px] size-2 rounded-full bg-brand ring-2 ring-bg"
                            aria-hidden="true"
                        />
                        <img
                            :src="entry.screenshot.url"
                            :alt="entry.screenshot.file_name"
                            loading="lazy"
                            class="aspect-video w-96 shrink-0 rounded-lg object-cover"
                        />
                        <div class="flex min-w-0 flex-col gap-0.5">
                            <span class="text-xl font-semibold text-[var(--color-text-primary)]">
                                {{ formatTime(dayjs(entry.screenshot.created_at).toDate()) }}
                            </span>
                            <span
                                v-if="showInstance"
                                class="truncate text-base text-[var(--color-text-secondary)]"
                            >
                                {{ entry.screenshot.instance_name }}
                            </span>
                            <span class="truncate text-sm text-[var(--color-text-tertiary)]">
                                {{ entry.screenshot.file_name }}
                            </span>
                        </div>
                    </li>
                </template>
            </ol>
        </section>
    </div>
</template>
