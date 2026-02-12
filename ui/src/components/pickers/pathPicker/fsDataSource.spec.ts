import { beforeEach, describe, expect, it, vi } from 'vitest'

import { ApiError, apiFetch } from '@/lib/api'

import { fsPickerDataSource } from './fsDataSource'

vi.mock('@/lib/api', async () => {
  const actual = await vi.importActual<typeof import('@/lib/api')>('@/lib/api')
  return {
    ...actual,
    apiFetch: vi.fn(),
  }
})

const mockedApiFetch = vi.mocked(apiFetch)

describe('fsPickerDataSource', () => {
  beforeEach(() => {
    mockedApiFetch.mockReset()
  })

  it('classifies agent_fs_list_failed by structured agent error code only', () => {
    const legacyMessageErr = new ApiError(400, 'not a directory', {
      error: 'agent_fs_list_failed',
      message: 'not a directory',
    })
    expect(fsPickerDataSource.mapError(legacyMessageErr, 'node-1').kind).toBe('error')

    const structuredErr = new ApiError(400, 'agent list failed', {
      error: 'agent_fs_list_failed',
      message: 'agent list failed',
      details: { agent_error_code: 'permission_denied' },
    })
    expect(fsPickerDataSource.mapError(structuredErr, 'node-1').kind).toBe('permission_denied')
  })

  it('does not trigger parent fallback from message substrings', async () => {
    mockedApiFetch.mockRejectedValueOnce(
      new ApiError(400, 'not a directory', {
        error: 'agent_fs_list_failed',
        message: 'not a directory',
      }),
    )

    await expect(
      fsPickerDataSource.list('node-1', {
        path: '/tmp/file.txt',
        cursor: null,
        limit: 50,
        mode: 'multi_paths',
      }),
    ).rejects.toBeInstanceOf(ApiError)

    expect(mockedApiFetch).toHaveBeenCalledTimes(1)
  })

  it('falls back to parent list when structured not_directory is present', async () => {
    mockedApiFetch
      .mockRejectedValueOnce(
        new ApiError(400, 'agent list failed', {
          error: 'agent_fs_list_failed',
          message: 'agent list failed',
          details: { agent_error_code: 'not_directory' },
        }),
      )
      .mockResolvedValueOnce({
        path: '/tmp',
        entries: [],
        next_cursor: null,
        total: 0,
      })

    const result = await fsPickerDataSource.list('node-1', {
      path: '/tmp/file.txt',
      cursor: null,
      limit: 50,
      mode: 'multi_paths',
    })

    expect(result.path).toBe('/tmp')
    expect(mockedApiFetch).toHaveBeenCalledTimes(2)

    const secondCall = mockedApiFetch.mock.calls[1]
    expect(secondCall).toBeDefined()
    const secondCallUrl = String(secondCall?.[0] ?? '')
    expect(secondCallUrl).toContain('path=%2Ftmp')
  })
})
