import type { RunEvent } from '@/stores/jobs'

export type RunEventFilters = {
  query?: string | null | undefined
  level?: string | null | undefined
  kind?: string | null | undefined
}

export type RunEventJsonRecord = Record<string, unknown>

export type RunEventEnvelopeTextRef = {
  key: string
  params: RunEventJsonRecord
}

export type RunEventErrorEnvelope = {
  schemaVersion: string | null
  code: string
  kind: string
  retriable: {
    value: boolean
    reason: string | null
    retryAfterSec: number | null
  }
  hint: RunEventEnvelopeTextRef | null
  message: RunEventEnvelopeTextRef | null
  transport: {
    protocol: string
    statusCode: number | null
    statusText: string | null
    provider: string | null
    providerCode: string | null
    providerRequestId: string | null
    disconnectCode: number | null
    ioKind: string | null
    osErrorCode: number | null
  }
  context: RunEventJsonRecord | null
}

export type RunEventTranslate = (key: string, params?: Record<string, unknown>) => string

export type RunEventDetailHeaderMetaField = 'timestamp' | 'level' | 'kind' | 'seq' | 'traceId' | 'requestId'

export const RUN_EVENT_DETAIL_HEADER_META_FIELDS_DEFAULT: readonly RunEventDetailHeaderMetaField[] = ['timestamp', 'level', 'kind']
export const RUN_EVENT_DETAIL_HEADER_META_FIELDS_WITH_IDENTIFIERS: readonly RunEventDetailHeaderMetaField[] = [
  'timestamp',
  'level',
  'kind',
  'seq',
  'requestId',
]

export type RunEventTransportMetadata = {
  protocol: string | null
  requestId: string | null
  traceId: string | null
}

type TransportMetadataAdapter = {
  requestIdPaths: string[][]
  traceIdPaths: string[][]
}

function norm(s: string | null | undefined): string {
  return (s ?? '').trim().toLowerCase()
}

function toRecord(value: unknown): RunEventJsonRecord | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as RunEventJsonRecord
}

function toNonEmptyString(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const normalized = value.trim()
  return normalized.length > 0 ? normalized : null
}

function toFiniteNumber(value: unknown): number | null {
  if (typeof value !== 'number' || !Number.isFinite(value)) return null
  return value
}

function toBoolean(value: unknown): boolean | null {
  if (typeof value !== 'boolean') return null
  return value
}

function pickPathString(root: unknown, path: string[]): string | null {
  let current: unknown = root
  for (const key of path) {
    const record = toRecord(current)
    if (!record) return null
    current = record[key]
  }
  return toNonEmptyString(current)
}

const COMMON_METADATA_ADAPTER: TransportMetadataAdapter = {
  requestIdPaths: [
    ['error_envelope', 'transport', 'provider_request_id'],
    ['error_envelope', 'transport', 'request_id'],
    ['error_envelope', 'context', 'request_id'],
    ['provider_request_id'],
    ['request_id'],
    ['requestId'],
  ],
  traceIdPaths: [
    ['error_envelope', 'context', 'trace_id'],
    ['error_envelope', 'context', 'traceId'],
    ['error_envelope', 'transport', 'trace_id'],
    ['trace_id'],
    ['traceId'],
  ],
}

const PROTOCOL_METADATA_ADAPTERS: Record<string, TransportMetadataAdapter> = {
  http: {
    requestIdPaths: [
      ['response', 'headers', 'x-request-id'],
      ['response', 'headers', 'x-amz-request-id'],
      ['response', 'headers', 'x-ms-request-id'],
    ],
    traceIdPaths: [
      ['response', 'headers', 'x-trace-id'],
      ['response', 'headers', 'x-correlation-id'],
      ['response', 'headers', 'traceparent'],
    ],
  },
  webdav: {
    requestIdPaths: [
      ['response', 'headers', 'x-request-id'],
      ['response', 'headers', 'x-amz-request-id'],
      ['response', 'headers', 'x-ms-request-id'],
    ],
    traceIdPaths: [
      ['response', 'headers', 'x-trace-id'],
      ['response', 'headers', 'x-correlation-id'],
      ['response', 'headers', 'traceparent'],
    ],
  },
  sftp: {
    requestIdPaths: [
      ['error_envelope', 'transport', 'sftp_request_id'],
      ['error_envelope', 'transport', 'ssh_request_id'],
      ['sftp_request_id'],
      ['ssh_request_id'],
    ],
    traceIdPaths: [
      ['error_envelope', 'context', 'session_id'],
      ['session_id'],
    ],
  },
}

function firstNonEmptyPathValue(root: unknown, paths: string[][]): string | null {
  for (const path of paths) {
    const value = pickPathString(root, path)
    if (value) return value
  }
  return null
}

function normalizeTextRef(value: unknown): RunEventEnvelopeTextRef | null {
  const obj = toRecord(value)
  if (!obj) return null
  const key = toNonEmptyString(obj.key)
  if (!key) return null
  return { key, params: toRecord(obj.params) ?? {} }
}

