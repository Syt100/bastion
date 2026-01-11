import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useAuthStore } from './auth'
import { useNotificationsStore } from './notifications'

describe('useNotificationsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes notification settings', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          enabled: true,
          channels: { wecom_bot: { enabled: true }, email: { enabled: true } },
          templates: { wecom_markdown: 'w', email_subject: 's', email_body: 'b' },
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const store = useNotificationsStore()
    await store.refreshSettings()

    expect(store.settings?.enabled).toBe(true)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/notifications/settings',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('saves notification settings with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const store = useNotificationsStore()
    await store.saveSettings({
      enabled: true,
      channels: { wecom_bot: { enabled: true }, email: { enabled: false } },
      templates: { wecom_markdown: 'w', email_subject: 's', email_body: 'b' },
    })

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(init.method).toBe('PUT')
  })

  it('lists notification queue with query params', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          items: [],
          page: 1,
          page_size: 20,
          total: 0,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const store = useNotificationsStore()
    await store.listQueue({ status: 'failed', channel: 'email', page: 2, pageSize: 10 })

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/notifications/queue?status%5B%5D=failed&channel%5B%5D=email&page=2&page_size=10',
      expect.objectContaining({ credentials: 'include' }),
    )
  })
})
