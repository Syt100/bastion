import { defineStore } from 'pinia'
import { ref } from 'vue'

import { apiFetch } from '@/lib/api'
import { ensureCsrfToken } from '@/stores/csrf'

export type OverlapPolicy = 'reject' | 'queue'
export type JobType = 'filesystem' | 'sqlite' | 'vaultwarden'
export type RunStatus = 'queued' | 'running' | 'success' | 'failed' | 'rejected'

export type JobListItem = {
  id: string
  name: string
  agent_id: string | null
  schedule: string | null
  overlap_policy: OverlapPolicy
  created_at: number
  updated_at: number
  archived_at?: number | null
}

export type JobDetail = JobListItem & {
  spec: { v: 1; type: JobType } & Record<string, unknown>
}

export type CreateOrUpdateJobRequest = {
  name: string
  agent_id: string | null
  schedule: string | null
  overlap_policy: OverlapPolicy
  spec: { v: 1; type: JobType } & Record<string, unknown>
}

export type TriggerRunResponse = {
  run_id: string
  status: RunStatus
}

export type RunListItem = {
  id: string
  status: RunStatus
  started_at: number
  ended_at: number | null
  error: string | null
  executed_offline?: boolean
}

export type RunEvent = {
  run_id: string
  seq: number
  ts: number
  level: string
  kind: string
  message: string
  fields: unknown | null
}

export const useJobsStore = defineStore('jobs', () => {
  const items = ref<JobListItem[]>([])
  const loading = ref<boolean>(false)

  async function refresh(params?: { includeArchived?: boolean }): Promise<void> {
    loading.value = true
    try {
      const q = new URLSearchParams()
      if (params?.includeArchived) q.set('include_archived', 'true')
      const suffix = q.toString() ? `?${q.toString()}` : ''
      items.value = await apiFetch<JobListItem[]>(`/api/jobs${suffix}`)
    } finally {
      loading.value = false
    }
  }

  async function getJob(jobId: string): Promise<JobDetail> {
    return await apiFetch<JobDetail>(`/api/jobs/${encodeURIComponent(jobId)}`)
  }

  async function createJob(payload: CreateOrUpdateJobRequest): Promise<JobDetail> {
    const csrf = await ensureCsrfToken()
    return await apiFetch<JobDetail>('/api/jobs', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify(payload),
    })
  }

  async function updateJob(jobId: string, payload: CreateOrUpdateJobRequest): Promise<JobDetail> {
    const csrf = await ensureCsrfToken()
    return await apiFetch<JobDetail>(`/api/jobs/${encodeURIComponent(jobId)}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify(payload),
    })
  }

  async function deleteJob(jobId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}`, {
      method: 'DELETE',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function archiveJob(jobId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}/archive`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function unarchiveJob(jobId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}/unarchive`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function runNow(jobId: string): Promise<TriggerRunResponse> {
    const csrf = await ensureCsrfToken()
    return await apiFetch<TriggerRunResponse>(`/api/jobs/${encodeURIComponent(jobId)}/run`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
    })
  }

  async function listRuns(jobId: string): Promise<RunListItem[]> {
    return await apiFetch<RunListItem[]>(`/api/jobs/${encodeURIComponent(jobId)}/runs`)
  }

  async function listRunEvents(runId: string): Promise<RunEvent[]> {
    return await apiFetch<RunEvent[]>(`/api/runs/${encodeURIComponent(runId)}/events`)
  }

  return {
    items,
    loading,
    refresh,
    getJob,
    createJob,
    updateJob,
    deleteJob,
    archiveJob,
    unarchiveJob,
    runNow,
    listRuns,
    listRunEvents,
  }
})
