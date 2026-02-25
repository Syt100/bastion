import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'
import { ensureCsrfToken } from '@/stores/csrf'

export type OperationKind = 'restore' | 'verify'
export type OperationStatus = 'running' | 'success' | 'failed' | 'canceled'

export type Operation = {
  id: string
  kind: OperationKind
  status: OperationStatus
  created_at: number
  started_at: number
  ended_at: number | null
  cancel_requested_at?: number | null
  cancel_requested_by_user_id?: number | null
  cancel_reason?: string | null
  progress?: unknown | null
  summary: unknown | null
  error: string | null
}

export type OperationEvent = {
  op_id: string
  seq: number
  ts: number
  level: string
  kind: string
  message: string
  fields: unknown | null
}

export type ConflictPolicy = 'overwrite' | 'skip' | 'fail'

export type RestoreDestination =
  | { type: 'local_fs'; node_id: string; directory: string }
  | { type: 'webdav'; base_url: string; secret_name: string; prefix: string }

export type RestoreExecutor = { node_id: string }

export const useOperationsStore = defineStore('operations', () => {
  const cancelOperationInFlight = new Map<string, Promise<Operation>>()

  async function startRestore(
    runId: string,
    destination: RestoreDestination,
    conflictPolicy: ConflictPolicy,
    selection?: { files: string[]; dirs: string[] } | null,
    executor?: RestoreExecutor | null,
  ): Promise<string> {
    const csrf = await ensureCsrfToken()
    const normalizedSelection =
      selection == null
        ? null
        : {
            files: Array.from(new Set(selection.files.map((v) => v.trim()).filter((v) => v.length > 0))),
            dirs: Array.from(
              new Set(
                selection.dirs
                  .map((v) => v.trim().replace(/[\\/]+$/, ''))
                  .filter((v) => v.length > 0),
              ),
            ),
          }
    const res = await apiFetch<{ op_id: string }>(`/api/runs/${encodeURIComponent(runId)}/restore`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({
        destination,
        ...(executor?.node_id?.trim() ? { executor: { node_id: executor.node_id.trim() } } : {}),
        conflict_policy: conflictPolicy,
        ...(normalizedSelection ? { selection: normalizedSelection } : {}),
      }),
    })
    return res.op_id
  }

  async function startVerify(runId: string): Promise<string> {
    const csrf = await ensureCsrfToken()
    const res = await apiFetch<{ op_id: string }>(`/api/runs/${encodeURIComponent(runId)}/verify`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
    })
    return res.op_id
  }

  async function getOperation(opId: string): Promise<Operation> {
    return await apiFetch<Operation>(`/api/operations/${encodeURIComponent(opId)}`)
  }

  async function listRunOperations(runId: string): Promise<Operation[]> {
    return await apiFetch<Operation[]>(`/api/runs/${encodeURIComponent(runId)}/operations`)
  }

  async function listEvents(opId: string): Promise<OperationEvent[]> {
    return await apiFetch<OperationEvent[]>(`/api/operations/${encodeURIComponent(opId)}/events`)
  }

  async function cancelOperation(opId: string, reason?: string): Promise<Operation> {
    const key = opId
    const existing = cancelOperationInFlight.get(key)
    if (existing) {
      return await existing
    }

    const request = (async () => {
      const csrf = await ensureCsrfToken()
      const normalizedReason = reason?.trim()
      return await apiFetch<Operation>(`/api/operations/${encodeURIComponent(opId)}/cancel`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-CSRF-Token': csrf,
        },
        body: JSON.stringify(normalizedReason ? { reason: normalizedReason } : {}),
      })
    })()

    cancelOperationInFlight.set(key, request)
    try {
      return await request
    } finally {
      if (cancelOperationInFlight.get(key) === request) {
        cancelOperationInFlight.delete(key)
      }
    }
  }

  return { startRestore, startVerify, getOperation, listRunOperations, listEvents, cancelOperation }
})
