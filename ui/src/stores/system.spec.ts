import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useSystemStore } from './system'

describe('useSystemStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('refreshes system status', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ version: '0.1.0', build_time_unix: 1730000000, insecure_http: true, hub_timezone: 'UTC' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const system = useSystemStore()
    await system.refresh()

    expect(system.version).toBe('0.1.0')
    expect(system.buildTimeUnix).toBe(1730000000)
    expect(system.insecureHttp).toBe(true)
    expect(system.hubTimezone).toBe('UTC')
  })
})
