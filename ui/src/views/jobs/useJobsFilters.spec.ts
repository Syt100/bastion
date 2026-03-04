import { describe, expect, it } from 'vitest'

import { useJobsFilters } from './useJobsFilters'

describe('useJobsFilters route hydration', () => {
  const t = (key: string) => key

  it('hydrates filter refs from route query with shared parsing fallbacks', () => {
    const filters = useJobsFilters(t)

    filters.applyRouteQuery({
      q: '  mysql  ',
      archived: 'true',
      status: 'failed',
      schedule: 'scheduled',
      sort: 'name_asc',
    })

    expect(filters.searchText.value).toBe('mysql')
    expect(filters.showArchived.value).toBe(true)
    expect(filters.latestStatusFilter.value).toBe('failed')
    expect(filters.scheduleFilter.value).toBe('scheduled')
    expect(filters.sortKey.value).toBe('name_asc')

    filters.applyRouteQuery({ status: 'invalid', archived: 'no', sort: 'bad' })
    expect(filters.latestStatusFilter.value).toBe('all')
    expect(filters.showArchived.value).toBe(false)
    expect(filters.sortKey.value).toBe('updated_desc')
  })
})
