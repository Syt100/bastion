<script setup lang="ts">
import { NSelect, NSwitch } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import ListFilterField from '@/components/list/ListFilterField.vue'

import type { JobLatestStatusFilter, JobScheduleFilter, JobSortKey } from './useJobsFilters'

type FilterOption<T> = {
  label: string
  value: T
}

withDefaults(
  defineProps<{
    layout?: 'stack' | 'inline'
    showArchived: boolean
    latestStatusFilter: JobLatestStatusFilter
    scheduleFilter: JobScheduleFilter
    sortKey: JobSortKey
    latestStatusOptions: FilterOption<JobLatestStatusFilter>[]
    scheduleOptions: FilterOption<JobScheduleFilter>[]
    sortOptions: FilterOption<JobSortKey>[]
    showStatus?: boolean
    showSchedule?: boolean
    showSort?: boolean
  }>(),
  {
    layout: 'stack',
    showStatus: true,
    showSchedule: true,
    showSort: true,
  },
)

const emit = defineEmits<{
  'update:showArchived': [value: boolean]
  'update:latestStatusFilter': [value: JobLatestStatusFilter]
  'update:scheduleFilter': [value: JobScheduleFilter]
  'update:sortKey': [value: JobSortKey]
}>()

const { t } = useI18n()
</script>

<template>
  <div v-if="layout === 'inline'" class="flex flex-wrap items-center gap-2.5">
    <div class="shrink-0 flex items-center gap-2 whitespace-nowrap">
      <span class="app-filter-label">{{ t('jobs.showArchived') }}</span>
      <n-switch
        :value="showArchived"
        :aria-label="t('jobs.showArchived')"
        @update:value="(value: boolean) => emit('update:showArchived', value)"
      />
    </div>

    <ListFilterField
      v-if="showStatus"
      :label="t('runs.columns.status')"
      layout="inline"
    >
      <n-select
        :value="latestStatusFilter"
        size="small"
        :aria-label="t('runs.columns.status')"
        :options="latestStatusOptions"
        :consistent-menu-width="false"
        class="w-full"
        @update:value="(value: JobLatestStatusFilter) => emit('update:latestStatusFilter', value)"
      />
    </ListFilterField>

    <ListFilterField
      v-if="showSchedule"
      :label="t('jobs.columns.schedule')"
      layout="inline"
    >
      <n-select
        :value="scheduleFilter"
        size="small"
        :aria-label="t('jobs.columns.schedule')"
        :options="scheduleOptions"
        :consistent-menu-width="false"
        class="w-full"
        @update:value="(value: JobScheduleFilter) => emit('update:scheduleFilter', value)"
      />
    </ListFilterField>

    <ListFilterField
      v-if="showSort"
      :label="t('common.sort')"
      layout="inline"
      control-width="full"
    >
      <n-select
        :value="sortKey"
        size="small"
        :aria-label="t('common.sort')"
        :options="sortOptions"
        class="w-full"
        @update:value="(value: JobSortKey) => emit('update:sortKey', value)"
      />
    </ListFilterField>
  </div>

  <div v-else class="space-y-4">
    <div class="flex items-center justify-between gap-3">
      <span class="app-filter-label">{{ t('jobs.showArchived') }}</span>
      <n-switch
        :value="showArchived"
        :aria-label="t('jobs.showArchived')"
        @update:value="(value: boolean) => emit('update:showArchived', value)"
      />
    </div>

    <ListFilterField
      v-if="showStatus"
      :label="t('runs.columns.status')"
      layout="stack"
    >
      <n-select
        :value="latestStatusFilter"
        size="small"
        :aria-label="t('runs.columns.status')"
        :options="latestStatusOptions"
        :consistent-menu-width="false"
        class="w-full"
        @update:value="(value: JobLatestStatusFilter) => emit('update:latestStatusFilter', value)"
      />
    </ListFilterField>

    <ListFilterField
      v-if="showSchedule"
      :label="t('jobs.columns.schedule')"
      layout="stack"
    >
      <n-select
        :value="scheduleFilter"
        size="small"
        :aria-label="t('jobs.columns.schedule')"
        :options="scheduleOptions"
        :consistent-menu-width="false"
        class="w-full"
        @update:value="(value: JobScheduleFilter) => emit('update:scheduleFilter', value)"
      />
    </ListFilterField>

    <ListFilterField
      v-if="showSort"
      :label="t('common.sort')"
      layout="stack"
    >
      <n-select
        :value="sortKey"
        size="small"
        :aria-label="t('common.sort')"
        :options="sortOptions"
        class="w-full"
        @update:value="(value: JobSortKey) => emit('update:sortKey', value)"
      />
    </ListFilterField>
  </div>
</template>
