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

export type RunEventSummaryChip = {
  text: string
  type: 'default' | 'warning' | 'error' | 'success'
}

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

function formatBytesCompact(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '0B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let value = bytes
  let idx = 0
  while (value >= 1024 && idx < units.length - 1) {
    value /= 1024
    idx += 1
  }
  const fixed = value >= 10 || idx === 0 ? value.toFixed(0) : value.toFixed(1)
  return `${fixed}${units[idx]}`
}

function formatDurationMs(ms: number): string {
  if (ms < 1000) return `${Math.max(0, Math.floor(ms))}ms`
  const secs = ms / 1000
  if (secs < 10) return `${secs.toFixed(1)}s`
  return `${Math.round(secs)}s`
}

function formatRelativeSeconds(seconds: number, t: RunEventTranslate): string {
  const abs = Math.abs(seconds)
  const unit =
    abs >= 86400 ? { n: Math.round(abs / 86400), s: 'd' } : abs >= 3600
      ? { n: Math.round(abs / 3600), s: 'h' }
      : abs >= 60
        ? { n: Math.round(abs / 60), s: 'm' }
        : { n: Math.round(abs), s: 's' }
  const value = `${unit.n}${t(`common.timeUnits.${unit.s}`)}`
  return seconds >= 0 ? t('common.relativeTime.in', { value }) : t('common.relativeTime.ago', { value })
}

function shortId(value: string): string {
  if (value.length <= 10) return value
  return `${value.slice(0, 8)}…`
}

export function runEventSummaryChips(
  event: RunEvent,
  t: RunEventTranslate,
  options: {
    nowTs?: number
    maxChips?: number
  } = {},
): RunEventSummaryChip[] {
  const fields = runEventFieldsRecord(event)
  if (!fields) return []
  const envelope = runEventErrorEnvelope(event)
  const nowTs = options.nowTs ?? Math.floor(Date.now() / 1000)
  const maxChips = Math.max(1, options.maxChips ?? 3)
  const out: RunEventSummaryChip[] = []
  const push = (chip: RunEventSummaryChip): void => {
    if (out.length >= maxChips) return
    out.push(chip)
  }

  const errorKind = envelope?.kind ?? toNonEmptyString(fields.error_kind) ?? toNonEmptyString(fields.last_error_kind)
  if (errorKind) {
    const type: RunEventSummaryChip['type'] = errorKind === 'auth' || errorKind === 'config' ? 'error' : 'warning'
    push({ text: errorKind, type })
  }

  const httpStatus = envelope?.transport.statusCode ?? toFiniteNumber(fields.http_status)
  if (httpStatus != null) {
    const rounded = Math.max(0, Math.floor(httpStatus))
    const type: RunEventSummaryChip['type'] = rounded >= 500 ? 'warning' : rounded >= 400 ? 'error' : 'default'
    push({ text: `HTTP ${rounded}`, type })
  }

  const attempt = toFiniteNumber(fields.attempt) ?? toFiniteNumber(fields.attempts)
  if (attempt != null) {
    push({ text: `#${Math.max(0, Math.floor(attempt))}`, type: 'default' })
  }

  const nextAttemptAt = toFiniteNumber(fields.next_attempt_at)
  if (nextAttemptAt != null) {
    push({ text: formatRelativeSeconds(nextAttemptAt - nowTs, t), type: 'default' })
  }

  const durationMs = toFiniteNumber(fields.duration_ms)
  if (durationMs != null) {
    push({ text: formatDurationMs(durationMs), type: 'default' })
  }

  const errorsTotal = toFiniteNumber(fields.errors_total)
  const warningsTotal = toFiniteNumber(fields.warnings_total)
  if (errorsTotal != null || warningsTotal != null) {
    const errors = Math.max(0, Math.floor(errorsTotal ?? 0))
    const warnings = Math.max(0, Math.floor(warningsTotal ?? 0))
    push({ text: `E${errors}/W${warnings}`, type: errors > 0 ? 'error' : warnings > 0 ? 'warning' : 'default' })
  }

  const ok = typeof fields.ok === 'boolean' ? fields.ok : null
  if (ok != null) {
    push({ text: ok ? 'OK' : 'FAIL', type: ok ? 'success' : 'error' })
  }

  const channel = toNonEmptyString(fields.channel)
  if (channel) push({ text: channel, type: 'default' })

  const source = toNonEmptyString(fields.source)
  if (source) push({ text: source, type: 'default' })

  const executedOffline = typeof fields.executed_offline === 'boolean' ? fields.executed_offline : null
  if (executedOffline === true) push({ text: t('runs.badges.offline'), type: 'default' })

  const agentId = toNonEmptyString(fields.agent_id)
  if (agentId) push({ text: shortId(agentId), type: 'default' })

  const secretName = toNonEmptyString(fields.secret_name)
  if (secretName) push({ text: secretName, type: 'default' })

  const partName = toNonEmptyString(fields.part_name)
  if (partName) push({ text: partName, type: 'default' })

  const partSize = toFiniteNumber(fields.part_size_bytes)
  if (partSize != null) push({ text: formatBytesCompact(partSize), type: 'default' })

  const transportCode = envelope?.transport.providerCode ?? toNonEmptyString(fields.transport_code)
  if (transportCode) push({ text: transportCode, type: 'default' })

  const hint = runEventHint(event, t)
  if (hint) push({ text: hint, type: 'warning' })

  return out
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
