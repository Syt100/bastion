import { computed, ref } from 'vue'

import type { PickerActiveChip } from '@/components/pickers/PickerActiveChipsRow.vue'
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

  const filtersActiveCount = computed(() => {
    let n = 0
    if (searchText.value.trim().length > 0) n += 1
    if (showArchived.value) n += 1
    if (latestStatusFilter.value !== 'all') n += 1
    if (scheduleFilter.value !== 'all') n += 1
    if (sortKey.value !== 'updated_desc') n += 1
    return n
  })

  const activeFilterChips = computed<PickerActiveChip[]>(() => {
    const chips: PickerActiveChip[] = []

    const q = searchText.value.trim()
    if (q.length > 0) {
      chips.push({
        key: 'q',
        label: `${t('common.search')}: ${q}`,
        onClose: () => {
          searchText.value = ''
        },
      })
    }

    if (showArchived.value) {
      chips.push({
        key: 'archived',
        label: t('jobs.showArchived'),
        onClose: () => {
          showArchived.value = false
        },
      })
    }

    if (latestStatusFilter.value !== 'all') {
      const label =
        latestStatusFilterOptions.value.find((o) => o.value === latestStatusFilter.value)?.label ??
        String(latestStatusFilter.value)
      chips.push({
        key: 'status',
        label: `${t('runs.columns.status')}: ${label}`,
        onClose: () => {
          latestStatusFilter.value = 'all'
        },
      })
    }

    if (scheduleFilter.value !== 'all') {
      const label =
        scheduleFilterOptions.value.find((o) => o.value === scheduleFilter.value)?.label ??
        String(scheduleFilter.value)
      chips.push({
        key: 'schedule',
        label: `${t('jobs.columns.schedule')}: ${label}`,
        onClose: () => {
          scheduleFilter.value = 'all'
        },
      })
    }

    if (sortKey.value !== 'updated_desc') {
      const label = sortOptions.value.find((o) => o.value === sortKey.value)?.label ?? String(sortKey.value)
      chips.push({
        key: 'sort',
        label: `${t('common.sort')}: ${label}`,
        onClose: () => {
          sortKey.value = 'updated_desc'
        },
      })
    }

    return chips
  })

  const hasActiveFilters = computed<boolean>(() => {
    return (
      showArchived.value ||
      searchText.value.trim().length > 0 ||
      latestStatusFilter.value !== 'all' ||
      scheduleFilter.value !== 'all' ||
      sortKey.value !== 'updated_desc'
    )
  })

  function clearFilters(): void {
    searchText.value = ''
    showArchived.value = false
    sortKey.value = 'updated_desc'
    latestStatusFilter.value = 'all'
    scheduleFilter.value = 'all'
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
  }
}
