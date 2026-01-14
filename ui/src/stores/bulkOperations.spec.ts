import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { useBulkOperationsStore } from './bulkOperations'
import { useAuthStore } from './auth'

describe('useBulkOperationsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('lists bulk operations', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify([]), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)

    const bulk = useBulkOperationsStore()
    await bulk.list()

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/bulk-operations',
      expect.objectContaining({ credentials: 'include' }),
    )
  })

  it('creates bulk operation with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ op_id: 'op1' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const bulk = useBulkOperationsStore()
    const id = await bulk.create({
      kind: 'agent_labels_add',
      selector: { node_ids: ['a'] },
      payload: { labels: ['prod'] },
    })
    expect(id).toBe('op1')

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    expect(init.method).toBe('POST')
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
  })

  it('cancels bulk operation with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const bulk = useBulkOperationsStore()
    await bulk.cancel('op 1')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/bulk-operations/op%201/cancel',
      expect.objectContaining({ credentials: 'include' }),
    )
    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
  })

  it('retries failed items with CSRF header', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-123'

    const bulk = useBulkOperationsStore()
    await bulk.retryFailed('op 1')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/bulk-operations/op%201/retry-failed',
      expect.objectContaining({ credentials: 'include' }),
    )
    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
  })
})

