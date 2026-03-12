import { describe, expect, it } from 'vitest'

import {
  envelopeEventDiagnostic,
  latestEnvelopeDiagnosticEvent,
  preferredEventDiagnostic,
} from './eventDiagnostics'

describe('eventDiagnostics helpers', () => {
  const t = (key: string): string => {
    if (key === 'diagnostics.message.execute.snapshot_cleanup_failed') return 'Snapshot cleanup failed'
    if (key === 'diagnostics.hint.execute.snapshot_cleanup_failed') return 'Clean up stale snapshots'
    if (key === 'runEvents.details.genericHint') return 'generic hint'
    if (key === 'runEvents.details.genericMessage') return 'generic message'
    return key
  }

  it('picks the latest event with an error envelope', () => {
    const event = latestEnvelopeDiagnosticEvent([
      {
        run_id: 'run-1',
        seq: 1,
        ts: 1,
        level: 'warn',
        kind: 'failed',
        message: 'legacy 1',
        fields: null,
      },
      {
        run_id: 'run-1',
        seq: 2,
        ts: 2,
        level: 'warn',
        kind: 'failed',
        message: 'legacy 2',
        fields: {
          error_envelope: {
            code: 'scheduler.execute.filesystem.snapshot_cleanup_failed',
            kind: 'io',
            retriable: { value: false, reason: null, retry_after_sec: null },
            hint: { key: 'diagnostics.hint.execute.snapshot_cleanup_failed', params: {} },
            message: { key: 'diagnostics.message.execute.snapshot_cleanup_failed', params: {} },
            transport: { protocol: 'file' },
          },
        },
      },
    ])

    expect(event?.seq).toBe(2)
  })

  it('prefers envelope diagnostics over legacy fields', () => {
    const diagnostic = preferredEventDiagnostic(
      [
        {
          run_id: 'run-1',
          seq: 2,
          ts: 2,
          level: 'warn',
          kind: 'failed',
          message: 'legacy event',
          fields: {
            error_envelope: {
              code: 'scheduler.execute.filesystem.snapshot_cleanup_failed',
              kind: 'io',
              retriable: { value: false, reason: null, retry_after_sec: null },
              hint: { key: 'diagnostics.hint.execute.snapshot_cleanup_failed', params: {} },
              message: { key: 'diagnostics.message.execute.snapshot_cleanup_failed', params: {} },
              transport: { protocol: 'file' },
            },
          },
        },
      ],
      t,
      'unknown',
      'legacy fallback',
    )

    expect(diagnostic).toEqual({
      kind: 'io',
      message: 'Snapshot cleanup failed',
      hint: 'Clean up stale snapshots',
      copyText: 'io\nSnapshot cleanup failed\nClean up stale snapshots',
      source: 'envelope',
    })
  })

  it('falls back to legacy task diagnostics when no envelope exists', () => {
    const diagnostic = preferredEventDiagnostic([], t, 'network', 'dial tcp timeout')

    expect(diagnostic).toEqual({
      kind: 'network',
      message: 'dial tcp timeout',
      hint: null,
      copyText: 'network: dial tcp timeout',
      source: 'legacy',
    })
  })

  it('builds an envelope diagnostic from a single event', () => {
    const diagnostic = envelopeEventDiagnostic(
      {
        run_id: 'run-1',
        seq: 3,
        ts: 3,
        level: 'warn',
        kind: 'failed',
        message: 'legacy event',
        fields: {
          error_envelope: {
            code: 'scheduler.execute.filesystem.snapshot_cleanup_failed',
            kind: 'io',
            retriable: { value: false, reason: null, retry_after_sec: null },
            hint: { key: 'diagnostics.hint.execute.snapshot_cleanup_failed', params: {} },
            message: { key: 'diagnostics.message.execute.snapshot_cleanup_failed', params: {} },
            transport: { protocol: 'file' },
          },
        },
      },
      t,
    )

    expect(diagnostic?.message).toBe('Snapshot cleanup failed')
    expect(diagnostic?.hint).toBe('Clean up stale snapshots')
  })
})
