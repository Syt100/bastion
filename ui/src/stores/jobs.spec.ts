import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useAuthStore } from './auth'
import { useJobsStore } from './jobs'

describe('useJobsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes jobs list', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify([
          {
            id: 'j1',
            name: 'job1',
            agent_id: null,
            schedule: null,
            overlap_policy: 'queue',
            created_at: 1,
            updated_at: 1,
          },
        ]),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const jobs = useJobsStore()
    await jobs.refresh()

    expect(jobs.items).toHaveLength(1)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/jobs',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('creates job with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          id: 'j1',
          name: 'job1',
          agent_id: null,
          schedule: null,
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
})
