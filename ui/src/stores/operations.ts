import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'
import { ensureCsrfToken } from '@/stores/csrf'

export type OperationKind = 'restore' | 'verify'
export type OperationStatus = 'running' | 'success' | 'failed'

export type Operation = {
  id: string
  kind: OperationKind
  status: OperationStatus
  created_at: number
  started_at: number
  ended_at: number | null
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

export const useOperationsStore = defineStore('operations', () => {
  async function startRestore(runId: string, destinationDir: string, conflictPolicy: ConflictPolicy): Promise<string> {
    const csrf = await ensureCsrfToken()
    const res = await apiFetch<{ op_id: string }>(`/api/runs/${encodeURIComponent(runId)}/restore`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({ destination_dir: destinationDir, conflict_policy: conflictPolicy }),
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

  async function listEvents(opId: string): Promise<OperationEvent[]> {
    return await apiFetch<OperationEvent[]>(`/api/operations/${encodeURIComponent(opId)}/events`)
  }

  return { startRestore, startVerify, getOperation, listEvents }
})
