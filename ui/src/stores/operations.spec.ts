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
    const opId = await ops.startRestore(
      'run-1',
      { type: 'local_fs', node_id: 'hub', directory: '/tmp/restore' },
      'overwrite',
    )

    expect(opId).toBe('op-1')
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/runs/run-1/restore',
      expect.objectContaining({ credentials: 'include', method: 'POST' }),
    )

    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-123')
    expect(headers['Content-Type']).toBe('application/json')
    expect(init.body).toBe(
      JSON.stringify({
        destination: { type: 'local_fs', node_id: 'hub', directory: '/tmp/restore' },
        conflict_policy: 'overwrite',
      }),
    )
  })

  it('starts restore with selection when provided', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ op_id: 'op-3' }), { status: 200, headers: { 'Content-Type': 'application/json' } }),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-abc'

    const ops = useOperationsStore()
    const opId = await ops.startRestore(
      'run-1',
      { type: 'local_fs', node_id: 'hub', directory: '/tmp/restore' },
      'overwrite',
      {
        files: ['a', ' a ', ''],
        dirs: ['b/', 'b', ''],
      },
    )

    expect(opId).toBe('op-3')
    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    expect(init.body).toBe(
      JSON.stringify({
        destination: { type: 'local_fs', node_id: 'hub', directory: '/tmp/restore' },
        conflict_policy: 'overwrite',
        selection: { files: ['a'], dirs: ['b'] },
      }),
    )
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

  it('cancels operation with CSRF header and JSON body', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          id: 'op-9',
          kind: 'restore',
          status: 'running',
          created_at: 1,
          started_at: 1,
          ended_at: null,
          cancel_requested_at: 2,
          summary: null,
          error: null,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-777'

    const ops = useOperationsStore()
    const op = await ops.cancelOperation('op 9', ' stop ')

    expect(op.cancel_requested_at).toBe(2)
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/operations/op%209/cancel',
      expect.objectContaining({ credentials: 'include', method: 'POST' }),
    )
    const init = fetchMock.mock.calls[0]?.[1] as RequestInit
    const headers = init.headers as Record<string, string>
    expect(headers['X-CSRF-Token']).toBe('csrf-777')
    expect(headers['Content-Type']).toBe('application/json')
    expect(init.body).toBe(JSON.stringify({ reason: 'stop' }))
  })

  it('dedupes concurrent cancelOperation requests for the same operation', async () => {
    let resolve: (value: Response | PromiseLike<Response>) => void = () => undefined
    const pending = new Promise<Response>((res) => {
      resolve = res
    })
    const fetchMock = vi.fn().mockImplementation(() => pending)
    vi.stubGlobal('fetch', fetchMock)

    const auth = useAuthStore()
    auth.status = 'authenticated'
    auth.csrfToken = 'csrf-999'

    const ops = useOperationsStore()
    const first = ops.cancelOperation('op-1')
    const second = ops.cancelOperation('op-1')
    await Promise.resolve()

    expect(fetchMock).toHaveBeenCalledTimes(1)

    resolve(
      new Response(
        JSON.stringify({
          id: 'op-1',
          kind: 'verify',
          status: 'running',
          created_at: 1,
          started_at: 1,
          ended_at: null,
          cancel_requested_at: 8,
          summary: null,
          error: null,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )

    const [a, b] = await Promise.all([first, second])
    expect(a.cancel_requested_at).toBe(8)
    expect(b.cancel_requested_at).toBe(8)
  })
})
