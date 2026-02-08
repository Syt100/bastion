import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useAgentsStore } from './agents'
import { useAuthStore } from './auth'

function deferredResponse() {
  let resolve: (value: Response | PromiseLike<Response>) => void = () => undefined
  let reject: (reason?: unknown) => void = () => undefined
  const promise = new Promise<Response>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

function buildAgentsResponse(id: string, total = 1): Response {
  return new Response(
    JSON.stringify({
      items: [
        {
          id,
          name: null,
          revoked: false,
          last_seen_at: null,
          online: false,
          labels: [],
          desired_config_snapshot_id: null,
          applied_config_snapshot_id: null,
          config_sync_status: 'offline',
          last_config_sync_attempt_at: null,
          last_config_sync_error_kind: null,
          last_config_sync_error: null,
        },
      ],
      page: 1,
      page_size: 20,
      total,
    }),
    { status: 200, headers: { 'Content-Type': 'application/json' } },
  )
}


describe('useAgentsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes the agents list', async () => {
    const fetchMock = vi.fn().mockResolvedValue(buildAgentsResponse('a1'))
    vi.stubGlobal('fetch', fetchMock)

    const agents = useAgentsStore()
    await agents.refresh()

    expect(agents.items).toHaveLength(1)
    expect(agents.total).toBe(1)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/agents',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('ignores stale refresh success responses', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const fetchMock = vi
      .fn()
      .mockImplementationOnce(() => first.promise)
      .mockImplementationOnce(() => second.promise)
    vi.stubGlobal('fetch', fetchMock)

    const agents = useAgentsStore()
    const p1 = agents.refresh()
    const p2 = agents.refresh({ labels: ['prod'] })

    second.resolve(buildAgentsResponse('newer'))
    await p2

    first.resolve(buildAgentsResponse('older'))
    await expect(p1).resolves.toBeUndefined()

    expect(agents.items.map((item) => item.id)).toEqual(['newer'])
    expect(agents.loading).toBe(false)
  })

  it('aborts stale refresh requests when a newer refresh starts', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const signals: AbortSignal[] = []
    const fetchMock = vi.fn().mockImplementation((_input: RequestInfo | URL, init?: RequestInit) => {
      signals.push((init?.signal ?? null) as AbortSignal)
      if (signals.length === 1) return first.promise
      return second.promise
    })
    vi.stubGlobal('fetch', fetchMock)

    const agents = useAgentsStore()
    const p1 = agents.refresh()
    await Promise.resolve()

    expect(signals).toHaveLength(1)
    expect(signals[0]?.aborted).toBe(false)

    const p2 = agents.refresh({ labels: ['prod'] })

    expect(signals).toHaveLength(2)
    expect(signals[0]?.aborted).toBe(true)

    second.resolve(buildAgentsResponse('newer'))
    await p2

    first.reject(new Error('aborted by newer refresh'))
    await expect(p1).resolves.toBeUndefined()

    expect(agents.items.map((item) => item.id)).toEqual(['newer'])
    expect(agents.loading).toBe(false)
  })

  it('ignores stale refresh failures after a newer success', async () => {
    const first = deferredResponse()
    const second = deferredResponse()
    const fetchMock = vi
      .fn()
      .mockImplementationOnce(() => first.promise)
      .mockImplementationOnce(() => second.promise)
    vi.stubGlobal('fetch', fetchMock)

    const agents = useAgentsStore()
    const p1 = agents.refresh()
    const p2 = agents.refresh()

    second.resolve(buildAgentsResponse('stable'))
    await p2

    first.reject(new Error('stale network error'))
    await expect(p1).resolves.toBeUndefined()

    expect(agents.items.map((item) => item.id)).toEqual(['stable'])
    expect(agents.loading).toBe(false)
  })

  it('creates an enrollment token with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ token: 't1', expires_at: 123, remaining_uses: null }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const agents = useAgentsStore()
    const token = await agents.createEnrollmentToken({ ttlSeconds: 60, remainingUses: null })

    expect(token.token).toBe('t1')

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(headers['Content-Type']).toBe('application/json')

    const body = JSON.parse(init.body as string) as Record<string, unknown>
    expect(body.ttl_seconds).toBe(60)
    expect(body.remaining_uses).toBeNull()
  })

  it('revokes an agent with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const agents = useAgentsStore()
    await agents.revokeAgent('a b')

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(init.method).toBe('POST')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/agents/a%20b/revoke',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('rotates an agent key with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ agent_id: 'a1', agent_key: 'k1' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const agents = useAgentsStore()
    const res = await agents.rotateAgentKey('a b')
    expect(res.agent_key).toBe('k1')

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(init.method).toBe('POST')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/agents/a%20b/rotate-key',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('fetches agent detail', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          id: 'a1',
          name: null,
          revoked: false,
          created_at: 1,
          last_seen_at: null,
          online: false,
          capabilities_json: null,
          labels: [],
          desired_config_snapshot_id: null,
          desired_config_snapshot_at: null,
          applied_config_snapshot_id: null,
          applied_config_snapshot_at: null,
          config_sync_status: 'offline',
          last_config_sync_attempt_at: null,
          last_config_sync_error_kind: null,
          last_config_sync_error: null,
          last_config_sync_error_at: null,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const agents = useAgentsStore()
    const detail = await agents.getAgent('a b')

    expect(detail.id).toBe('a1')
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/agents/a%20b',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('syncs config now with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ outcome: 'sent' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const agents = useAgentsStore()
    const res = await agents.syncConfigNow('a b')
    expect(res.outcome).toBe('sent')

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(init.method).toBe('POST')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/agents/a%20b/sync-config-now',
      expect.objectContaining({ credentials: 'include' }),
    )
  })
})
