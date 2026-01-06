import { ApiError } from '@/lib/api'

export type ApiErrorInfo = {
  status?: number
  code?: string
  message: string
  field?: string
  details?: unknown
  requestId?: string
}

type Translator = (key: string, params?: Record<string, unknown>) => string

function translateOrNull(t: Translator, key: string, params?: Record<string, unknown>): string | null {
  const out = t(key, params)
  return out === key ? null : out
}

function extractField(details: unknown): string | undefined {
  if (!details || typeof details !== 'object') return undefined
  if (!('field' in details)) return undefined
  const field = (details as { field?: unknown }).field
  return typeof field === 'string' && field.trim() ? field : undefined
}

function extractRetryAfter(details: unknown): number | undefined {
  if (!details || typeof details !== 'object') return undefined
  if (!('retry_after_seconds' in details)) return undefined
  const raw = (details as { retry_after_seconds?: unknown }).retry_after_seconds
  const n = typeof raw === 'number' ? raw : typeof raw === 'string' ? Number(raw) : Number.NaN
  if (!Number.isFinite(n) || n <= 0) return undefined
  return Math.floor(n)
}

export function toApiErrorInfo(error: unknown, t?: Translator): ApiErrorInfo {
  if (error instanceof ApiError) {
    const code = error.body?.error
    const details = error.body?.details
    const field = extractField(details)
    const retryAfterSeconds = extractRetryAfter(details)
    const requestId = error.requestId && error.requestId.trim() ? error.requestId.trim() : undefined

    if (t && code) {
      const key = `apiErrors.${code}`
      const localized =
        code === 'rate_limited' && retryAfterSeconds
          ? translateOrNull(t, key, { seconds: retryAfterSeconds })
          : translateOrNull(t, key)

      if (localized) {
        const message = withRequestIdSuffix(localized, requestId, error.status, t)
        return {
          status: error.status,
          code,
          message,
          field,
          details,
          requestId,
        }
      }
    }

    const message = withRequestIdSuffix(
      error.body?.message?.trim() || error.message || `HTTP ${error.status}`,
      requestId,
      error.status,
      t,
    )
    return {
      status: error.status,
      code,
      message,
      field,
      details,
      requestId,
    }
  }

  if (error && typeof error === 'object' && 'message' in error) {
    const msg = String((error as { message?: unknown }).message ?? '').trim()
    if (msg) return { message: msg }
  }

  return { message: 'Unknown error' }
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
