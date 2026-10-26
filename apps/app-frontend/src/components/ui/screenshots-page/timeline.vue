<script setup lang="ts">
import { defineMessages, useFormatDateTime, useVIntl } from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import type { InstanceScreenshot } from '@/helpers/screenshots'

import TimelineRow from './timeline-row.vue'

const props = defineProps<{
    screenshots: InstanceScreenshot[]
    showInstance: boolean
    noteFor?: (screenshot: InstanceScreenshot) => string | undefined
    editingKey?: string | null
}>()

const emit = defineEmits<{
    (e: 'activate', screenshot: InstanceScreenshot, event: MouseEvent): void
    (e: 'contextmenu', screenshot: InstanceScreenshot, event: MouseEvent): void
    (e: 'save-note', screenshot: InstanceScreenshot, note: string): void
    (e: 'cancel-note'): void
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
    dayLabel: { id: 'app.screenshots.timeline.day', defaultMessage: '{date} · {count}' },
    empty: { id: 'app.screenshots.timeline.empty', defaultMessage: 'Nothing to show yet' },
})

const formatFullDate = useFormatDateTime({ dateStyle: 'full' })

/** Days with more screenshots than this get hour separators inside them. */
const HOUR_SEPARATOR_THRESHOLD = 5

const DAY_HEIGHT = 60
const HOUR_HEIGHT = 34
const ROW_HEIGHT = 268
const DAY_GAP = 24
const OVERSCAN = 600

type TimelineItem =
    | { kind: 'day'; key: string; label: string; top: number; height: number }
    | { kind: 'hour'; key: string; label: string; top: number; height: number }
    | {
          kind: 'shot'
          key: string
          screenshot: InstanceScreenshot
          top: number
          height: number
      }

const items = computed<TimelineItem[]>(() => {
    const grouped = new Map<string, InstanceScreenshot[]>()
    for (const screenshot of props.screenshots) {
        const key = dayjs(screenshot.created_at).format('YYYY-MM-DD')
        const bucket = grouped.get(key) ?? []
        bucket.push(screenshot)
        grouped.set(key, bucket)
    }

    const result: TimelineItem[] = []
    let top = 0
    for (const [id, shots] of grouped) {
        result.push({
            kind: 'day',
            key: `day-${id}`,
            label: formatMessage(messages.dayLabel, {
                date: formatFullDate(dayjs(shots[0].created_at).toDate()),
                count: shots.length,
            }),
            top,
            height: DAY_HEIGHT,
        })
        top += DAY_HEIGHT

        const useHours = shots.length > HOUR_SEPARATOR_THRESHOLD
        let currentHour = ''
        for (const screenshot of shots) {
            const moment = dayjs(screenshot.created_at)
            const hour = moment.format('HH')
            if (useHours && hour !== currentHour) {
                currentHour = hour
                result.push({
                    kind: 'hour',
                    key: `hour-${id}-${hour}`,
                    label: moment.startOf('hour').format('HH:00'),
                    top,
                    height: HOUR_HEIGHT,
                })
                top += HOUR_HEIGHT
            }
            top += ROW_HEIGHT
            result.push({
                kind: 'shot',
                key: `shot-${screenshot.id}`,
                screenshot,
                top: top - ROW_HEIGHT,
                height: ROW_HEIGHT,
            })
        }
        top += DAY_GAP
    }
    return result
})

const totalHeight = computed(() => {
    const list = items.value
    if (list.length === 0) return 0
    const last = list[list.length - 1]
    return last.top + last.height + DAY_GAP
})

const root = ref<HTMLElement>()
const scroller = ref<HTMLElement>()
const scrollTop = ref(0)
const viewportHeight = ref(800)
const availableHeight = ref(600)

let resizeObserver: ResizeObserver | undefined

function measure() {
    if (root.value) {
        const top = root.value.getBoundingClientRect().top
        availableHeight.value = Math.max(240, window.innerHeight - top - 32)
    }
    if (!scroller.value) return
    viewportHeight.value = scroller.value.clientHeight || 800
    scrollTop.value = scroller.value.scrollTop
}

