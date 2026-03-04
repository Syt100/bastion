import { describe, expect, it } from 'vitest'

import {
  appendListFilterParams,
  appendPaginationParams,
  appendQueryArrayParam,
  appendQueryBooleanParam,
  appendQueryNumberParam,
  appendQueryTextParam,
  buildQuerySuffix,
  sizeUnitMultiplier,
} from './listQuery'

describe('appendListFilterParams', () => {
  it('serializes common list filter params with default size keys', () => {
    const params = new URLSearchParams()
    appendListFilterParams(params, {
      q: '  alpha  ',
      kind: 'file',
      hideDotfiles: true,
      typeSort: 'dir_first',
      sortBy: 'name',
      sortDir: 'asc',
      size: { min: 1.5, max: 3.1, unit: 'KB' },
    })

    expect(params.get('q')).toBe('alpha')
    expect(params.get('kind')).toBe('file')
    expect(params.get('hide_dotfiles')).toBe('true')
    expect(params.get('type_sort')).toBe('dir_first')
    expect(params.get('sort_by')).toBe('name')
    expect(params.get('sort_dir')).toBe('asc')
    expect(params.get('size_min_bytes')).toBe(String(Math.floor(1.5 * 1024)))
    expect(params.get('size_max_bytes')).toBe(String(Math.floor(3.1 * 1024)))
  })

  it('skips empty/default values and supports custom size keys', () => {
    const params = new URLSearchParams()
    appendListFilterParams(params, {
      q: '   ',
      kind: 'all',
      hideDotfiles: false,
      typeSort: 'dir_first',
      typeSortDefault: 'dir_first',
      size: { min: null, max: 2, unit: 'MB' },
      sizeMinKey: 'min_size_bytes',
      sizeMaxKey: 'max_size_bytes',
    })

    expect(params.has('q')).toBe(false)
    expect(params.has('kind')).toBe(false)
    expect(params.has('hide_dotfiles')).toBe(false)
    expect(params.has('type_sort')).toBe(false)
    expect(params.has('min_size_bytes')).toBe(false)
    expect(params.get('max_size_bytes')).toBe(String(2 * 1024 * 1024))
  })
})

describe('sizeUnitMultiplier', () => {
  it('returns expected multipliers including fallback', () => {
    expect(sizeUnitMultiplier('B')).toBe(1)
    expect(sizeUnitMultiplier('KB')).toBe(1024)
    expect(sizeUnitMultiplier('MB')).toBe(1024 * 1024)
    expect(sizeUnitMultiplier('GB')).toBe(1024 * 1024 * 1024)
    expect(sizeUnitMultiplier('TB')).toBe(1)
  })
})

describe('generic list query helpers', () => {
  it('serializes text/array/boolean/number params and pagination', () => {
    const params = new URLSearchParams()
    appendQueryTextParam(params, 'q', '  hello  ')
    appendQueryTextParam(params, 'skip', '   ')
    appendQueryArrayParam(params, 'labels[]', [' edge ', '', 'db'])
    appendQueryBooleanParam(params, 'include_archived', true)
    appendQueryNumberParam(params, 'cursor', 12.8)
    appendPaginationParams(params, { page: 2, pageSize: 50 })

    expect(params.get('q')).toBe('hello')
    expect(params.has('skip')).toBe(false)
    expect(params.getAll('labels[]')).toEqual(['edge', 'db'])
    expect(params.get('include_archived')).toBe('true')
    expect(params.get('cursor')).toBe('12')
    expect(params.get('page')).toBe('2')
    expect(params.get('page_size')).toBe('50')
    expect(buildQuerySuffix(params)).toContain('?')
  })

  it('returns empty suffix when query has no params', () => {
    expect(buildQuerySuffix(new URLSearchParams())).toBe('')
  })
})
