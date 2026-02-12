import { describe, expect, it } from 'vitest'

import { ApiError } from './api'
import { extractApiFieldIssues, formatToastError, resolveApiFieldErrors, toApiErrorInfo } from './errors'

const dict: Record<string, string> = {
  'apiErrors.invalid_webhook_url.default': 'Webhook URL is invalid',
  'apiErrors.invalid_webhook_url.required': 'Webhook URL is required',
  'apiErrors.invalid_password.default': 'Password is invalid',
  'apiErrors.invalid_password.min_length': 'Password must be at least {min_length} characters',
  'apiErrors.invalid_name.default': 'Name is invalid',
  'apiErrors.invalid_name.required': 'Name is required',
  'apiErrors.rate_limited.throttled': 'Rate limited ({seconds}s)',
  'apiErrors.invalid_to.required': 'Recipient is required',
  'apiErrors.invalid_to.invalid_format': 'Recipient is invalid',
  'apiErrors.invalid_token.exhausted': 'Token has no remaining uses',
  'apiErrors.invalid_token.default': 'Invalid token',
  'apiErrors.invalid_cursor.missing_key': 'Invalid cursor: missing {key} key',
  'apiErrors.invalid_label.max_length': 'Label must be at most {max_length} characters',
  'apiErrors.invalid_job_id.max_length': 'Job id must be at most {max_length} characters',
  'apiErrors.invalid_agent_id.revoked': 'Agent is revoked',
  'apiErrors.invalid_run_id.already_associated': 'Run id is already associated with a different job',
  'apiErrors.invalid_selector.resolved_empty': 'Selector resolved to no agents',
  'apiErrors.invalid_path.invalid_segment': 'Path contains invalid segment',
  'apiErrors.invalid_sort_by.unsupported_value': 'Invalid sort field',
  'apiErrors.invalid_channel.unsupported_value': 'Notification channel is invalid',
  'apiErrors.invalid_kind.unsupported_value': 'Type is invalid',
  'apiErrors.invalid_timezone.invalid_format': 'Invalid timezone',
  'apiErrors.invalid_page.invalid_format': 'Invalid page number',
  'apiErrors.invalid_page.must_be_positive': 'Page must be at least {min}',
  'apiErrors.invalid_page_size.invalid_format': 'Invalid page size',
  'apiErrors.invalid_page_size.must_be_positive': 'Page size must be at least {min}',
  'apiErrors.invalid_retention.invalid_policy': 'Retention policy is invalid',
  'apiErrors.missing_webdav_secret.not_found': 'WebDAV credential not found',
  'apiErrors.webdav_list_failed.client_init_failed': 'Failed to initialize WebDAV client',
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

  it('uses reason-specific translation for token variants', () => {
    const err = new ApiError(401, 'Invalid enrollment token', {
      error: 'invalid_token',
      message: 'Invalid enrollment token',
      details: { reason: 'exhausted', field: 'token' },
    })

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Token has no remaining uses')
  })

  it('interpolates reason params for cursor errors', () => {
    const err = new ApiError(400, 'cursor missing mtime key', {
      error: 'invalid_cursor',
      message: 'cursor missing mtime key',
      details: { reason: 'missing_key', field: 'cursor', params: { key: 'mtime' } },
    })

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Invalid cursor: missing mtime key')
  })

  it('interpolates reason params for label length errors', () => {
    const err = new ApiError(400, 'Label is too long', {
      error: 'invalid_label',
      message: 'Label is too long',
      details: { reason: 'max_length', field: 'labels', params: { max_length: 32 } },
    })

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Label must be at most 32 characters')
  })

  it('uses reason-specific translation for selector errors', () => {
    const err = new ApiError(400, 'Selector resolved to no agents', {
      error: 'invalid_selector',
      message: 'Selector resolved to no agents',
      details: { reason: 'resolved_empty', field: 'selector.labels' },
    })

    const info = toApiErrorInfo(err, t)
    expect(info.message).toBe('Selector resolved to no agents')
  })

  it('supports structured reasons for run, job and agent identifiers', () => {
    const runIdErr = new ApiError(400, 'Run id is already associated', {
      error: 'invalid_run_id',
      message: 'Run id is already associated',
      details: { reason: 'already_associated', field: 'run.id' },
    })
    const runIdInfo = toApiErrorInfo(runIdErr, t)
    expect(runIdInfo.message).toBe('Run id is already associated with a different job')

    const jobIdErr = new ApiError(400, 'Job id is too long', {
      error: 'invalid_job_id',
      message: 'Job id is too long',
      details: { reason: 'max_length', field: 'run.job_id', params: { max_length: 128 } },
    })
    const jobIdInfo = toApiErrorInfo(jobIdErr, t)
    expect(jobIdInfo.message).toBe('Job id must be at most 128 characters')

    const agentIdErr = new ApiError(400, 'Agent is revoked', {
      error: 'invalid_agent_id',
      message: 'Agent is revoked',
      details: { reason: 'revoked', field: 'agent_id' },
    })
    const agentIdInfo = toApiErrorInfo(agentIdErr, t)
    expect(agentIdInfo.message).toBe('Agent is revoked')
  })

  it('supports structured reasons for path and sort errors', () => {
    const pathErr = new ApiError(400, 'invalid path segment', {
      error: 'invalid_path',
      message: 'invalid path segment',
      details: { reason: 'invalid_segment', field: 'path' },
    })
    const pathInfo = toApiErrorInfo(pathErr, t)
    expect(pathInfo.message).toBe('Path contains invalid segment')

    const sortErr = new ApiError(400, 'invalid sort_by', {
      error: 'invalid_sort_by',
      message: 'invalid sort_by',
      details: { reason: 'unsupported_value', field: 'sort_by' },
    })
    const sortInfo = toApiErrorInfo(sortErr, t)
    expect(sortInfo.message).toBe('Invalid sort field')
  })

  it('supports structured reasons for channel and kind errors', () => {
    const channelErr = new ApiError(400, 'Unsupported notification channel', {
      error: 'invalid_channel',
      message: 'Unsupported notification channel',
      details: { reason: 'unsupported_value', field: 'channel' },
    })
    const channelInfo = toApiErrorInfo(channelErr, t)
    expect(channelInfo.message).toBe('Notification channel is invalid')

    const kindErr = new ApiError(400, 'Invalid kind', {
      error: 'invalid_kind',
      message: 'Invalid kind',
      details: { reason: 'unsupported_value', field: 'kind' },
    })
    const kindInfo = toApiErrorInfo(kindErr, t)
    expect(kindInfo.message).toBe('Type is invalid')
  })

  it('supports structured reasons for timezone and pagination errors', () => {
    const timezoneErr = new ApiError(400, 'Invalid schedule timezone', {
      error: 'invalid_timezone',
      message: 'Invalid schedule timezone',
      details: { reason: 'invalid_format', field: 'schedule_timezone' },
    })
    const timezoneInfo = toApiErrorInfo(timezoneErr, t)
    expect(timezoneInfo.message).toBe('Invalid timezone')

    const pageErr = new ApiError(400, 'Invalid page', {
      error: 'invalid_page',
      message: 'Invalid page',
      details: { reason: 'must_be_positive', field: 'page', params: { min: 1 } },
    })
    const pageInfo = toApiErrorInfo(pageErr, t)
    expect(pageInfo.message).toBe('Page must be at least 1')

    const pageSizeErr = new ApiError(400, 'Invalid page_size', {
      error: 'invalid_page_size',
      message: 'Invalid page_size',
      details: { reason: 'invalid_format', field: 'page_size' },
    })
    const pageSizeInfo = toApiErrorInfo(pageSizeErr, t)
    expect(pageSizeInfo.message).toBe('Invalid page size')
  })

  it('supports structured reasons for retention, webdav secret and webdav list errors', () => {
    const retentionErr = new ApiError(400, 'Invalid retention: keep_days must be > 0', {
      error: 'invalid_retention',
      message: 'Invalid retention: keep_days must be > 0',
      details: { reason: 'invalid_policy', field: 'retention' },
    })
    const retentionInfo = toApiErrorInfo(retentionErr, t)
    expect(retentionInfo.message).toBe('Retention policy is invalid')

    const missingSecretErr = new ApiError(400, 'WebDAV credential not found', {
      error: 'missing_webdav_secret',
      message: 'WebDAV credential not found',
      details: { reason: 'not_found', field: 'destination.secret_name' },
    })
    const missingSecretInfo = toApiErrorInfo(missingSecretErr, t)
    expect(missingSecretInfo.message).toBe('WebDAV credential not found')

    const webdavErr = new ApiError(400, 'builder error', {
      error: 'webdav_list_failed',
      message: 'builder error',
      details: { reason: 'client_init_failed', field: 'base_url' },
    })
    const webdavInfo = toApiErrorInfo(webdavErr, t)
    expect(webdavInfo.message).toBe('Failed to initialize WebDAV client')
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
