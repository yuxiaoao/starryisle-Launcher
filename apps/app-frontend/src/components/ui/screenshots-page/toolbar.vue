<script setup lang="ts">
import {
    ArrowUpDownIcon,
    GridIcon,
    ImportIcon,
    PlusIcon,
    SearchIcon,
} from '@modrinth/assets'
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
const groupFilter = defineModel<string>('groupFilter', { required: false, default: 'all' })
const view = defineModel<string>('view', { required: false, default: 'grid' })

defineProps<{
    sortOptions: ComboboxOption<string>[]
    groupOptions: ComboboxOption<string>[]
    showInstanceFilter?: boolean
    instanceOptions?: ComboboxOption<string>[]
    showGroup?: boolean
    showViewToggle?: boolean
    viewOptions?: ComboboxOption<string>[]
    showGroupFilter?: boolean
    groupFilterOptions?: ComboboxOption<string>[]
    showImportAction?: boolean
    onImportBundle?: () => void
}>()

const emit = defineEmits<{
    (e: 'new-group'): void
    (e: 'import-bundle'): void
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
    search: { id: 'app.screenshots.search', defaultMessage: 'Search' },
    newGroup: { id: 'app.screenshots.group.new', defaultMessage: 'New group' },
    sortBy: { id: 'app.screenshots.sort-by', defaultMessage: 'Sort by' },
    groupBy: { id: 'app.screenshots.group-by', defaultMessage: 'Group by' },
    instanceFilter: { id: 'app.screenshots.filter.instance', defaultMessage: 'Filter by instance' },
    groupFilter: { id: 'app.screenshots.filter.group', defaultMessage: 'Filter by group' },
    viewToggle: { id: 'app.screenshots.view', defaultMessage: 'View' },
    importBundle: { id: 'app.screenshots.bundle.import-toolbar', defaultMessage: 'Import bundle' },
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
                v-if="showViewToggle"
                v-model="view"
                width="content"
                :options="viewOptions ?? []"
                :show-icon-in-selected="false"
                dropdown-min-width="160px"
            >
                <template #prefix>
                    <GridIcon
                        class="size-5 text-[var(--color-text-default)]"
                        :aria-label="formatMessage(messages.viewToggle)"
                    />
                </template>
                <template #selected="{ label }">
                    <span>{{ label }}</span>
                </template>
            </Combobox>
            <Combobox
                v-if="showGroup"
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
            <Button
                v-if="showImportAction"
                v-tooltip="formatMessage(messages.importBundle)"
                type="base"
                @click="onImportBundle ? onImportBundle() : emit('import-bundle')"
            >
                <ImportIcon />
                <span class="text-[var(--color-text-default)]">{{ formatMessage(messages.importBundle) }}</span>
            </Button>
        </div>
    </div>
</template>
