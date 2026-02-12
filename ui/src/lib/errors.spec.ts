import { describe, expect, it } from 'vitest'

import { ApiError } from './api'
import { extractApiFieldIssues, formatToastError, resolveApiFieldErrors, toApiErrorInfo } from './errors'

const dict: Record<string, string> = {
  'apiErrors.invalid_webhook_url.default': 'Webhook URL is invalid',
  'apiErrors.invalid_webhook_url.required': 'Webhook URL is required',
  'apiErrors.invalid_password.default': 'Password is invalid',
  'apiErrors.invalid_password.min_length': 'Password must be at least {min_length} characters',
  'apiErrors.rate_limited.throttled': 'Rate limited ({seconds}s)',
  'apiErrors.invalid_to.required': 'Recipient is required',
  'apiErrors.invalid_to.invalid_format': 'Recipient is invalid',
  'common.requestId': 'Request ID',
}

const t = (key: string, params?: Record<string, unknown>): string => {
  const template = dict[key]
  if (!template) return key
  return template.replace(/\{([^}]+)\}/g, (_, name: string) => String(params?.[name] ?? `{${name}}`))
}

describe('toApiErrorInfo', () => {
  it('prefers code+reason translation over generic code', () => {
    const err = new ApiError(400, 'Webhook URL is required', {
      error: 'invalid_webhook_url',
      message: 'Webhook URL is required',
      details: { reason: 'required', field: 'webhook_url' },
    })

    const info = toApiErrorInfo(err, t)
    expect(info.code).toBe('invalid_webhook_url')
    expect(info.reason).toBe('required')
    expect(info.field).toBe('webhook_url')
    expect(info.message).toBe('Webhook URL is required')
  })

  it('falls back to generic code translation when reason key is missing', () => {
    const err = new ApiError(400, 'Webhook URL is invalid', {
      error: 'invalid_webhook_url',
      message: 'Webhook URL is invalid',
      details: { reason: 'unknown_reason', field: 'webhook_url' },
    })

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Webhook URL is invalid')
  })

  it('uses params bag for reason-specific interpolation', () => {
    const err = new ApiError(400, 'Password too short', {
      error: 'invalid_password',
      message: 'Password too short',
      details: { reason: 'min_length', field: 'password', params: { min_length: 12 } },
    })

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Password must be at least 12 characters')
    expect(info.params?.min_length).toBe(12)
  })

  it('supports legacy params aliases for compatibility', () => {
    const err = new ApiError(429, 'Too many attempts', {
      error: 'rate_limited',
      message: 'Too many attempts',
      details: { reason: 'throttled', params: { retry_after_seconds: 9 } },
    })

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Rate limited (9s)')
  })

  it('falls back to backend message for unknown codes', () => {
    const err = new ApiError(400, 'Bad request', {
      error: 'some_new_code',
      message: 'Some human message',
    })

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Some human message')
  })

  it('appends request id suffix on 5xx responses', () => {
    const err = new ApiError(500, 'Internal server error', {
      error: 'internal_error',
      message: 'Internal server error',
    }, 'req-123')

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Internal server error (Request ID: req-123)')
  })
})

describe('field mapping helpers', () => {
  it('extracts single-field issue from details.field', () => {
    const err = new ApiError(400, 'Webhook URL is required', {
      error: 'invalid_webhook_url',
      message: 'Webhook URL is required',
      details: { reason: 'required', field: 'webhook_url' },
    })

    const info = toApiErrorInfo(err, t)
    expect(extractApiFieldIssues(info)).toEqual([
      {
        field: 'webhook_url',
        reason: 'required',
        params: undefined,
      },
    ])
  })

  it('maps violations with field aliasing in one pass', () => {
    const err = new ApiError(400, 'Validation failed', {
      error: 'invalid_to',
      message: 'Validation failed',
      details: {
        violations: [
          { field: 'to', reason: 'required' },
          { field: 'from', reason: 'invalid_format' },
        ],
      },
    })

    const info = toApiErrorInfo(err, t)
    const mapped = resolveApiFieldErrors(info, {
      t,
      fieldMap: {
        to: 'toText',
      },
    })

    expect(mapped).toEqual({
      toText: 'Recipient is required',
      from: 'Recipient is invalid',
    })
  })
})

describe('formatToastError', () => {
  it('prefixes title with resolved api error message', () => {
    const err = new ApiError(400, 'Webhook URL is required', {
      error: 'invalid_webhook_url',
      message: 'Webhook URL is required',
      details: { reason: 'required', field: 'webhook_url' },
    })
    expect(formatToastError('Save failed', err, t)).toBe('Save failed: Webhook URL is required')
  })
})
