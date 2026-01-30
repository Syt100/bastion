import { describe, expect, it } from 'vitest'

import { bulkOperationItemStatusLabel, bulkOperationKindLabel, bulkOperationStatusLabel } from './bulkOperations'

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

