import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'
import {
  appendPaginationParams,
  appendQueryArrayParam,
  appendQueryTextParam,
  buildQuerySuffix,
} from '@/lib/listQuery'
import { ensureCsrfToken } from '@/stores/csrf'

export type CleanupTargetType = 'webdav' | 'local_dir'
export type CleanupTaskStatus = 'queued' | 'running' | 'retrying' | 'blocked' | 'done' | 'ignored' | 'abandoned'

export type CleanupTaskListItem = {
  run_id: string
  job_id: string
  job_name: string
  node_id: string
  target_type: CleanupTargetType
  status: CleanupTaskStatus
  attempts: number
  last_attempt_at: number | null
  next_attempt_at: number
  created_at: number
  updated_at: number
  last_error_kind: string | null
  last_error: string | null
}

export type ListCleanupTasksResponse = {
  items: CleanupTaskListItem[]
  page: number
  page_size: number
  total: number
}

export type CleanupTaskDetail = {
  run_id: string
  job_id: string
  job_name: string
  node_id: string
  target_type: CleanupTargetType
  target_snapshot: unknown
  status: CleanupTaskStatus
  attempts: number
  created_at: number
  updated_at: number
  last_attempt_at: number | null
  next_attempt_at: number
  last_error_kind: string | null
  last_error: string | null
  ignored_at: number | null
  ignored_by_user_id: number | null
  ignore_reason: string | null
}

export type CleanupEvent = {
  run_id: string
  seq: number
  ts: number
  level: string
  kind: string
  message: string
  fields: unknown | null
}

export type GetCleanupTaskResponse = {
  task: CleanupTaskDetail
  events: CleanupEvent[]
}

export const useIncompleteCleanupStore = defineStore('incompleteCleanup', () => {
  async function listTasks(params: {
    status?: CleanupTaskStatus | CleanupTaskStatus[]
    targetType?: CleanupTargetType | CleanupTargetType[]
    nodeId?: string
    jobId?: string
    page?: number
    pageSize?: number
    signal?: AbortSignal
  }): Promise<ListCleanupTasksResponse> {
    const q = new URLSearchParams()
    appendQueryArrayParam(
      q,
      'status[]',
      Array.isArray(params.status) ? params.status : params.status ? [params.status] : undefined,
    )
    appendQueryArrayParam(
      q,
      'target_type[]',
      Array.isArray(params.targetType) ? params.targetType : params.targetType ? [params.targetType] : undefined,
    )
    appendQueryTextParam(q, 'node_id', params.nodeId)
    appendQueryTextParam(q, 'job_id', params.jobId)
    appendPaginationParams(q, { page: params.page, pageSize: params.pageSize })
    const suffix = buildQuerySuffix(q)
    return await apiFetch<ListCleanupTasksResponse>(`/api/maintenance/incomplete-cleanup${suffix}`, {
      signal: params.signal,
    })
  }

  async function getTask(runId: string, signal?: AbortSignal): Promise<GetCleanupTaskResponse> {
    return await apiFetch<GetCleanupTaskResponse>(
      `/api/maintenance/incomplete-cleanup/${encodeURIComponent(runId)}`,
      { signal },
    )
  }

  async function retryNow(runId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/maintenance/incomplete-cleanup/${encodeURIComponent(runId)}/retry-now`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function ignore(runId: string, reason?: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/maintenance/incomplete-cleanup/${encodeURIComponent(runId)}/ignore`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({ reason }),
      expectedStatus: 204,
    })
  }

  async function unignore(runId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/maintenance/incomplete-cleanup/${encodeURIComponent(runId)}/unignore`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  return { listTasks, getTask, retryNow, ignore, unignore }
})
