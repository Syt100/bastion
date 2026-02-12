import { ApiError } from '@/lib/api'

export type ApiErrorParams = Record<string, unknown>

export type ApiErrorViolation = {
  field: string
  reason?: string
  params?: ApiErrorParams
  message?: string
}

export type ApiErrorInfo = {
  status?: number
  code?: string
  message: string
  reason?: string
  field?: string
  params?: ApiErrorParams
  violations?: ApiErrorViolation[]
  details?: unknown
  requestId?: string
}

export type ApiFieldIssue = ApiErrorViolation

type Translator = (key: string, params?: Record<string, unknown>) => string

type JsonRecord = Record<string, unknown>

function translateOrNull(t: Translator, key: string, params?: Record<string, unknown>): string | null {
  const out = t(key, params)
  return out === key ? null : out
}

function isRecord(value: unknown): value is JsonRecord {
  return !!value && typeof value === 'object' && !Array.isArray(value)
}

function normalizeText(value: unknown): string | undefined {
  if (typeof value !== 'string') return undefined
  const out = value.trim()
  return out || undefined
}

function toParams(value: unknown): ApiErrorParams | undefined {
  if (!isRecord(value)) return undefined
  const entries = Object.entries(value)
  if (entries.length === 0) return undefined
  return Object.fromEntries(entries)
}

function camelCaseKey(key: string): string {
  return key.replace(/_([a-z])/g, (_, c: string) => c.toUpperCase())
}

function withParamAliases(params: ApiErrorParams | undefined): ApiErrorParams | undefined {
  if (!params) return undefined

  const out: ApiErrorParams = { ...params }
  for (const [key, value] of Object.entries(params)) {
    if (key.includes('_')) {
      const camel = camelCaseKey(key)
      if (!(camel in out)) out[camel] = value
    }
    if (key === 'retry_after_seconds' && !("seconds" in out)) {
      out.seconds = value
    }
  }

  return out
}

function extractReason(details: unknown): string | undefined {
  if (!isRecord(details)) return undefined
  return normalizeText(details.reason)
}

function extractField(details: unknown): string | undefined {
  if (!isRecord(details)) return undefined
  return normalizeText(details.field)
}

function extractParams(details: unknown): ApiErrorParams | undefined {
  if (!isRecord(details)) return undefined

  const explicit = withParamAliases(toParams(details.params))
  if (explicit) return explicit

  const legacyParams: JsonRecord = {}
  for (const [key, value] of Object.entries(details)) {
    if (key === 'reason' || key === 'field' || key === 'params' || key === 'violations') continue
    legacyParams[key] = value
  }
  return withParamAliases(toParams(legacyParams))
}

function extractViolations(details: unknown): ApiErrorViolation[] | undefined {
  if (!isRecord(details)) return undefined
  if (!Array.isArray(details.violations)) return undefined

  const out: ApiErrorViolation[] = []
  for (const item of details.violations) {
    if (!isRecord(item)) continue
    const field = normalizeText(item.field)
    if (!field) continue

    out.push({
      field,
      reason: normalizeText(item.reason),
      params: withParamAliases(toParams(item.params)),
      message: normalizeText(item.message),
    })
  }

  return out.length > 0 ? out : undefined
}

function resolveLocalizedApiErrorMessage(
  t: Translator,
  code: string,
  reason?: string,
  params?: ApiErrorParams,
): string | null {
  if (reason) {
    const reasoned = translateOrNull(t, `apiErrors.${code}.${reason}`, params)
    if (reasoned) return reasoned
  }

  const generic = translateOrNull(t, `apiErrors.${code}`, params)
  if (generic) return generic

  return translateOrNull(t, `apiErrors.${code}.default`, params)
}

export function toApiErrorInfo(error: unknown, t?: Translator): ApiErrorInfo {
  if (error instanceof ApiError) {
    const code = normalizeText(error.body?.error)
    const details = error.body?.details
    const reason = extractReason(details)
    const field = extractField(details)
    const params = extractParams(details)
    const violations = extractViolations(details)
    const requestId = normalizeText(error.requestId)

    const localized = t && code ? resolveLocalizedApiErrorMessage(t, code, reason, params) : null
    const fallbackMessage = normalizeText(error.body?.message) || normalizeText(error.message) || `HTTP ${error.status}`
    const message = withRequestIdSuffix(localized || fallbackMessage, requestId, error.status, t)

    return {
      status: error.status,
      code,
      message,
      reason,
      field,
      params,
      violations,
      details,
      requestId,
    }
  }

  if (error && typeof error === 'object' && 'message' in error) {
    const msg = normalizeText((error as { message?: unknown }).message)
    if (msg) return { message: msg }
  }

  return { message: 'Unknown error' }
}

export function extractApiFieldIssues(info: ApiErrorInfo): ApiFieldIssue[] {
  if (Array.isArray(info.violations) && info.violations.length > 0) {
    return info.violations
      .filter((v) => normalizeText(v.field))
      .map((v) => ({
        field: v.field,
        reason: v.reason,
        params: v.params,
        message: v.message,
      }))
  }

  if (!info.field) return []
  return [
    {
      field: info.field,
      reason: info.reason,
      params: info.params,
    },
  ]
}

export function resolveApiFieldErrors(
  info: ApiErrorInfo,
  options: {
    t?: Translator
    fieldMap?: Record<string, string>
  } = {},
): Record<string, string> {
  const issues = extractApiFieldIssues(info)
  if (issues.length === 0) return {}

  const out: Record<string, string> = {}
  for (const issue of issues) {
    const targetField = options.fieldMap?.[issue.field] || issue.field
    if (!targetField || out[targetField]) continue

    const reason = issue.reason ?? info.reason
    const params = issue.params ?? info.params

    const localized = options.t && info.code ? resolveLocalizedApiErrorMessage(options.t, info.code, reason, params) : null
    const message = localized || issue.message || info.message
    if (!message) continue

    out[targetField] = message
  }

  return out
}

function withRequestIdSuffix(
  message: string,
  requestId: string | undefined,
  status: number | undefined,
  t?: Translator,
): string {
  if (!message) return message
  if (!requestId) return message
  if (!status || status < 500) return message
  const label = (t && translateOrNull(t, 'common.requestId')) || 'Request ID'
  return `${message} (${label}: ${requestId})`
}

export function formatToastError(
  title: string,
  error: unknown,
  t?: Translator,
): string {
  const info = toApiErrorInfo(error, t)
  if (!info.message || info.message === title) return title
  return `${title}: ${info.message}`
}
