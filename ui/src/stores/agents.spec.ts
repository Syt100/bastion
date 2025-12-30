import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useAgentsStore } from './agents'
import { useAuthStore } from './auth'

describe('useAgentsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes the agents list', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify([
          { id: 'a1', name: null, revoked: false, last_seen_at: null, online: false },
        ]),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const agents = useAgentsStore()
    await agents.refresh()

    expect(agents.items).toHaveLength(1)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/agents',
      expect.objectContaining({ credentials: 'include' }),
    )
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
})
