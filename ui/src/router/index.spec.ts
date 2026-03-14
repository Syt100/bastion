// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import router from './index'

describe('router web ui redesign routes', () => {
  it('resolves the new command center landing route', () => {
    const resolved = router.resolve('/')
    expect(resolved.meta.titleKey).toBe('commandCenter.title')
    expect(resolved.meta.primaryNav).toBe('command-center')
  })

  it('resolves the new top-level runs detail route', () => {
    const resolved = router.resolve('/runs/00000000-0000-0000-0000-000000000000')
    expect(resolved.params.runId).toBe('00000000-0000-0000-0000-000000000000')
    expect(resolved.meta.titleKey).toBe('runs.detail.pageTitle')
    expect(resolved.meta.primaryNav).toBe('runs')
  })

  it('resolves the canonical integrations storage route', () => {
    const resolved = router.resolve('/integrations/storage')
    expect(resolved.meta.titleKey).toBe('settings.menu.storage')
    expect(resolved.meta.primaryNav).toBe('integrations')
  })

  it('keeps legacy node-scoped jobs routes available during migration', () => {
    const resolved = router.resolve('/n/hub/jobs/job1/history/runs/00000000-0000-0000-0000-000000000000')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.params.jobId).toBe('job1')
    expect(resolved.params.runId).toBe('00000000-0000-0000-0000-000000000000')
    expect(resolved.meta.primaryNav).toBe('jobs')
  })

  it('keeps legacy node-scoped storage routes available during migration', () => {
    const resolved = router.resolve('/n/hub/settings/storage')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.meta.titleKey).toBe('settings.menu.storage')
    expect(resolved.meta.primaryNav).toBe('integrations')
  })
})
