import { describe, expect, it } from 'vitest'

import { isNodeScopedPath, nodeJobsPath, nodeScopedPath, nodeStoragePath, stripNodeScope } from './nodeRoute'

describe('nodeRoute helpers', () => {
  it('builds node-scoped paths with encoded node id', () => {
    expect(nodeJobsPath('agent/a b')).toBe('/n/agent%2Fa%20b/jobs')
    expect(nodeStoragePath('hub')).toBe('/n/hub/settings/storage')
    expect(nodeScopedPath('n-1', '/jobs/detail')).toBe('/n/n-1/jobs/detail')
  })

  it('strips node prefix for scoped routes only', () => {
    expect(isNodeScopedPath('/n/hub/jobs')).toBe(true)
    expect(isNodeScopedPath('/settings')).toBe(false)
    expect(stripNodeScope('/n/agent-1/jobs')).toBe('/jobs')
    expect(stripNodeScope('/n/agent-1')).toBe('/')
    expect(stripNodeScope('/settings/storage')).toBe('/settings/storage')
  })
})
