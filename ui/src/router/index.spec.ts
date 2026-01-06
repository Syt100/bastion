// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import router from './index'

describe('router node-scoped routes', () => {
  it('resolves /n/:nodeId/jobs', () => {
    const resolved = router.resolve('/n/hub/jobs')
    expect(resolved.params.nodeId).toBe('hub')
    expect(resolved.meta.titleKey).toBe('jobs.title')
  })
})

