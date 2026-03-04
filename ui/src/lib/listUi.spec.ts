import { describe, expect, it } from 'vitest'

import { buildListRangeSummary } from './listUi'

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
