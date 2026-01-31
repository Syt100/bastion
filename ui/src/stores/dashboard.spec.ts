import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useDashboardStore } from './dashboard'

describe('useDashboardStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes dashboard overview', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          stats: {
            agents: { total: 2, active: 2, online: 1, offline: 1, revoked: 0 },
            jobs: { active: 3, archived: 1 },
            runs: { running: 1, queued: 2, success_24h: 5, failed_24h: 1, rejected_24h: 0 },
          },
          trend_7d: [{ day: '2026-01-31', success: 1, failed: 0 }],
          recent_runs: [
            {
              run_id: 'r1',
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
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const dashboard = useDashboardStore()
    await dashboard.refresh()

    expect(dashboard.overview?.stats.agents.online).toBe(1)
    expect(dashboard.overview?.recent_runs[0]?.run_id).toBe('r1')
  })
})

