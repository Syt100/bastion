import type { OperationKind, OperationStatus } from '@/stores/operations'

type TranslateFn = (key: string, params?: Record<string, unknown>) => string

export function operationKindLabel(t: TranslateFn, kind: OperationKind): string {
  const key = `operations.kinds.${kind}`
  const v = t(key)
  // Fallback for unknown/new kinds if translations are missing.
  return v === key ? kind : v
}

export function operationStatusLabel(t: TranslateFn, status: OperationStatus): string {
  const key = `operations.statuses.${status}`
  const v = t(key)
  // Fallback for unknown/new statuses if translations are missing.
  return v === key ? status : v
}

