import { computed, ref } from 'vue'
import { describe, expect, it } from 'vitest'

import {
  createMultiSelectFilterField,
  createSingleSelectFilterField,
  createTextFilterField,
  parseRouteQueryBoolean,
  parseRouteQueryEnum,
  parseRouteQueryFirst,
  parseRouteQueryList,
  useListFilters,
} from './listFilters'

describe('useListFilters', () => {
  it('derives text/single filter chips and clears defaults', () => {
    const searchText = ref('  edge-01  ')
    const status = ref<'all' | 'online' | 'offline'>('offline')

    const statusOptions = computed(() => [
      { label: 'All', value: 'all' as const },
      { label: 'Online', value: 'online' as const },
      { label: 'Offline', value: 'offline' as const },
    ])

    const filters = useListFilters([
      createTextFilterField({ key: 'q', label: 'Search', value: searchText }),
      createSingleSelectFilterField({
        key: 'status',
        label: 'Status',
        value: status,
        defaultValue: 'all',
        options: () => statusOptions.value,
      }),
    ])

    expect(filters.hasActiveFilters.value).toBe(true)
    expect(filters.filtersActiveCount.value).toBe(2)
    expect(filters.activeFilterChips.value.map((chip) => chip.label)).toEqual([
      'Search: edge-01',
      'Status: Offline',
    ])

    filters.clearFilters()
    expect(searchText.value).toBe('')
    expect(status.value).toBe('all')
    expect(filters.hasActiveFilters.value).toBe(false)
  })

  it('supports per-value close for multi-select filters while active count stays field-based', () => {
    const labels = ref<string[] | null>(['edge', 'db'])

    const filters = useListFilters([
      createMultiSelectFilterField({
        key: 'labels',
        label: 'Label',
        value: labels,
        options: () => [
          { label: 'edge', value: 'edge' },
          { label: 'db', value: 'db' },
        ],
      }),
    ])

    expect(filters.filtersActiveCount.value).toBe(1)
    expect(filters.activeFilterChips.value.map((chip) => chip.label)).toEqual([
      'Label: edge',
      'Label: db',
    ])

    filters.activeFilterChips.value[0]?.onClose()
    expect(labels.value).toEqual(['db'])

    filters.clearFilters()
    expect(labels.value).toEqual([])
  })
})

describe('parseRouteQueryList', () => {
  it('parses comma-separated query values from string/array forms', () => {
    expect(parseRouteQueryList('failed, queued')).toEqual(['failed', 'queued'])
    expect(parseRouteQueryList(['failed,queued', ' sent '])).toEqual(['failed', 'queued', 'sent'])
    expect(parseRouteQueryList(null)).toEqual([])
  })
})

describe('route query parsing helpers', () => {
  it('extracts the first normalized route query value', () => {
    expect(parseRouteQueryFirst(' offline ')).toBe('offline')
    expect(parseRouteQueryFirst(['', 'revoked'])).toBe('revoked')
    expect(parseRouteQueryFirst(['a,b', 'c'])).toBe('a')
    expect(parseRouteQueryFirst(undefined)).toBeNull()
  })

  it('parses enum values with fallback for invalid query values', () => {
    const allowed = ['all', 'online', 'offline', 'revoked'] as const
    expect(parseRouteQueryEnum('online', allowed, 'all')).toBe('online')
    expect(parseRouteQueryEnum(['foo,offline'], allowed, 'all')).toBe('offline')
    expect(parseRouteQueryEnum('unknown', allowed, 'all')).toBe('all')
    expect(parseRouteQueryEnum(null, allowed, 'all')).toBe('all')
  })

  it('parses boolean route query values', () => {
    expect(parseRouteQueryBoolean('true')).toBe(true)
    expect(parseRouteQueryBoolean('1')).toBe(true)
    expect(parseRouteQueryBoolean('off', true)).toBe(false)
    expect(parseRouteQueryBoolean('unknown', true)).toBe(true)
    expect(parseRouteQueryBoolean(undefined)).toBe(false)
  })
})
