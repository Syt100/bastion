import type { RouteLocationRaw } from 'vue-router'

import type { RunStatus } from '@/stores/jobs'
import type { ScopeValue } from '@/lib/scope'

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

export function runKindLabel(t: TranslateFn, kind: string | null | undefined): string {
  if (!kind) return t('runs.kinds.backup')
  const key = `runs.kinds.${kind}`
  const v = t(key)
  return v === key ? kind : v
}

export function runFailureStageLabel(t: TranslateFn, stage: string | null | undefined): string {
  if (!stage) return '-'
  const key = `runs.progress.stages.${stage}`
  const v = t(key)
  return v === key ? stage : v
}

export function buildRunDetailLocation(
  runId: string,
  options: {
    fromScope?: ScopeValue | string | null
    fromJob?: string | null
    fromSection?: 'overview' | 'history' | 'data' | string | null
  } = {},
): RouteLocationRaw {
  const query: Record<string, string> = {}
  if (options.fromScope) query.from_scope = String(options.fromScope)
  if (options.fromJob) query.from_job = options.fromJob
  if (options.fromSection) query.from_section = String(options.fromSection)
  return {
    path: `/runs/${encodeURIComponent(runId)}`,
    query,
  }
}
