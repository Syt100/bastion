import { describe, expect, it } from 'vitest'

import { parseRunSummary } from './run_summary'

describe('parseRunSummary', () => {
  it('parses consistencyChangedTotal from filesystem summary', () => {
    const parsed = parseRunSummary({
      filesystem: {
        consistency: {
          v: 2,
          changed_total: 1,
          replaced_total: 2,
          deleted_total: 3,
          read_error_total: 4,
          sample_truncated: false,
          sample: [{ path: 'a.txt', reason: 'mtime_changed' }],
        },
      },
    })

    expect(parsed.consistencyChangedTotal).toBe(10)
    expect(parsed.consistency).toEqual({
      v: 2,
      changedTotal: 1,
      replacedTotal: 2,
      deletedTotal: 3,
      readErrorTotal: 4,
      total: 10,
      sampleTruncated: false,
      sample: [
        {
          path: 'a.txt',
          reason: 'mtime_changed',
          error: null,
          before: null,
          afterHandle: null,
          afterPath: null,
        },
      ],
    })
  })

  it('parses consistencyChangedTotal from vaultwarden summary', () => {
    const parsed = parseRunSummary({
      vaultwarden: {
        data_dir: '/vw',
        db: 'db.sqlite3',
        consistency: {
          v: 2,
          changed_total: 0,
          replaced_total: 1,
          deleted_total: 0,
          read_error_total: 0,
          sample_truncated: false,
          sample: [
            {
              path: 'db.sqlite3',
              reason: 'size_changed',
              before: { size_bytes: 10, mtime_unix_nanos: 1 },
              after_handle: { size_bytes: 10, mtime_unix_nanos: 1 },
              after_path: { size_bytes: 11, mtime_unix_nanos: 2 },
            },
          ],
        },
      },
    })

    expect(parsed.consistencyChangedTotal).toBe(1)
    expect(parsed.consistency?.sample[0]).toEqual({
      path: 'db.sqlite3',
      reason: 'size_changed',
      error: null,
      before: { size_bytes: 10, mtime_unix_nanos: 1 },
      afterHandle: { size_bytes: 10, mtime_unix_nanos: 1 },
      afterPath: { size_bytes: 11, mtime_unix_nanos: 2 },
    })
  })

  it('returns null when consistency is absent', () => {
    const parsed = parseRunSummary({ filesystem: { warnings_total: 0, errors_total: 0 } })
    expect(parsed.consistencyChangedTotal).toBe(null)
  })

  it('parses filesystemSnapshot when snapshot mode is enabled', () => {
    const parsed = parseRunSummary({
      filesystem: {
        snapshot: { mode: 'auto', status: 'ready', provider: 'btrfs' },
      },
    })
    expect(parsed.filesystemSnapshot).toEqual({
      mode: 'auto',
      status: 'ready',
      provider: 'btrfs',
      reason: null,
    })
  })

  it('hides filesystemSnapshot when snapshot mode is off', () => {
    const parsed = parseRunSummary({
      filesystem: {
        snapshot: { mode: 'off', status: 'off' },
      },
    })
    expect(parsed.filesystemSnapshot).toBe(null)
  })
})
