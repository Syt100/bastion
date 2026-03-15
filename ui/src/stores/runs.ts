import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'
import { appendPaginationParams, appendQueryTextParam, buildQuerySuffix } from '@/lib/listQuery'
import type { ScopeValue } from '@/lib/scope'
import type { RunStatus } from '@/stores/jobs'
import { ensureCsrfToken } from '@/stores/csrf'

export type RunKind = 'backup' | 'restore' | 'verify' | 'cleanup'

export type RunWorkspaceListItem = {
  id: string
  job_id: string
  job_name: string
  scope: ScopeValue | string
  node_id: string
  node_name?: string | null
  status: RunStatus
  kind: RunKind | string
  started_at: number
  ended_at: number | null
  error?: string | null
  failure_title?: string | null
}

export type RunsWorkspaceListResponse = {
  scope: {
    requested: ScopeValue | string
    effective: ScopeValue | string
  }
  filters: {
    q: string
    status: RunStatus | 'all' | string
    job_id?: string | null
    kind: RunKind | 'all' | string
    range: '24h' | '7d' | '30d' | string
  }
  items: RunWorkspaceListItem[]
  page: number
  page_size: number
  total: number
}

export type RunWorkspaceDiagnostics = {
  state: 'structured' | 'fallback' | string
  failure_kind?: string | null
  failure_stage?: string | null
  failure_title: string
  failure_hint?: string | null
  first_error_event_seq?: number | null
  root_cause_event_seq?: number | null
}

export type RunWorkspaceCapabilities = {
  can_cancel: boolean
  can_restore: boolean
  can_verify: boolean
}

export type RunWorkspaceRelatedSummary = {
  operations_total: number
  artifacts_total: number
}

export type RunWorkspaceDetail = {
  run: {
    id: string
    job_id: string
    job_name?: string | null
    scope: ScopeValue | string
    node_id: string
    node_name?: string | null
    status: RunStatus
    kind: RunKind | string
    started_at: number
    ended_at: number | null
    cancel_requested_at?: number | null
    cancel_requested_by_user_id?: number | null
    cancel_reason?: string | null
    error?: string | null
  }
  progress?: unknown | null
  summary?: unknown | null
  diagnostics: RunWorkspaceDiagnostics
  capabilities: RunWorkspaceCapabilities
  related: RunWorkspaceRelatedSummary
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

export type RunEventConsoleResponse = {
  filters: {
    q: string
    levels: string[]
    kinds: string[]
  }
  window: {
    first_seq?: number | null
    last_seq?: number | null
    has_older: boolean
    has_newer: boolean
  }
  locators: {
    first_error_seq?: number | null
    root_cause_seq?: number | null
  }
  items: RunEvent[]
}

export const useRunsStore = defineStore('runs', () => {
  async function listWorkspace(params?: {
    scope?: ScopeValue
    status?: RunStatus | 'all'
    jobId?: string
    kind?: RunKind | 'all'
    range?: '24h' | '7d' | '30d'
    q?: string
    page?: number
    pageSize?: number
    signal?: AbortSignal
  }): Promise<RunsWorkspaceListResponse> {
    const q = new URLSearchParams()
    appendQueryTextParam(q, 'scope', params?.scope)
    appendQueryTextParam(q, 'q', params?.q)
    appendQueryTextParam(q, 'job_id', params?.jobId)
    if (params?.status && params.status !== 'all') q.set('status', params.status)
    if (params?.kind && params.kind !== 'all') q.set('kind', params.kind)
    if (params?.range) q.set('range', params.range)
    appendPaginationParams(q, { page: params?.page, pageSize: params?.pageSize })
    return await apiFetch<RunsWorkspaceListResponse>('/api/runs' + buildQuerySuffix(q), {
      signal: params?.signal,
    })
  }

  async function getWorkspace(runId: string): Promise<RunWorkspaceDetail> {
    return await apiFetch<RunWorkspaceDetail>(`/api/runs/${encodeURIComponent(runId)}/workspace`)
  }

  async function listEventConsole(
    runId: string,
    params?: {
      q?: string
      levels?: string[]
      kinds?: string[]
      limit?: number
      beforeSeq?: number
      afterSeq?: number
      anchor?: 'tail' | 'first_error' | `seq:${number}`
      signal?: AbortSignal
    },
  ): Promise<RunEventConsoleResponse> {
    const q = new URLSearchParams()
    appendQueryTextParam(q, 'q', params?.q)
    if (params?.levels && params.levels.length > 0) q.set('levels', params.levels.join(','))
    if (params?.kinds && params.kinds.length > 0) q.set('kinds', params.kinds.join(','))
    if (params?.limit != null) q.set('limit', String(params.limit))
    if (params?.beforeSeq != null) q.set('before_seq', String(params.beforeSeq))
    if (params?.afterSeq != null) q.set('after_seq', String(params.afterSeq))
    if (params?.anchor) q.set('anchor', params.anchor)
    return await apiFetch<RunEventConsoleResponse>(
      `/api/runs/${encodeURIComponent(runId)}/event-console${buildQuerySuffix(q)}`,
      { signal: params?.signal },
    )
  }

  async function cancelRun(runId: string, reason?: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    const normalizedReason = reason?.trim()
    await apiFetch(`/api/runs/${encodeURIComponent(runId)}/cancel`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify(normalizedReason ? { reason: normalizedReason } : {}),
    })
  }

  return {
    listWorkspace,
    getWorkspace,
    listEventConsole,
    cancelRun,
  }
})
