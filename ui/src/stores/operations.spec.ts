import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useAuthStore } from './auth'
import { useOperationsStore } from './operations'

describe('useOperationsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('starts restore with CSRF header and JSON body', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ op_id: 'op-1' }), { status: 200, headers: { 'Content-Type': 'application/json' } }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const ops = useOperationsStore()
    const opId = await ops.startRestore('run-1', '/tmp/restore', 'overwrite')

    expect(opId).toBe('op-1')
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/runs/run-1/restore',
      expect.objectContaining({ credentials: 'include', method: 'POST' }),
    )

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(headers['Content-Type']).toBe('application/json')
    expect(init.body).toBe(JSON.stringify({ destination_dir: '/tmp/restore', conflict_policy: 'overwrite' }))
  })

  it('starts verify with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ op_id: 'op-2' }), { status: 200, headers: { 'Content-Type': 'application/json' } }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-xyz'

    const ops = useOperationsStore()
    const opId = await ops.startVerify('run-2')

    expect(opId).toBe('op-2')
    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-xyz')
  })
})

