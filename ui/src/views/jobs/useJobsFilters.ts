import { computed, ref } from 'vue'

import {
  createSingleSelectFilterField,
  createTextFilterField,
  parseRouteQueryBoolean,
  parseRouteQueryEnum,
  parseRouteQueryFirst,
  useListFilters,
} from '@/lib/listFilters'
import { runStatusLabel } from '@/lib/runs'
import type { RunStatus } from '@/stores/jobs'

export type JobSortKey = 'updated_desc' | 'updated_asc' | 'name_asc' | 'name_desc'
export type JobLatestStatusFilter = RunStatus | 'never' | 'all'
export type JobScheduleFilter = 'all' | 'manual' | 'scheduled'

type Translate = (key: string, params?: Record<string, unknown>) => string

export function useJobsFilters(t: Translate) {
  const showArchived = ref<boolean>(false)
  const searchText = ref<string>('')
  const sortKey = ref<JobSortKey>('updated_desc')
  const latestStatusFilter = ref<JobLatestStatusFilter>('all')
  const scheduleFilter = ref<JobScheduleFilter>('all')
  const statusValues = ['all', 'never', 'success', 'failed', 'running', 'queued', 'rejected', 'canceled'] as const
  const scheduleValues = ['all', 'manual', 'scheduled'] as const
  const sortValues = ['updated_desc', 'updated_asc', 'name_asc', 'name_desc'] as const

  const sortOptions = computed<Array<{ label: string; value: JobSortKey }>>(() => [
    { label: t('jobs.sort.updatedDesc'), value: 'updated_desc' },
    { label: t('jobs.sort.updatedAsc'), value: 'updated_asc' },
    { label: t('jobs.sort.nameAsc'), value: 'name_asc' },
    { label: t('jobs.sort.nameDesc'), value: 'name_desc' },
  ])

  const latestStatusFilterOptions = computed<Array<{ label: string; value: JobLatestStatusFilter }>>(() => [
    { label: t('runs.filters.all'), value: 'all' },
    { label: t('runs.neverRan'), value: 'never' },
    { label: runStatusLabel(t, 'success'), value: 'success' },
    { label: runStatusLabel(t, 'failed'), value: 'failed' },
    { label: runStatusLabel(t, 'running'), value: 'running' },
    { label: runStatusLabel(t, 'queued'), value: 'queued' },
    { label: runStatusLabel(t, 'rejected'), value: 'rejected' },
    { label: runStatusLabel(t, 'canceled'), value: 'canceled' },
  ])

  const scheduleFilterOptions = computed<Array<{ label: string; value: JobScheduleFilter }>>(() => [
    { label: t('runs.filters.all'), value: 'all' },
    { label: t('jobs.scheduleMode.manual'), value: 'manual' },
    { label: t('jobs.workspace.filters.scheduled'), value: 'scheduled' },
  ])

  const {
    filtersActiveCount,
    activeFilterChips,
    hasActiveFilters,
    clearFilters,
  } = useListFilters([
    createTextFilterField({
      key: 'q',
      label: t('common.search'),
      value: searchText,
    }),
    {
      clear: () => {
        showArchived.value = false
      },
      isActive: () => showArchived.value,
      chips: () =>
        showArchived.value
          ? [
              {
                key: 'archived',
                label: t('jobs.showArchived'),
                onClose: () => {
                  showArchived.value = false
                },
              },
            ]
          : [],
    },
    createSingleSelectFilterField({
      key: 'status',
      label: t('runs.columns.status'),
      value: latestStatusFilter,
      defaultValue: 'all',
      options: () => latestStatusFilterOptions.value,
    }),
    createSingleSelectFilterField({
      key: 'schedule',
      label: t('jobs.columns.schedule'),
      value: scheduleFilter,
      defaultValue: 'all',
      options: () => scheduleFilterOptions.value,
    }),
    createSingleSelectFilterField({
      key: 'sort',
      label: t('common.sort'),
      value: sortKey,
      defaultValue: 'updated_desc',
      options: () => sortOptions.value,
    }),
  ])

  function applyRouteQuery(query: Record<string, unknown>): void {
    showArchived.value = parseRouteQueryBoolean(query.archived, false)
    searchText.value = parseRouteQueryFirst(query.q) ?? ''
    latestStatusFilter.value = parseRouteQueryEnum(query.status, statusValues, 'all')
    scheduleFilter.value = parseRouteQueryEnum(query.schedule, scheduleValues, 'all')
    sortKey.value = parseRouteQueryEnum(query.sort, sortValues, 'updated_desc')
  }

  return {
    showArchived,
    searchText,
    sortKey,
    latestStatusFilter,
    scheduleFilter,
    sortOptions,
    latestStatusFilterOptions,
    scheduleFilterOptions,
    filtersActiveCount,
    activeFilterChips,
    hasActiveFilters,
    clearFilters,
    applyRouteQuery,
  }
}
