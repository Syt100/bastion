// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import router from './index'

describe('router node-scoped routes', () => {
  it('resolves /n/:nodeId/jobs', () => {
    const resolved = router.resolve('/n/hub/jobs')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.meta.titleKey).toBe('jobs.title')
  })

  it('resolves /n/:nodeId/jobs/:jobId', () => {
    const resolved = router.resolve('/n/hub/jobs/job1')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.params.jobId).toBe('job1')
    // Job detail runs tab.
    expect(resolved.meta.titleKey).toBe('runs.title')
  })

  it('resolves /n/:nodeId/jobs/:jobId/snapshots', () => {
    const resolved = router.resolve('/n/hub/jobs/job1/snapshots')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.params.jobId).toBe('job1')
    expect(resolved.meta.titleKey).toBe('snapshots.title')
  })

  it('resolves /n/:nodeId/runs/:runId', () => {
    const resolved = router.resolve('/n/hub/runs/00000000-0000-0000-0000-000000000000')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.params.runId).toBe('00000000-0000-0000-0000-000000000000')
    expect(resolved.meta.titleKey).toBe('runs.title')
  })

  it('resolves /n/:nodeId/settings/storage', () => {
    const resolved = router.resolve('/n/hub/settings/storage')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.meta.titleKey).toBe('settings.menu.storage')
  })
})
