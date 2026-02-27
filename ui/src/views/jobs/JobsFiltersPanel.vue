<script setup lang="ts">
import { NSelect, NSwitch } from 'naive-ui'
import { useI18n } from 'vue-i18n'

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
  <div v-if="layout === 'inline'" class="flex flex-wrap items-center gap-2">
    <div class="shrink-0 flex items-center gap-2 whitespace-nowrap h-7">
      <span class="text-sm app-text-muted">{{ t('jobs.showArchived') }}</span>
      <n-switch
        :value="showArchived"
        :aria-label="t('jobs.showArchived')"
        @update:value="(value: boolean) => emit('update:showArchived', value)"
      />
    </div>

    <div v-if="showStatus" class="shrink-0 flex items-center gap-2 whitespace-nowrap">
      <span class="text-sm app-text-muted">{{ t('runs.columns.status') }}</span>
      <n-select
        :value="latestStatusFilter"
        size="small"
        :aria-label="t('runs.columns.status')"
        :options="latestStatusOptions"
        :consistent-menu-width="false"
        class="min-w-[8rem]"
        @update:value="(value: JobLatestStatusFilter) => emit('update:latestStatusFilter', value)"
      />
    </div>

    <div v-if="showSchedule" class="shrink-0 flex items-center gap-2 whitespace-nowrap">
      <span class="text-sm app-text-muted">{{ t('jobs.columns.schedule') }}</span>
      <n-select
        :value="scheduleFilter"
        size="small"
        :aria-label="t('jobs.columns.schedule')"
        :options="scheduleOptions"
        :consistent-menu-width="false"
        class="min-w-[8rem]"
        @update:value="(value: JobScheduleFilter) => emit('update:scheduleFilter', value)"
      />
    </div>

    <div v-if="showSort" class="w-full md:w-56 md:flex-none">
      <n-select
        :value="sortKey"
        size="small"
        :aria-label="t('common.sort')"
        :options="sortOptions"
        @update:value="(value: JobSortKey) => emit('update:sortKey', value)"
      />
    </div>
  </div>

  <div v-else class="space-y-4">
    <div class="flex items-center justify-between gap-3">
      <span class="text-sm app-text-muted">{{ t('jobs.showArchived') }}</span>
      <n-switch
        :value="showArchived"
        :aria-label="t('jobs.showArchived')"
        @update:value="(value: boolean) => emit('update:showArchived', value)"
      />
    </div>

    <div v-if="showStatus" class="space-y-2">
      <div class="text-sm app-text-muted">{{ t('runs.columns.status') }}</div>
      <n-select
        :value="latestStatusFilter"
        size="small"
        :aria-label="t('runs.columns.status')"
        :options="latestStatusOptions"
        :consistent-menu-width="false"
        class="w-full"
        @update:value="(value: JobLatestStatusFilter) => emit('update:latestStatusFilter', value)"
      />
    </div>

    <div v-if="showSchedule" class="space-y-2">
      <div class="text-sm app-text-muted">{{ t('jobs.columns.schedule') }}</div>
      <n-select
        :value="scheduleFilter"
        size="small"
        :aria-label="t('jobs.columns.schedule')"
        :options="scheduleOptions"
        :consistent-menu-width="false"
        class="w-full"
        @update:value="(value: JobScheduleFilter) => emit('update:scheduleFilter', value)"
      />
    </div>

    <div v-if="showSort" class="space-y-2">
      <div class="text-sm app-text-muted">{{ t('common.sort') }}</div>
      <n-select
        :value="sortKey"
        size="small"
        :aria-label="t('common.sort')"
        :options="sortOptions"
        class="w-full"
        @update:value="(value: JobSortKey) => emit('update:sortKey', value)"
      />
    </div>
  </div>
</template>
