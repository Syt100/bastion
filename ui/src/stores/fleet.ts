import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'
import { appendPaginationParams, appendQueryArrayParam, appendQueryTextParam, buildQuerySuffix } from '@/lib/listQuery'

export type FleetListStatusFilter = 'all' | 'online' | 'offline' | 'revoked'
export type FleetLabelsMode = 'and' | 'or'

export type FleetSummary = {
  total: number
  online: number
  offline: number
  revoked: number
  drifted: number
}

export type FleetOnboarding = {
  public_base_url?: string | null
  command_generation_ready: boolean
}

export type FleetConfigSync = {
  state: 'synced' | 'pending' | 'error' | 'offline'
  last_error_kind?: string | null
  last_error?: string | null
  last_attempt_at?: number | null
}

export type FleetListItem = {
  id: string
  name?: string | null
  status: 'online' | 'offline' | 'revoked'
  last_seen_at?: number | null
  labels: string[]
  config_sync: FleetConfigSync
  assigned_jobs_total: number
  pending_tasks_total: number
}

export type FleetListResponse = {
  summary: FleetSummary
  onboarding: FleetOnboarding
  items: FleetListItem[]
  page: number
  page_size: number
  total: number
}

export type FleetListFilters = {
  labels?: string[]
  labelsMode?: FleetLabelsMode
  status?: FleetListStatusFilter
  q?: string
  page?: number
  pageSize?: number
}

export type FleetAgentSummary = {
  id: string
  name?: string | null
  status: 'online' | 'offline' | 'revoked'
  created_at: number
  last_seen_at?: number | null
  labels: string[]
}

export type FleetAgentSync = {
  desired_snapshot_id?: string | null
  desired_snapshot_at?: number | null
  applied_snapshot_id?: string | null
  applied_snapshot_at?: number | null
  state: 'synced' | 'pending' | 'error' | 'offline'
  last_error_kind?: string | null
  last_error?: string | null
  last_attempt_at?: number | null
}

export type FleetActivityItem = {
  run_id: string
  job_id: string
  job_name: string
  status: string
  started_at?: number | null
  ended_at?: number | null
}

export type FleetRelatedJob = {
  id: string
  name: string
  schedule?: string | null
  updated_at: number
}

export type FleetCapabilities = {
  can_rotate_key: boolean
  can_revoke: boolean
  can_sync_now: boolean
  can_manage_storage: boolean
}

export type FleetAgentDetailResponse = {
  agent: FleetAgentSummary
  sync: FleetAgentSync
  recent_activity: FleetActivityItem[]
  related_jobs: FleetRelatedJob[]
  capabilities: FleetCapabilities
}

export const useFleetStore = defineStore('fleet', () => {
  async function list(filters?: FleetListFilters, signal?: AbortSignal): Promise<FleetListResponse> {
    const q = new URLSearchParams()
    appendQueryArrayParam(q, 'labels[]', filters?.labels)
    appendQueryTextParam(q, 'labels_mode', filters?.labelsMode)
    if (filters?.status && filters.status !== 'all') q.set('status', filters.status)
    appendQueryTextParam(q, 'q', filters?.q)
    appendPaginationParams(q, { page: filters?.page, pageSize: filters?.pageSize })

    return await apiFetch<FleetListResponse>('/api/fleet' + buildQuerySuffix(q), { signal })
  }

  async function get(agentId: string, signal?: AbortSignal): Promise<FleetAgentDetailResponse> {
    return await apiFetch<FleetAgentDetailResponse>(`/api/fleet/${encodeURIComponent(agentId)}`, { signal })
  }

  return {
    list,
    get,
  }
})
