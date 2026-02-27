import { describe, expect, it } from 'vitest'

import {
  filterRunEvents,
  findFirstEventSeq,
  runEventDisplayMessage,
  runEventErrorEnvelope,
  runEventHint,
  runEventSummaryChips,
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

  it('normalizes canonical error envelope fields', () => {
    const event = {
      run_id: 'r1',
      seq: 10,
      ts: 10,
      level: 'error',
      kind: 'upload_failed',
      message: 'legacy msg',
      fields: {
        error_envelope: {
          schema_version: '1.0',
          code: 'target.webdav.timeout',
          kind: 'timeout',
          retriable: { value: true, reason: 'transient', retry_after_sec: 5 },
          hint: { key: 'diagnostics.hint.run_failed.timeout', params: {} },
          message: { key: 'diagnostics.message.target.webdav.put_failed', params: { attempt: 3 } },
          transport: {
            protocol: 'http',
            status_code: 504,
            provider_code: 'GATEWAY_TIMEOUT',
            provider_request_id: 'req-123',
          },
          context: { stage: 'upload', target_url: 'https://example.com/upload' },
        },
      },
    }

    expect(runEventErrorEnvelope(event)).toEqual({
      schemaVersion: '1.0',
      code: 'target.webdav.timeout',
      kind: 'timeout',
      retriable: { value: true, reason: 'transient', retryAfterSec: 5 },
      hint: { key: 'diagnostics.hint.run_failed.timeout', params: {} },
      message: { key: 'diagnostics.message.target.webdav.put_failed', params: { attempt: 3 } },
      transport: {
        protocol: 'http',
        statusCode: 504,
        statusText: null,
        provider: null,
        providerCode: 'GATEWAY_TIMEOUT',
        providerRequestId: 'req-123',
        disconnectCode: null,
        ioKind: null,
        osErrorCode: null,
      },
      context: { stage: 'upload', target_url: 'https://example.com/upload' },
    })
  })

  it('uses envelope-first message and hint with legacy fallbacks', () => {
    const event = {
      run_id: 'r1',
      seq: 11,
      ts: 11,
      level: 'error',
      kind: 'upload_failed',
      message: 'legacy msg',
      fields: {
        hint: 'legacy hint',
        error_envelope: {
          code: 'target.webdav.timeout',
          kind: 'timeout',
          retriable: { value: true, reason: null, retry_after_sec: null },
          message: { key: 'diagnostics.message.target.webdav.put_failed', params: { attempt: 3 } },
          hint: { key: 'diagnostics.hint.run_failed.timeout', params: {} },
          transport: { protocol: 'http' },
        },
      },
    }
    const t = (key: string, params?: Record<string, unknown>): string => {
      if (key === 'diagnostics.message.target.webdav.put_failed') {
        return `PUT failed attempt=${params?.attempt as number}`
      }
      if (key === 'diagnostics.hint.run_failed.timeout') return 'Please retry later'
      if (key === 'runEvents.details.genericHint') return 'generic hint'
      if (key === 'runEvents.details.genericMessage') return 'generic message'
      return key
    }

    expect(runEventDisplayMessage(event, t)).toBe('PUT failed attempt=3')
    expect(runEventHint(event, t)).toBe('Please retry later')

    const missingI18n = (key: string): string => key
    expect(runEventDisplayMessage(event, missingI18n)).toBe('legacy msg')
    expect(runEventHint(event, missingI18n)).toBe('legacy hint')
    expect(runEventHint({ ...event, fields: { error_envelope: event.fields.error_envelope } }, missingI18n)).toBeNull()
    expect(runEventHint({ ...event, fields: { error_envelope: event.fields.error_envelope } }, missingI18n, { allowGenericFallback: true })).toBe(
      'runEvents.details.genericHint',
    )
  })

  it('builds summary chips with bounded size and envelope-first diagnostics', () => {
    const event = {
      run_id: 'r1',
      seq: 12,
      ts: 12,
      level: 'error',
      kind: 'upload_failed',
      message: 'legacy msg',
      fields: {
        attempt: 2,
        next_attempt_at: 130,
        part_size_bytes: 16 * 1024 * 1024,
        hint: 'legacy hint fallback',
        error_envelope: {
          code: 'target.webdav.timeout',
          kind: 'timeout',
          retriable: { value: true, reason: null, retry_after_sec: null },
          message: { key: 'diagnostics.message.target.webdav.put_failed', params: {} },
          hint: { key: 'diagnostics.hint.run_failed.timeout', params: {} },
          transport: { protocol: 'http', status_code: 504 },
        },
      },
    }
    const t = (key: string, params?: Record<string, unknown>): string => {
      if (key === 'diagnostics.hint.run_failed.timeout') return 'localized hint'
      if (key === 'common.timeUnits.s') return 's'
      if (key === 'common.relativeTime.in') return `in ${String(params?.value ?? '')}`
      if (key === 'common.relativeTime.ago') return `${String(params?.value ?? '')} ago`
      return key
    }

    expect(runEventSummaryChips(event, t, { nowTs: 100, maxChips: 3 })).toEqual([
      { text: 'timeout', type: 'warning' },
      { text: 'HTTP 504', type: 'warning' },
      { text: '#2', type: 'default' },
    ])

    expect(runEventSummaryChips(event, t, { nowTs: 100, maxChips: 10 })).toContainEqual({
      text: 'in 30s',
      type: 'default',
    })
    expect(runEventSummaryChips(event, t, { nowTs: 100, maxChips: 10 })).toContainEqual({
      text: '16MB',
      type: 'default',
    })
    expect(runEventSummaryChips(event, t, { nowTs: 100, maxChips: 10 })).toContainEqual({
      text: 'localized hint',
      type: 'warning',
    })
  })
})
