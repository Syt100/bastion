import { describe, expect, it } from 'vitest'

import { filterRunEvents, findFirstEventSeq, uniqueRunEventKinds } from './run_events'

describe('run_events helpers', () => {
  const base = [
    { run_id: 'r1', seq: 1, ts: 1, level: 'info', kind: 'scan', message: 'scan', fields: null },
    { run_id: 'r1', seq: 2, ts: 2, level: 'warn', kind: 'fs_issues', message: 'filesystem issues', fields: null },
    { run_id: 'r1', seq: 3, ts: 3, level: 'error', kind: 'upload', message: 'MKCOL failed', fields: null },
    { run_id: 'r1', seq: 4, ts: 4, level: 'info', kind: 'complete', message: 'complete', fields: null },
  ]

  it('filters by level/kind/query', () => {
    expect(filterRunEvents(base, { level: 'error' }).map((e) => e.seq)).toEqual([3])
    expect(filterRunEvents(base, { kind: 'upload' }).map((e) => e.seq)).toEqual([3])
    expect(filterRunEvents(base, { query: 'mkcol' }).map((e) => e.seq)).toEqual([3])
    expect(filterRunEvents(base, { query: 'issues' }).map((e) => e.seq)).toEqual([2])
  })

  it('computes unique kinds', () => {
    expect(uniqueRunEventKinds(base)).toEqual(['complete', 'fs_issues', 'scan', 'upload'])
  })

  it('finds first matching event seq', () => {
    expect(findFirstEventSeq(base, (e) => e.level === 'error')).toBe(3)
    expect(findFirstEventSeq(base, (e) => e.kind === 'missing')).toBeNull()
  })
})

