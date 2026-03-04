import { describe, expect, it } from 'vitest'

import { buildListRangeSummary, DEFAULT_LIST_PAGE_SIZE, LIST_PAGE_SIZE_OPTIONS } from './listUi'

describe('list pagination defaults', () => {
  it('exports shared default page size options', () => {
    expect(LIST_PAGE_SIZE_OPTIONS).toEqual([20, 50, 100])
    expect(DEFAULT_LIST_PAGE_SIZE).toBe(20)
  })
})

describe('buildListRangeSummary', () => {
  it('returns zero range for empty lists', () => {
    expect(buildListRangeSummary(0, 1, 20)).toEqual({ start: 0, end: 0, total: 0 })
  })

  it('returns start/end for non-empty pages', () => {
    expect(buildListRangeSummary(125, 1, 20)).toEqual({ start: 1, end: 20, total: 125 })
    expect(buildListRangeSummary(125, 3, 20)).toEqual({ start: 41, end: 60, total: 125 })
  })

  it('caps end and start when page exceeds remaining items', () => {
    expect(buildListRangeSummary(41, 3, 20)).toEqual({ start: 41, end: 41, total: 41 })
  })
})
