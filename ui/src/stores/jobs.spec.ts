import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useAuthStore } from './auth'
import { useJobsStore } from './jobs'

function deferredResponse() {
  let resolve: (value: Response | PromiseLike<Response>) => void = () => undefined
  let reject: (reason?: unknown) => void = () => undefined
  const promise = new Promise<Response>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

function buildJobsResponse(id: string): Response {
  return new Response(
    JSON.stringify([
      {
        id,
        name: id,
        agent_id: null,
        schedule: null,
        schedule_timezone: 'UTC',
        overlap_policy: 'queue',
        created_at: 1,
        updated_at: 1,
      },
    ]),
    { status: 200, headers: { 'Content-Type': 'application/json' } },
  )
}

describe('useJobsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes jobs list', async () => {
    const fetchMock = vi.fn().mockResolvedValue(buildJobsResponse('j1'))
    vi.stubGlobal('fetch', fetchMock)

    const jobs = useJobsStore()
    await jobs.refresh()

    expect(jobs.items).toHaveLength(1)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/jobs',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('ignores stale jobs refresh success responses', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const fetchMock = vi
      .fn()
      .mockImplementationOnce(() => first.promise)
      .mockImplementationOnce(() => second.promise)
    vi.stubGlobal('fetch', fetchMock)

    const jobs = useJobsStore()
    const p1 = jobs.refresh()
    const p2 = jobs.refresh({ includeArchived: true })

    second.resolve(buildJobsResponse('newer'))
    await p2

    first.resolve(buildJobsResponse('older'))
    await expect(p1).resolves.toBeUndefined()

    expect(jobs.items.map((item) => item.id)).toEqual(['newer'])
    expect(jobs.loading).toBe(false)
  })

  it('aborts stale jobs refresh requests when a newer refresh starts', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const signals: AbortSignal[] = []
    const fetchMock = vi.fn().mockImplementation((_input: RequestInfo | URL, init?: RequestInit) => {
      signals.push((init?.signal ?? null) as AbortSignal)
      if (signals.length === 1) return first.promise
      return second.promise
    })
    vi.stubGlobal('fetch', fetchMock)

    const jobs = useJobsStore()
    const p1 = jobs.refresh()
    await Promise.resolve()

    expect(signals).toHaveLength(1)
    expect(signals[0]?.aborted).toBe(false)

    const p2 = jobs.refresh({ includeArchived: true })

    expect(signals).toHaveLength(2)
    expect(signals[0]?.aborted).toBe(true)

    second.resolve(buildJobsResponse('newer'))
    await p2

    first.reject(new Error('aborted by newer refresh'))
    await expect(p1).resolves.toBeUndefined()

    expect(jobs.items.map((item) => item.id)).toEqual(['newer'])
    expect(jobs.loading).toBe(false)
  })

  it('ignores stale jobs refresh failures after a newer success', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const fetchMock = vi
      .fn()
      .mockImplementationOnce(() => first.promise)
      .mockImplementationOnce(() => second.promise)
    vi.stubGlobal('fetch', fetchMock)

    const jobs = useJobsStore()
    const p1 = jobs.refresh()
    const p2 = jobs.refresh()

    second.resolve(buildJobsResponse('stable'))
    await p2

    first.reject(new Error('stale network error'))
    await expect(p1).resolves.toBeUndefined()

    expect(jobs.items.map((item) => item.id)).toEqual(['stable'])
    expect(jobs.loading).toBe(false)
  })

  it('creates job with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          id: 'j1',
          name: 'job1',
          agent_id: null,
          schedule: null,
          schedule_timezone: 'UTC',
          overlap_policy: 'queue',
          spec: { v: 1, type: 'filesystem' },
          created_at: 1,
          updated_at: 1,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const jobs = useJobsStore()
    await jobs.createJob({
      name: 'job1',
      agent_id: null,
      schedule: null,
      schedule_timezone: 'UTC',
      overlap_policy: 'queue',
      spec: { v: 1, type: 'filesystem' },
    })

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(headers['Content-Type']).toBe('application/json')
  })

  it('lists run events', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify([
          {
            run_id: 'r1',
            seq: 1,
            ts: 1,
            level: 'info',
            kind: 'start',
            message: 'start',
            fields: null,
          },
        ]),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const jobs = useJobsStore()
    const events = await jobs.listRunEvents('r1')

    expect(events).toHaveLength(1)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/runs/r1/events',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('previews retention without CSRF', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          retention: { enabled: true, keep_last: 1 },
          keep_total: 1,
          delete_total: 0,
          keep: [],
          delete: [],
          scan_truncated: false,
          result_truncated: false,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const jobs = useJobsStore()
    await jobs.previewJobRetention('j1', { enabled: true, keep_last: 1 })

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    expect(init.method).toBe('POST')
    expect(init.headers).toEqual(expect.objectContaining({ 'Content-Type': 'application/json' }))
    expect(init.body).toBe(JSON.stringify({ retention: { enabled: true, keep_last: 1 } }))
  })

  it('applies retention with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ enqueued: ['r1'], already_exists: 0, skipped_due_to_limits: 0 }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const jobs = useJobsStore()
    await jobs.applyJobRetention('j1', { enabled: true, keep_last: 1 })

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(init.method).toBe('POST')
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(headers['Content-Type']).toBe('application/json')
  })
})
