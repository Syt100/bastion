import type { RunStatus } from '@/stores/jobs'

type TranslateFn = (key: string, params?: Record<string, unknown>) => string

export function runStatusLabel(t: TranslateFn, status: RunStatus | string): string {
  const key = `runs.statuses.${status}`
  const v = t(key)
  // Fallback for unknown/new statuses if translations are missing.
  return v === key ? status : v
}

export function runTargetTypeLabel(t: TranslateFn, targetType: string | null | undefined): string {
  if (!targetType) return '-'
  if (targetType === 'local_dir') return t('jobs.targets.localDir')
  if (targetType === 'webdav') return t('jobs.targets.webdav')
  // Fallback for unknown/new target types.
  return targetType
}
