import { describe, expect, it } from 'vitest'

import { bulkOperationItemStatusLabel, bulkOperationKindLabel, bulkOperationStatusLabel, filterBulkOperationItems } from './bulkOperations'

describe('bulk operations labels', () => {
  it('falls back to raw values when translations are missing', () => {
    const t = (key: string) => key
    expect(bulkOperationKindLabel(t, 'sync_config_now')).toBe('sync_config_now')
    expect(bulkOperationStatusLabel(t, 'queued')).toBe('queued')
    expect(bulkOperationItemStatusLabel(t, 'failed')).toBe('failed')
  })

  it('returns translated labels when available', () => {
    const t = (key: string) => {
      if (key === 'bulk.kinds.sync_config_now') return 'Sync config'
      if (key === 'bulk.statuses.queued') return 'Queued'
      if (key === 'bulk.itemStatuses.failed') return 'Failed'
      return key
    }
    expect(bulkOperationKindLabel(t, 'sync_config_now')).toBe('Sync config')
    expect(bulkOperationStatusLabel(t, 'queued')).toBe('Queued')
    expect(bulkOperationItemStatusLabel(t, 'failed')).toBe('Failed')
  })
})

describe('bulk operations filters', () => {
  it('filters failed items', () => {
    const items = [
      {
        op_id: 'op1',
        agent_id: 'a1',
        agent_name: null,
        status: 'failed',
        attempts: 1,
        created_at: 1,
        updated_at: 1,
        started_at: 1,
        ended_at: 1,
        last_error_kind: 'error',
        last_error: 'nope',
      },
      {
        op_id: 'op1',
        agent_id: 'a2',
        agent_name: null,
        status: 'success',
        attempts: 1,
        created_at: 1,
        updated_at: 1,
        started_at: 1,
        ended_at: 1,
        last_error_kind: null,
        last_error: null,
      },
    ] as const

    expect(filterBulkOperationItems([...items], 'all').length).toBe(2)
    const failed = filterBulkOperationItems([...items], 'failed')
    expect(failed.length).toBe(1)
    expect(failed[0]!.agent_id).toBe('a1')
  })
})
