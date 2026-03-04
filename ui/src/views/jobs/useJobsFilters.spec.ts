import { describe, expect, it } from 'vitest'

import { useJobsFilters } from './useJobsFilters'

const i18nMap: Record<string, string> = {
  'common.search': 'Search',
  'jobs.showArchived': 'Show archived',
  'runs.columns.status': 'Status',
  'jobs.columns.schedule': 'Schedule',
  'common.sort': 'Sort',
  'jobs.sort.updatedDesc': 'Updated ↓',
  'jobs.sort.updatedAsc': 'Updated ↑',
  'jobs.sort.nameAsc': 'Name A-Z',
  'jobs.sort.nameDesc': 'Name Z-A',
  'runs.filters.all': 'All',
  'runs.neverRan': 'Never',
  'jobs.scheduleMode.manual': 'Manual',
  'jobs.workspace.filters.scheduled': 'Scheduled',
  'runs.statuses.success': 'Success',
  'runs.statuses.failed': 'Failed',
  'runs.statuses.running': 'Running',
  'runs.statuses.queued': 'Queued',
  'runs.statuses.rejected': 'Rejected',
  'runs.statuses.canceled': 'Canceled',
}

function t(key: string): string {
  return i18nMap[key] ?? key
}

describe('useJobsFilters', () => {
  it('derives chips/count through shared list filter model and clears defaults', () => {
    const filters = useJobsFilters(t)

    filters.searchText.value = '  edge  '
    filters.showArchived.value = true
    filters.latestStatusFilter.value = 'failed'
    filters.scheduleFilter.value = 'scheduled'
    filters.sortKey.value = 'name_asc'

    expect(filters.filtersActiveCount.value).toBe(5)
    expect(filters.activeFilterChips.value.map((chip) => chip.label)).toEqual([
      'Search: edge',
      'Show archived',
      'Status: Failed',
      'Schedule: Scheduled',
      'Sort: Name A-Z',
    ])
    expect(filters.hasActiveFilters.value).toBe(true)

    filters.clearFilters()
    expect(filters.searchText.value).toBe('')
    expect(filters.showArchived.value).toBe(false)
    expect(filters.latestStatusFilter.value).toBe('all')
    expect(filters.scheduleFilter.value).toBe('all')
    expect(filters.sortKey.value).toBe('updated_desc')
    expect(filters.hasActiveFilters.value).toBe(false)
  })
})