function resolveTextRef(textRef: RunEventEnvelopeTextRef | null, t: RunEventTranslate): string | null {
  if (!textRef) return null
  const translated = t(textRef.key, textRef.params)
  return translated === textRef.key ? null : translated
}

export function runEventFieldsRecord(event: RunEvent | null | undefined): RunEventJsonRecord | null {
  return toRecord(event?.fields ?? null)
}

export function runEventErrorEnvelope(event: RunEvent | null | undefined): RunEventErrorEnvelope | null {
  const fields = runEventFieldsRecord(event)
  if (!fields) return null
  const envelope = toRecord(fields.error_envelope)
  if (!envelope) return null

  const code = toNonEmptyString(envelope.code)
  const kind = toNonEmptyString(envelope.kind)
  const retriable = toRecord(envelope.retriable)
  const transport = toRecord(envelope.transport)
  if (!code || !kind || !retriable || !transport) return null

  const retriableValue = toBoolean(retriable.value)
  const protocol = toNonEmptyString(transport.protocol)
  if (retriableValue == null || !protocol) return null

  return {
    schemaVersion: toNonEmptyString(envelope.schema_version),
    code,
    kind,
    retriable: {
      value: retriableValue,
      reason: toNonEmptyString(retriable.reason),
      retryAfterSec: toFiniteNumber(retriable.retry_after_sec),
    },
    hint: normalizeTextRef(envelope.hint),
    message: normalizeTextRef(envelope.message),
    transport: {
      protocol,
      statusCode: toFiniteNumber(transport.status_code),
      statusText: toNonEmptyString(transport.status_text),
      provider: toNonEmptyString(transport.provider),
      providerCode: toNonEmptyString(transport.provider_code),
      providerRequestId: toNonEmptyString(transport.provider_request_id),
      disconnectCode: toFiniteNumber(transport.disconnect_code),
      ioKind: toNonEmptyString(transport.io_kind),
      osErrorCode: toFiniteNumber(transport.os_error_code),
    },
    context: toRecord(envelope.context),
  }
}

export function runEventDisplayMessage(event: RunEvent, t: RunEventTranslate): string {
  const envelope = runEventErrorEnvelope(event)
  if (!envelope) return event.message
  const localized = resolveTextRef(envelope.message, t)
  if (localized) return localized
  const legacyMessage = toNonEmptyString(event.message)
  if (legacyMessage) return legacyMessage
  return t('runEvents.details.genericMessage')
}

export function runEventHint(
  event: RunEvent | null | undefined,
  t: RunEventTranslate,
  options: { allowGenericFallback?: boolean } = {},
): string | null {
  if (!event) return null
  const fields = runEventFieldsRecord(event)
  const legacyHint = toNonEmptyString(fields?.hint)
  const envelope = runEventErrorEnvelope(event)
  if (!envelope) return legacyHint
  const localized = resolveTextRef(envelope.hint, t)
  if (localized) return localized
  if (legacyHint) return legacyHint
  if (options.allowGenericFallback) return t('runEvents.details.genericHint')
  return null
}

export function runEventTransportMetadata(event: RunEvent): RunEventTransportMetadata {
  const fields = runEventFieldsRecord(event)
  const protocol = (
    pickPathString(fields, ['error_envelope', 'transport', 'protocol']) ??
    pickPathString(fields, ['transport', 'protocol']) ??
    pickPathString(fields, ['protocol'])
  )?.toLowerCase() ?? null

  const adapter = protocol ? PROTOCOL_METADATA_ADAPTERS[protocol] : undefined
  const requestId = firstNonEmptyPathValue(fields, [
    ...(adapter?.requestIdPaths ?? []),
    ...COMMON_METADATA_ADAPTER.requestIdPaths,
  ])
  const traceId = firstNonEmptyPathValue(fields, [
    ...(adapter?.traceIdPaths ?? []),
    ...COMMON_METADATA_ADAPTER.traceIdPaths,
  ])

  return { protocol, requestId, traceId }
}

export function uniqueRunEventKinds(events: RunEvent[]): string[] {
  const set = new Set<string>()
  for (const e of events) {
    const k = (e.kind ?? '').trim()
    if (k) set.add(k)
  }
  return [...set].sort((a, b) => a.localeCompare(b))
}

export function filterRunEvents(events: RunEvent[], filters: RunEventFilters): RunEvent[] {
  const q = norm(filters.query)
  const level = norm(filters.level)
  const kind = (filters.kind ?? '').trim()

  if (!q && !level && !kind) return events

  return events.filter((e) => {
    if (level && norm(e.level) !== level) return false
    if (kind && e.kind !== kind) return false
    if (!q) return true
    const message = norm(e.message)
    const k = norm(e.kind)
    return message.includes(q) || k.includes(q)
  })
}

export function findFirstEventSeq(events: RunEvent[], predicate: (e: RunEvent) => boolean): number | null {
  for (const e of events) {
    if (predicate(e)) return e.seq
  }
  return null
}

export function runEventLevelTagType(level: string): 'success' | 'error' | 'warning' | 'default' {
  if (level === 'error') return 'error'
  if (level === 'warn' || level === 'warning') return 'warning'
  if (level === 'info') return 'success'
  return 'default'
}
