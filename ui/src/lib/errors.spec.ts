import { describe, expect, it } from 'vitest'

import { ApiError } from './api'
import { formatToastError, toApiErrorInfo } from './errors'

const t = (key: string, params?: Record<string, unknown>): string => {
  const dict: Record<string, string> = {
    'apiErrors.invalid_webhook_url': 'Webhook URL is invalid',
    'apiErrors.rate_limited': `Rate limited${params?.seconds ? ` (${params.seconds}s)` : ''}`,
  }
  return dict[key] ?? key
}

describe('toApiErrorInfo', () => {
  it('localizes known error codes', () => {
    const err = new ApiError(400, 'Webhook URL is invalid', {
      error: 'invalid_webhook_url',
      message: 'Webhook URL is invalid',
      details: { field: 'webhook_url' },
    })
    const info = toApiErrorInfo(err, t)
    expect(info.code).toBe('invalid_webhook_url')
    expect(info.field).toBe('webhook_url')
    expect(info.message).toBe('Webhook URL is invalid')
  })

  it('localizes rate_limited with retry_after_seconds', () => {
    const err = new ApiError(429, 'Too many login attempts', {
      error: 'rate_limited',
      message: 'Too many login attempts',
      details: { retry_after_seconds: 12 },
    })
    const info = toApiErrorInfo(err, t)
    expect(info.code).toBe('rate_limited')
    expect(info.message).toBe('Rate limited (12s)')
  })

  it('falls back to backend message for unknown codes', () => {
    const err = new ApiError(400, 'Bad request', {
      error: 'some_new_code',
      message: 'Some human message',
    })
    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Some human message')
  })
})

describe('formatToastError', () => {
  it('prefixes title with reason', () => {
    const err = new ApiError(400, 'Webhook URL is invalid', {
      error: 'invalid_webhook_url',
      message: 'Webhook URL is invalid',
    })
    expect(formatToastError('Save failed', err, t)).toBe('Save failed: Webhook URL is invalid')
  })
})

