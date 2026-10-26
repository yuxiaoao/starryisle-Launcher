<script setup lang="ts">
import { ArrowUpDownIcon, GridIcon, PlusIcon, SearchIcon } from '@modrinth/assets'
import {
    Button,
    Combobox,
    type ComboboxOption,
    defineMessages,
    StyledInput,
    useVIntl,
} from '@modrinth/ui'

const search = defineModel<string>('search', { required: true })
const sort = defineModel<string>('sort', { required: true })
const group = defineModel<string>('group', { required: true })
const instance = defineModel<string>('instance', { required: false, default: 'all' })
const range = defineModel<string>('range', { required: false, default: 'all' })
const groupFilter = defineModel<string>('groupFilter', { required: false, default: 'all' })

defineProps<{
    sortOptions: ComboboxOption<string>[]
    groupOptions: ComboboxOption<string>[]
    showInstanceFilter?: boolean
    instanceOptions?: ComboboxOption<string>[]
    showRangeFilter?: boolean
    rangeOptions?: ComboboxOption<string>[]
    showGroupFilter?: boolean
    groupFilterOptions?: ComboboxOption<string>[]
}>()

const emit = defineEmits<{
    (e: 'new-group'): void
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
    search: { id: 'app.screenshots.search', defaultMessage: 'Search' },
    newGroup: { id: 'app.screenshots.group.new', defaultMessage: 'New group' },
    sortBy: { id: 'app.screenshots.sort-by', defaultMessage: 'Sort by' },
    groupBy: { id: 'app.screenshots.group-by', defaultMessage: 'Group by' },
    instanceFilter: { id: 'app.screenshots.filter.instance', defaultMessage: 'Filter by instance' },
    rangeFilter: { id: 'app.screenshots.filter.range', defaultMessage: 'Filter by time' },
    groupFilter: { id: 'app.screenshots.filter.group', defaultMessage: 'Filter by group' },
})
</script>

<template>
    <div class="flex flex-col gap-2">
        <div class="flex flex-wrap gap-2">
            <StyledInput
                v-model="search"
                :icon="SearchIcon"
                type="text"
                :placeholder="formatMessage(messages.search)"
                clearable
                wrapper-class="min-w-[16rem] flex-1"
            />
            <Button @click="emit('new-group')">
                <PlusIcon />
                {{ formatMessage(messages.newGroup) }}
            </Button>
        </div>
        <div class="flex flex-wrap items-center gap-2">
            <Combobox
                v-model="sort"
                width="content"
                :options="sortOptions"
                :show-icon-in-selected="false"
                dropdown-min-width="160px"
            >
                <template #prefix>
                    <ArrowUpDownIcon
                        class="size-5 text-[var(--color-text-default)]"
                        :aria-label="formatMessage(messages.sortBy)"
                    />
                </template>
                <template #selected="{ label }">
                    <span>{{ label }}</span>
                </template>
            </Combobox>
            <Combobox
                v-model="group"
                width="content"
                :options="groupOptions"
                :show-icon-in-selected="false"
                dropdown-min-width="160px"
            >
                <template #prefix>
                    <GridIcon
                        class="size-5 text-[var(--color-text-default)]"
                        :aria-label="formatMessage(messages.groupBy)"
                    />
                </template>
                <template #selected="{ label }">
                    <span>{{ label }}</span>
                </template>
            </Combobox>
            <Combobox
                v-if="showInstanceFilter"
                v-model="instance"
                width="content"
                :options="instanceOptions ?? []"
                :show-icon-in-selected="false"
                dropdown-min-width="220px"
            >
                <template #prefix>
                    <GridIcon
                        class="size-5 text-[var(--color-text-default)]"
                        :aria-label="formatMessage(messages.instanceFilter)"
                    />
                </template>
                <template #selected="{ label }">
                    <span>{{ label }}</span>
                </template>
            </Combobox>
            <Combobox
                v-if="showRangeFilter"
                v-model="range"
                width="content"
                :options="rangeOptions ?? []"
                :show-icon-in-selected="false"
                dropdown-min-width="180px"
            >
                <template #prefix>
                    <GridIcon
                        class="size-5 text-[var(--color-text-default)]"
                        :aria-label="formatMessage(messages.rangeFilter)"
                    />
                </template>
                <template #selected="{ label }">
                    <span>{{ label }}</span>
                </template>
            </Combobox>
            <Combobox
                v-if="showGroupFilter"
                v-model="groupFilter"
                width="content"
                :options="groupFilterOptions ?? []"
                :show-icon-in-selected="false"
                dropdown-min-width="200px"
            >
                <template #prefix>
                    <GridIcon
                        class="size-5 text-[var(--color-text-default)]"
                        :aria-label="formatMessage(messages.groupFilter)"
                    />
                </template>
                <template #selected="{ label }">
                    <span>{{ label }}</span>
                </template>
            </Combobox>
        </div>
    </div>
</template>
