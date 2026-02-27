import { describe, expect, it } from 'vitest'

import {
  filterRunEvents,
  findFirstEventSeq,
  runEventTransportMetadata,
  uniqueRunEventKinds,
  RUN_EVENT_DETAIL_HEADER_META_FIELDS_DEFAULT,
  RUN_EVENT_DETAIL_HEADER_META_FIELDS_WITH_IDENTIFIERS,
} from './run_events'

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

  it('provides shared header meta schemas', () => {
    expect(RUN_EVENT_DETAIL_HEADER_META_FIELDS_DEFAULT).toEqual(['timestamp', 'level', 'kind'])
    expect(RUN_EVENT_DETAIL_HEADER_META_FIELDS_WITH_IDENTIFIERS).toEqual([
      'timestamp',
      'level',
      'kind',
      'seq',
      'requestId',
    ])
  })

  it('extracts protocol-aware request and trace metadata', () => {
    const httpEvent = {
      run_id: 'r1',
      seq: 1,
      ts: 1,
      level: 'error',
      kind: 'failed',
      message: 'failed',
      fields: {
        error_envelope: {
          transport: { protocol: 'http' },
        },
        response: {
          headers: {
            'x-request-id': 'http-req-1',
            'x-trace-id': 'http-trace-1',
          },
        },
      },
    }
    const sftpEvent = {
      run_id: 'r1',
      seq: 2,
      ts: 2,
      level: 'error',
      kind: 'failed',
      message: 'failed',
      fields: {
        error_envelope: {
          transport: { protocol: 'sftp', provider_request_id: 'fallback-provider-id' },
        },
        sftp_request_id: 'sftp-req-1',
        session_id: 'sftp-session-1',
      },
    }
    const unknownEvent = {
      run_id: 'r1',
      seq: 3,
      ts: 3,
      level: 'error',
      kind: 'failed',
      message: 'failed',
      fields: {
        request_id: 'generic-req',
        trace_id: 'generic-trace',
      },
    }

    expect(runEventTransportMetadata(httpEvent)).toEqual({
      protocol: 'http',
      requestId: 'http-req-1',
      traceId: 'http-trace-1',
    })
    expect(runEventTransportMetadata(sftpEvent)).toEqual({
      protocol: 'sftp',
      requestId: 'sftp-req-1',
      traceId: 'sftp-session-1',
    })
    expect(runEventTransportMetadata(unknownEvent)).toEqual({
      protocol: null,
      requestId: 'generic-req',
      traceId: 'generic-trace',
    })
  })
})