onMounted(() => {
    measure()
    requestAnimationFrame(measure)
    setTimeout(measure, 400)
    window.addEventListener('resize', measure)
    if (typeof ResizeObserver !== 'undefined') {
        resizeObserver = new ResizeObserver(measure)
        if (root.value?.parentElement) resizeObserver.observe(root.value.parentElement)
        if (root.value) resizeObserver.observe(root.value)
    }
})

onBeforeUnmount(() => {
    window.removeEventListener('resize', measure)
    resizeObserver?.disconnect()
})

watch(
    () => props.screenshots,
    () => {
        scroller.value?.scrollTo({ top: 0 })
        scrollTop.value = 0
    },
)

function handleKeydown(event: KeyboardEvent) {
    if (!scroller.value) return
    const page = viewportHeight.value * 0.9
    if (event.key === 'PageDown') {
        event.preventDefault()
        scroller.value.scrollBy({ top: page })
    } else if (event.key === 'PageUp') {
        event.preventDefault()
        scroller.value.scrollBy({ top: -page })
    } else if (event.key === 'Home') {
        event.preventDefault()
        scroller.value.scrollTo({ top: 0 })
    } else if (event.key === 'End') {
        event.preventDefault()
        scroller.value.scrollTo({ top: totalHeight.value })
    }
}

watch(
    () => props.screenshots.length,
    () => {
        void nextTick(() => measure())
    },
)

const visibleItems = computed(() => {
    const start = Math.max(0, scrollTop.value - OVERSCAN)
    const end = scrollTop.value + viewportHeight.value + OVERSCAN
    return items.value.filter((item) => item.top + item.height >= start && item.top <= end)
})
</script>

<template>
    <div ref="root" class="flex min-h-0 w-full flex-1 flex-col">
        <p v-if="items.length === 0" class="m-0 text-secondary">
            {{ formatMessage(messages.empty) }}
        </p>
        <div
            v-else
            ref="scroller"
            tabindex="0"
            :style="{ height: `${availableHeight}px` }"
            class="min-h-0 flex-none overflow-y-auto pr-2 outline-none [overscroll-behavior:contain] [scrollbar-color:var(--color-surface-5)_transparent] [scrollbar-width:thin]"
            @scroll.passive="scrollTop = ($event.target as HTMLElement).scrollTop"
            @keydown="handleKeydown"
        >
            <div class="relative w-full" :style="{ height: `${totalHeight}px` }">
                <div
                    v-for="item in visibleItems"
                    :key="item.key"
                    class="absolute left-0 right-0"
                    :style="{ top: `${item.top}px` }"
                >
                    <h2
                        v-if="item.kind === 'day'"
                        class="m-0 flex items-center gap-2 rounded-xl bg-surface-2 px-4 py-2.5 text-base font-semibold text-[var(--color-text-default)]"
                    >
                        {{ item.label }}
                    </h2>
                    <div
                        v-else-if="item.kind === 'hour'"
                        class="flex items-center gap-2 pl-1 text-sm font-medium text-[var(--color-text-tertiary)]"
                    >
                        <span class="h-px w-4 bg-surface-5" aria-hidden="true" />
                        {{ item.label }}
                    </div>
                    <div v-else class="relative pl-5 pb-3">
                        <span
                            class="pointer-events-none absolute inset-y-0 left-[5px] w-px bg-surface-5"
                            aria-hidden="true"
                        />
                        <TimelineRow
                            :screenshot="item.screenshot"
                            :show-instance="showInstance"
                            :note="noteFor?.(item.screenshot)"
                            :editing="editingKey === item.key"
                            @activate="(shot, event) => emit('activate', shot, event)"
                            @contextmenu="(shot, event) => emit('contextmenu', shot, event)"
                            @save-note="(shot, value) => emit('save-note', shot, value)"
                            @cancel-note="emit('cancel-note')"
                        />
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
