import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'
import { appendQueryTextParam, buildQuerySuffix } from '@/lib/listQuery'

export type IntegrationsDomainState = 'ready' | 'empty' | 'degraded'
export type StorageHealthState = 'healthy' | 'attention' | 'progressing' | 'configured' | 'unused'
export type DistributionConnectionState = 'online' | 'offline'
export type DistributionScopeState = 'covered' | 'drifted' | 'failed'

export type StorageSummary = {
  items_total: number
  in_use_total: number
  invalid_total: number
}

export type NotificationsSummary = {
  destinations_total: number
  recent_failures_total: number
  queue_backlog_total: number
}

export type DistributionSummary = {
  coverage_total: number
  drifted_total: number
  failed_total: number
  offline_total: number
}

export type IntegrationsDomainSummary<T> = {
  state: IntegrationsDomainState
  summary: T
}

export type IntegrationsSummaryResponse = {
  storage: IntegrationsDomainSummary<StorageSummary>
  notifications: IntegrationsDomainSummary<NotificationsSummary>
  distribution: IntegrationsDomainSummary<DistributionSummary>
}

export type StorageUsageRef = {
  job_id: string
  job_name: string
  latest_run_id?: string | null
  latest_run_status?: string | null
  latest_run_at?: number | null
}

export type StorageHealthSummary = {
  state: StorageHealthState
  latest_run_id?: string | null
  latest_run_status?: string | null
  latest_run_at?: number | null
}

export type StorageIntegrationItem = {
  name: string
  updated_at: number
  usage_total: number
  usage: StorageUsageRef[]
  health: StorageHealthSummary
}

export type StorageDetailsResponse = {
  node_id: string
  summary: StorageSummary
  items: StorageIntegrationItem[]
}

export type DistributionScopeItem = {
  agent_id: string
  agent_name?: string | null
  connection_status: DistributionConnectionState
  distribution_state: DistributionScopeState
  pending_tasks_total: number
  last_attempt_at?: number | null
  last_error_kind?: string | null
  last_error?: string | null
}

export type DistributionDetailsResponse = {
  summary: DistributionSummary
  items: DistributionScopeItem[]
}

export const useIntegrationsStore = defineStore('integrations', () => {
  async function getSummary(signal?: AbortSignal): Promise<IntegrationsSummaryResponse> {
    return await apiFetch<IntegrationsSummaryResponse>('/api/integrations', { signal })
  }

  async function getStorage(nodeId: string, signal?: AbortSignal): Promise<StorageDetailsResponse> {
    const q = new URLSearchParams()
    appendQueryTextParam(q, 'node_id', nodeId)
    return await apiFetch<StorageDetailsResponse>('/api/integrations/storage' + buildQuerySuffix(q), { signal })
  }

  async function getDistribution(signal?: AbortSignal): Promise<DistributionDetailsResponse> {
    return await apiFetch<DistributionDetailsResponse>('/api/integrations/distribution', { signal })
  }

  return {
    getSummary,
    getStorage,
    getDistribution,
  }
})
