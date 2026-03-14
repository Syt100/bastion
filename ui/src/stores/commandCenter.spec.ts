import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useCommandCenterStore } from './commandCenter'

function deferredResponse() {
  let resolve: (value: Response | PromiseLike<Response>) => void = () => undefined
  let reject: (reason?: unknown) => void = () => undefined
  const promise = new Promise<Response>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

function buildCommandCenterResponse(runId: string): Response {
  return new Response(
    JSON.stringify({
      generated_at: 100,
      scope: { requested: 'all', effective: 'all' },
      range: { preset: '24h', from: 1, to: 100 },
      attention: {
        state: 'ready',
        items: [
          {
            id: 'run:r1',
            kind: 'run_failed',
            severity: 'critical',
            title: 'Nightly backup needs review',
            summary: 'upload failed',
            occurred_at: 88,
            scope: 'agent:edge-1',
            context: { run_id: runId, job_id: 'job-1' },
            primary_action: { label: 'Open run', href: `/runs/${runId}` },
          },
        ],
      },
      critical_activity: { state: 'empty', items: [] },
      recovery_readiness: {
        state: 'degraded',
        overall: 'degraded',
        backup: { recent_success_at: 80, active_jobs: 2, covered_jobs: 1 },
        verify: { recent_success_at: null, active_jobs: 2, covered_jobs: 0 },
        blockers: [{ kind: 'missing_verification', title: 'Verification missing', summary: 'No verify', href: '/runs' }],
      },
      watchlist: { state: 'empty', items: [] },
    }),
    { status: 200, headers: { 'Content-Type': 'application/json' } },
  )
}

describe('useCommandCenterStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes command center snapshot', async () => {
    const fetchMock = vi.fn().mockResolvedValue(buildCommandCenterResponse('run-1'))
    vi.stubGlobal('fetch', fetchMock)

    const commandCenter = useCommandCenterStore()
    await commandCenter.refresh({ scope: 'agent:edge-1', range: '24h' })

    expect(commandCenter.snapshot?.attention.items[0]?.context.run_id).toBe('run-1')
    expect(commandCenter.snapshot?.recovery_readiness.overall).toBe('degraded')
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/command-center?scope=agent%3Aedge-1&range=24h',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('aborts stale refresh when a newer request starts', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const signals: AbortSignal[] = []
    const fetchMock = vi.fn().mockImplementation((_input: RequestInfo | URL, init?: RequestInit) => {
      signals.push((init?.signal ?? null) as AbortSignal)
      if (signals.length === 1) return first.promise
      return second.promise
    })
    vi.stubGlobal('fetch', fetchMock)

    const commandCenter = useCommandCenterStore()
    const p1 = commandCenter.refresh({ scope: 'all', range: '24h' })
    await Promise.resolve()

    const p2 = commandCenter.refresh({ scope: 'hub', range: '7d' })

    expect(signals).toHaveLength(2)
    expect(signals[0]?.aborted).toBe(true)

    second.resolve(buildCommandCenterResponse('run-newer'))
    await p2

    first.reject(new Error('aborted by newer refresh'))
    await expect(p1).resolves.toBeUndefined()

    expect(commandCenter.snapshot?.attention.items[0]?.context.run_id).toBe('run-newer')
    expect(commandCenter.loading).toBe(false)
  })
})
