import { describe, expect, it } from 'vitest'

import { parseRunSummary } from './run_summary'

describe('parseRunSummary', () => {
  it('parses consistencyChangedTotal from filesystem summary', () => {
    const parsed = parseRunSummary({
      filesystem: {
        consistency: {
          v: 1,
          changed_total: 1,
          replaced_total: 2,
          deleted_total: 3,
          read_error_total: 4,
          sample_truncated: false,
          sample: [],
        },
      },
    })

    expect(parsed.consistencyChangedTotal).toBe(10)
  })

  it('parses consistencyChangedTotal from vaultwarden summary', () => {
    const parsed = parseRunSummary({
      vaultwarden: {
        data_dir: '/vw',
        db: 'db.sqlite3',
        consistency: {
          v: 1,
          changed_total: 0,
          replaced_total: 1,
          deleted_total: 0,
          read_error_total: 0,
          sample_truncated: false,
          sample: [],
        },
      },
    })

    expect(parsed.consistencyChangedTotal).toBe(1)
  })

  it('returns null when consistency is absent', () => {
    const parsed = parseRunSummary({ filesystem: { warnings_total: 0, errors_total: 0 } })
    expect(parsed.consistencyChangedTotal).toBe(null)
  })
})

