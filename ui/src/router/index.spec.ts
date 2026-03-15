// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'

const authStore = {
  status: 'authenticated' as 'authenticated' | 'unknown',
  isAuthenticated: true,
  refreshSession: vi.fn(),
}

vi.mock('@/pinia', () => ({
  pinia: {},
}))

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => authStore,
}))

async function loadRouter() {
  const mod = await import('./index')
  return mod.default
}

describe('router run workspace routes', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.resetModules()
    authStore.status = 'authenticated'
    authStore.isAuthenticated = true
  })

  it('resolves the top-level runs detail route with the runs primary nav', async () => {
    const router = await loadRouter()
    const resolved = router.resolve('/runs/00000000-0000-0000-0000-000000000000')
    expect(resolved.params.runId).toBe('00000000-0000-0000-0000-000000000000')
    expect(resolved.meta.titleKey).toBe('runs.detail.pageTitle')
    expect(resolved.meta.primaryNav).toBe('runs')
  })

  it('normalizes nested job run routes to the canonical run detail route', async () => {
    const router = await loadRouter()
    const resolved = router.resolve('/jobs/job1/history/runs/00000000-0000-0000-0000-000000000000?scope=agent%3Adb-1')
    const redirect = resolved.matched[resolved.matched.length - 1]?.redirect
    expect(typeof redirect).toBe('function')

    const next = typeof redirect === 'function' ? (redirect as (...args: unknown[]) => unknown)(resolved) : redirect
    expect(next).toEqual({
      path: '/runs/00000000-0000-0000-0000-000000000000',
      query: {
        from_scope: 'agent:db-1',
        from_job: 'job1',
        from_section: 'history',
      },
    })
  })

  it('normalizes temporary legacy node-scoped run routes during migration', async () => {
    const router = await loadRouter()
    const resolved = router.resolve('/n/hub/jobs/job1/history/runs/00000000-0000-0000-0000-000000000000')
    const redirect = resolved.matched[resolved.matched.length - 1]?.redirect
    expect(typeof redirect).toBe('function')

    const next = typeof redirect === 'function' ? (redirect as (...args: unknown[]) => unknown)(resolved) : redirect
    expect(next).toEqual({
      path: '/runs/00000000-0000-0000-0000-000000000000',
      query: {
        from_scope: 'hub',
        from_job: 'job1',
        from_section: 'history',
      },
    })
  })
})
