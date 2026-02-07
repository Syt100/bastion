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
  schedule_timezone: string
  overlap_policy: OverlapPolicy
  created_at: number
  updated_at: number
  archived_at?: number | null
  latest_run_id?: string | null
  latest_run_status?: RunStatus | null
  latest_run_started_at?: number | null
  latest_run_ended_at?: number | null
}

export type JobDetail = JobListItem & {
  spec: { v: 1; type: JobType } & Record<string, unknown>
}

export type CreateOrUpdateJobRequest = {
  name: string
  agent_id: string | null
  schedule: string | null
  schedule_timezone: string
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
  issues_warnings_total?: number
  issues_errors_total?: number
  consistency_total?: number
  consistency_signal_total?: number
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

export type RunDetail = {
  id: string
  job_id: string
  status: RunStatus
  started_at: number
  ended_at: number | null
  progress?: unknown | null
  summary: unknown | null
  error: string | null
}

export type SnapshotStatus = 'present' | 'deleting' | 'deleted' | 'missing' | 'error'

export type SnapshotDeleteTaskSummary = {
  status: string
  attempts: number
  last_attempt_at?: number | null
  next_attempt_at: number
  last_error_kind?: string | null
  last_error?: string | null
  ignored_at?: number | null
}

export type SnapshotDeleteEvent = {
  run_id: string
  seq: number
  ts: number
  level: string
  kind: string
  message: string
  fields: unknown | null
}

export type SnapshotDeleteTaskDetail = {
  run_id: string
  job_id: string
  node_id: string
  target_type: string
  target_snapshot: unknown
  status: string
  attempts: number
  created_at: number
  updated_at: number
  last_attempt_at?: number | null
  next_attempt_at: number
  last_error_kind?: string | null
  last_error?: string | null
  ignored_at?: number | null
  ignored_by_user_id?: number | null
  ignore_reason?: string | null
}

export type RunArtifact = {
  run_id: string
  job_id: string
  node_id: string
  target_type: string
  target_snapshot: unknown
  artifact_format: string
  status: SnapshotStatus | string
  started_at: number
  ended_at: number
  pinned_at?: number | null
  pinned_by_user_id?: number | null
  source_files?: number | null
  source_dirs?: number | null
  source_bytes?: number | null
  transfer_bytes?: number | null
  last_error_kind?: string | null
  last_error?: string | null
  last_attempt_at?: number | null
  delete_task?: SnapshotDeleteTaskSummary | null
}

export type ListJobSnapshotsResponse = {
  items: RunArtifact[]
  next_cursor?: number | null
}

export type RetentionPolicy = {
  enabled: boolean
  keep_last?: number | null
  keep_days?: number | null
  max_delete_per_tick?: number
  max_delete_per_day?: number
}

export type RetentionPreviewItem = {
  run_id: string
  ended_at: number
  pinned: boolean
  source_bytes?: number | null
  transfer_bytes?: number | null
  reasons: string[]
}

export type RetentionPreviewResponse = {
  retention: RetentionPolicy
  keep_total: number
  delete_total: number
  keep: RetentionPreviewItem[]
  delete: RetentionPreviewItem[]
  scan_truncated: boolean
  result_truncated: boolean
}

export type RetentionApplyResponse = {
  enqueued: string[]
  already_exists: number
  skipped_due_to_limits: number
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

  async function archiveJob(jobId: string, opts?: { cascadeSnapshots?: boolean }): Promise<void> {
    const csrf = await ensureCsrfToken()
    const q = new URLSearchParams()
    if (opts?.cascadeSnapshots) q.set('cascade_snapshots', 'true')
    const suffix = q.toString() ? `?${q.toString()}` : ''
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}/archive${suffix}`, {
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

  async function getRun(runId: string): Promise<RunDetail> {
    return await apiFetch<RunDetail>(`/api/runs/${encodeURIComponent(runId)}`)
  }

  async function listJobSnapshots(
    jobId: string,
    params?: { cursor?: number; limit?: number; status?: string; signal?: AbortSignal },
  ): Promise<ListJobSnapshotsResponse> {
    const q = new URLSearchParams()
    if (params?.cursor !== undefined) q.set('cursor', String(params.cursor))
    if (params?.limit !== undefined) q.set('limit', String(params.limit))
    if (params?.status) q.set('status', params.status)
    const suffix = q.toString() ? `?${q.toString()}` : ''
    return await apiFetch<ListJobSnapshotsResponse>(`/api/jobs/${encodeURIComponent(jobId)}/snapshots${suffix}`, {
      signal: params?.signal,
    })
  }

  async function getJobRetention(jobId: string): Promise<RetentionPolicy> {
    return await apiFetch<RetentionPolicy>(`/api/jobs/${encodeURIComponent(jobId)}/retention`)
  }

  async function putJobRetention(jobId: string, retention: RetentionPolicy): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}/retention`, {
      method: 'PUT',
      expectedStatus: 204,
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify(retention),
    })
  }

  async function previewJobRetention(jobId: string, retention: RetentionPolicy): Promise<RetentionPreviewResponse> {
    return await apiFetch<RetentionPreviewResponse>(`/api/jobs/${encodeURIComponent(jobId)}/retention/preview`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ retention }),
    })
  }

  async function applyJobRetention(jobId: string, retention: RetentionPolicy): Promise<RetentionApplyResponse> {
    const csrf = await ensureCsrfToken()
    return await apiFetch<RetentionApplyResponse>(`/api/jobs/${encodeURIComponent(jobId)}/retention/apply`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({ retention }),
    })
  }

  async function deleteJobSnapshot(jobId: string, runId: string, opts?: { force?: boolean }): Promise<void> {
    const csrf = await ensureCsrfToken()
    const q = new URLSearchParams()
    if (opts?.force) q.set('force', 'true')
    const suffix = q.toString() ? `?${q.toString()}` : ''
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}/snapshots/${encodeURIComponent(runId)}/delete${suffix}`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function deleteJobSnapshotsBulk(jobId: string, runIds: string[], opts?: { force?: boolean }): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}/snapshots/delete`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': csrf },
      body: JSON.stringify({ run_ids: runIds, force: !!opts?.force }),
      expectedStatus: 204,
    })
  }

  async function pinJobSnapshot(jobId: string, runId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}/snapshots/${encodeURIComponent(runId)}/pin`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function unpinJobSnapshot(jobId: string, runId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/jobs/${encodeURIComponent(jobId)}/snapshots/${encodeURIComponent(runId)}/unpin`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function getJobSnapshotDeleteTask(jobId: string, runId: string): Promise<SnapshotDeleteTaskDetail> {
    return await apiFetch<SnapshotDeleteTaskDetail>(
      `/api/jobs/${encodeURIComponent(jobId)}/snapshots/${encodeURIComponent(runId)}/delete-task`,
    )
  }

  async function getJobSnapshotDeleteEvents(jobId: string, runId: string): Promise<SnapshotDeleteEvent[]> {
    return await apiFetch<SnapshotDeleteEvent[]>(
      `/api/jobs/${encodeURIComponent(jobId)}/snapshots/${encodeURIComponent(runId)}/delete-events`,
    )
  }

  async function retryJobSnapshotDeleteNow(jobId: string, runId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(
      `/api/jobs/${encodeURIComponent(jobId)}/snapshots/${encodeURIComponent(runId)}/delete/retry-now`,
      { method: 'POST', headers: { 'X-CSRF-Token': csrf }, expectedStatus: 204 },
    )
  }

  async function ignoreJobSnapshotDeleteTask(jobId: string, runId: string, reason?: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(
      `/api/jobs/${encodeURIComponent(jobId)}/snapshots/${encodeURIComponent(runId)}/delete/ignore`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': csrf },
        body: JSON.stringify({ reason }),
        expectedStatus: 204,
      },
    )
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
    getRun,
    listJobSnapshots,
    getJobRetention,
    putJobRetention,
    previewJobRetention,
    applyJobRetention,
    deleteJobSnapshot,
    deleteJobSnapshotsBulk,
    pinJobSnapshot,
    unpinJobSnapshot,
    getJobSnapshotDeleteTask,
    getJobSnapshotDeleteEvents,
    retryJobSnapshotDeleteNow,
    ignoreJobSnapshotDeleteTask,
  }
})
