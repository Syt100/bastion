// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import router from './index'

describe('router node-scoped routes', () => {
  it('resolves /n/:nodeId/jobs', () => {
    const resolved = router.resolve('/n/hub/jobs')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.meta.titleKey).toBe('jobs.title')
  })

  it('resolves /n/:nodeId/jobs/:jobId/overview', () => {
    const resolved = router.resolve('/n/hub/jobs/job1/overview')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.params.jobId).toBe('job1')
    expect(resolved.meta.titleKey).toBe('jobs.workspace.sections.overview')
  })

  it('resolves /n/:nodeId/jobs/:jobId/history/runs/:runId', () => {
    const resolved = router.resolve('/n/hub/jobs/job1/history/runs/00000000-0000-0000-0000-000000000000')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.params.jobId).toBe('job1')
    expect(resolved.params.runId).toBe('00000000-0000-0000-0000-000000000000')
    expect(resolved.meta.titleKey).toBe('runs.title')
  })

  it('resolves /n/:nodeId/settings/storage', () => {
    const resolved = router.resolve('/n/hub/settings/storage')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.meta.titleKey).toBe('settings.menu.storage')
  })
})
