import type { BulkOperationItemStatus, BulkOperationStatus } from '@/stores/bulkOperations'

type TranslateFn = (key: string, params?: Record<string, unknown>) => string

export function bulkOperationKindLabel(t: TranslateFn, kind: string): string {
  const key = `bulk.kinds.${kind}`
  const v = t(key)
  // Fallback for unknown/new kinds if translations are missing.
  return v === key ? kind : v
}

export function bulkOperationStatusLabel(t: TranslateFn, status: BulkOperationStatus): string {
  const key = `bulk.statuses.${status}`
  const v = t(key)
  // Fallback for unknown/new statuses if translations are missing.
  return v === key ? status : v
}

export function bulkOperationItemStatusLabel(t: TranslateFn, status: BulkOperationItemStatus): string {
  const key = `bulk.itemStatuses.${status}`
  const v = t(key)
  // Fallback for unknown/new statuses if translations are missing.
  return v === key ? status : v
}

