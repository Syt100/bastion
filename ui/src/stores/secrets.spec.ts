import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useAuthStore } from './auth'
import { useSecretsStore } from './secrets'

describe('useSecretsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes webdav secrets list', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify([{ name: 'primary', updated_at: 1 }]), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const secrets = useSecretsStore()
    await secrets.refreshWebdav('hub')

    expect(secrets.webdav).toHaveLength(1)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/nodes/hub/secrets/webdav',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('upserts webdav secret with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const secrets = useSecretsStore()
    await secrets.upsertWebdav('hub', 'primary', 'u1', 'p1')

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(init.method).toBe('PUT')
  })

  it('refreshes smtp secrets list', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify([{ name: 'primary', updated_at: 1 }]), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const secrets = useSecretsStore()
    await secrets.refreshSmtp()

    expect(secrets.smtp).toHaveLength(1)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/secrets/smtp',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('upserts smtp secret with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const secrets = useSecretsStore()
    await secrets.upsertSmtp('primary', {
      host: 'smtp.example.com',
      port: 587,
      username: 'u1',
      password: 'p1',
      from: 'NoBody <nobody@example.com>',
      to: ['a@example.com'],
      tls: 'starttls',
    })

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(init.method).toBe('PUT')
  })
})
