import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useDashboardStore } from './dashboard'

function deferredResponse() {
  let resolve: (value: Response | PromiseLike<Response>) => void = () => undefined
  let reject: (reason?: unknown) => void = () => undefined
  const promise = new Promise<Response>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

function buildDashboardResponse(runId: string): Response {
  return new Response(
    JSON.stringify({
      stats: {
        agents: { total: 2, active: 2, online: 1, offline: 1, revoked: 0 },
        jobs: { active: 3, archived: 1 },
        runs: { running: 1, queued: 2, success_24h: 5, failed_24h: 1, rejected_24h: 0 },
        notifications: { queued: 0, sending: 0, failed: 0, canceled: 0 },
      },
      trend_7d: [{ day: '2026-01-31', success: 1, failed: 0 }],
      recent_runs: [
        {
          run_id: runId,
          job_id: 'j1',
          job_name: 'Job 1',
          node_id: 'hub',
          status: 'success',
          started_at: 100,
          ended_at: 200,
          executed_offline: false,
        },
      ],
    }),
    { status: 200, headers: { 'Content-Type': 'application/json' } },
  )
}

describe('useDashboardStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes dashboard overview', async () => {
    const fetchMock = vi.fn().mockResolvedValue(buildDashboardResponse('r1'))
    vi.stubGlobal('fetch', fetchMock)

    const dashboard = useDashboardStore()
    await dashboard.refresh()

    expect(dashboard.overview?.stats.agents.online).toBe(1)
    expect(dashboard.overview?.recent_runs[0]?.run_id).toBe('r1')
  })

  it('aborts stale dashboard refresh when a newer refresh starts', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const signals: AbortSignal[] = []
    const fetchMock = vi.fn().mockImplementation((_input: RequestInfo | URL, init?: RequestInit) => {
      signals.push((init?.signal ?? null) as AbortSignal)
      if (signals.length === 1) return first.promise
      return second.promise
    })
    vi.stubGlobal('fetch', fetchMock)

    const dashboard = useDashboardStore()
    const p1 = dashboard.refresh()
    await Promise.resolve()

    expect(signals).toHaveLength(1)
    expect(signals[0]?.aborted).toBe(false)

    const p2 = dashboard.refresh()

    expect(signals).toHaveLength(2)
    expect(signals[0]?.aborted).toBe(true)

    second.resolve(buildDashboardResponse('newer'))
    await p2

    first.reject(new Error('aborted by newer refresh'))
    await expect(p1).resolves.toBeUndefined()

    expect(dashboard.overview?.recent_runs[0]?.run_id).toBe('newer')
    expect(dashboard.loading).toBe(false)
  })

  it('ignores stale success response after a newer refresh succeeds', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const fetchMock = vi
      .fn()
      .mockImplementationOnce(() => first.promise)
      .mockImplementationOnce(() => second.promise)
    vi.stubGlobal('fetch', fetchMock)

    const dashboard = useDashboardStore()
    const p1 = dashboard.refresh()
    const p2 = dashboard.refresh()

    second.resolve(buildDashboardResponse('newer'))
    await p2

    first.resolve(buildDashboardResponse('older'))
    await expect(p1).resolves.toBeUndefined()

    expect(dashboard.overview?.recent_runs[0]?.run_id).toBe('newer')
    expect(dashboard.loading).toBe(false)
  })
})
