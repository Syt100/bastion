import { defineStore } from 'pinia'
import { ref } from 'vue'

import { apiFetch } from '@/lib/api'
import { createLatestRequest } from '@/lib/latest'
import type { ScopeValue } from '@/lib/scope'

export type CommandCenterRangePreset = '24h' | '7d' | '30d'
export type CommandCenterSectionState = 'ready' | 'empty' | 'degraded'
export type CommandCenterSeverity = 'critical' | 'warning' | 'info'
export type CommandCenterReadinessOverall = 'healthy' | 'degraded' | 'empty'

export type CommandCenterAction = {
  label: string
  href: string
}

export type CommandCenterItemContext = {
  run_id?: string | null
  job_id?: string | null
  job_name?: string | null
  node_id?: string | null
  node_name?: string | null
  operation_id?: string | null
  notification_id?: string | null
  channel?: string | null
  status?: string | null
  error?: string | null
}

export type CommandCenterItem = {
  id: string
  kind: string
  severity: CommandCenterSeverity
  title: string
  summary: string
  occurred_at: number
  scope: ScopeValue | string
  context: CommandCenterItemContext
  primary_action: CommandCenterAction
  secondary_action?: CommandCenterAction | null
}

export type CommandCenterSection = {
  state: CommandCenterSectionState
  items: CommandCenterItem[]
  note?: string | null
}

export type CommandCenterReadinessSignal = {
  recent_success_at: number | null
  recent_run_id?: string | null
  recent_job_id?: string | null
  recent_job_name?: string | null
  recent_operation_id?: string | null
  active_jobs: number
  covered_jobs: number
}

export type CommandCenterReadinessBlocker = {
  kind: string
  title: string
  summary: string
  href: string
}

export type CommandCenterSnapshot = {
  generated_at: number
  scope: {
    requested: string
    effective: string
  }
  range: {
    preset: CommandCenterRangePreset
    from: number
    to: number
  }
  attention: CommandCenterSection
  critical_activity: CommandCenterSection
  recovery_readiness: {
    state: CommandCenterSectionState
    overall: CommandCenterReadinessOverall
    backup: CommandCenterReadinessSignal
    verify: CommandCenterReadinessSignal
    blockers: CommandCenterReadinessBlocker[]
  }
  watchlist: CommandCenterSection
}

export const useCommandCenterStore = defineStore('commandCenter', () => {
  const loading = ref<boolean>(false)
  const snapshot = ref<CommandCenterSnapshot | null>(null)
  const latestRefresh = createLatestRequest()

  async function refresh(params?: {
    scope?: ScopeValue | string
    range?: CommandCenterRangePreset
  }): Promise<void> {
    const current = latestRefresh.next()
    loading.value = true
    try {
      const query = new URLSearchParams()
      if (params?.scope) query.set('scope', params.scope)
      if (params?.range) query.set('range', params.range)
      const suffix = query.toString() ? `?${query.toString()}` : ''
      const nextSnapshot = await apiFetch<CommandCenterSnapshot>(`/api/command-center${suffix}`, {
        signal: current.signal,
      })
      if (current.isStale() || current.signal.aborted) return
      snapshot.value = nextSnapshot
    } catch (error) {
      if (current.isStale() || current.signal.aborted) return
      throw error
    } finally {
      if (!current.isStale()) {
        loading.value = false
        current.finish()
      }
    }
  }

  return {
    loading,
    snapshot,
    refresh,
  }
})
