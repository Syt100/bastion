import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useAuthStore } from './auth'
import { useHubRuntimeConfigStore } from './hubRuntimeConfig'

describe('useHubRuntimeConfigStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('gets hub runtime config', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          requires_restart: true,
          effective: {
            bind_host: '0.0.0.0',
            bind_port: 3000,
            data_dir: '/data',
            insecure_http: false,
            trusted_proxies: [],
            debug_errors: false,
            hub_timezone: 'UTC',
            run_retention_days: 30,
            incomplete_cleanup_days: 7,
            log_filter: 'info',
            log_file: null,
            log_rotation: 'daily',
            log_keep_files: 30,
          },
          saved: {
            hub_timezone: null,
            run_retention_days: null,
            incomplete_cleanup_days: null,
            log_filter: null,
            log_file: null,
            log_rotation: null,
            log_keep_files: null,
          },
          fields: {
            bind_host: { env: 'BASTION_HOST', source: 'default', editable: false },
            bind_port: { env: 'BASTION_PORT', source: 'default', editable: false },
            data_dir: { env: 'BASTION_DATA_DIR', source: 'default', editable: false },
            insecure_http: { env: 'BASTION_INSECURE_HTTP', source: 'default', editable: false },
            trusted_proxies: { env: 'BASTION_TRUSTED_PROXIES', source: 'default', editable: false },
            debug_errors: { env: 'BASTION_DEBUG_ERRORS', source: 'default', editable: false },
            hub_timezone: { env: 'BASTION_HUB_TIMEZONE', source: 'default', editable: true },
            run_retention_days: { env: 'BASTION_RUN_RETENTION_DAYS', source: 'default', editable: true },
            incomplete_cleanup_days: { env: 'BASTION_INCOMPLETE_CLEANUP_DAYS', source: 'default', editable: true },
            log_filter: { env: 'BASTION_LOG / RUST_LOG', source: 'default', editable: true },
            log_file: { env: 'BASTION_LOG_FILE', source: 'default', editable: true },
            log_rotation: { env: 'BASTION_LOG_ROTATION', source: 'default', editable: true },
            log_keep_files: { env: 'BASTION_LOG_KEEP_FILES', source: 'default', editable: true },
          },
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const store = useHubRuntimeConfigStore()
    const resp = await store.get()

    expect(resp.requires_restart).toBe(true)
    expect(resp.effective.hub_timezone).toBe('UTC')
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/settings/hub-runtime-config',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('saves hub runtime config with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const store = useHubRuntimeConfigStore()
    await store.save({ hub_timezone: 'UTC', run_retention_days: 30 })

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(init.method).toBe('PUT')
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(headers['Content-Type']).toBe('application/json')
  })
})

