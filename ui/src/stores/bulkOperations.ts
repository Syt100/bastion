import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'
import { ensureCsrfToken } from '@/stores/csrf'

export type BulkOperationStatus = 'queued' | 'running' | 'done' | 'canceled'
export type BulkOperationItemStatus = 'queued' | 'running' | 'success' | 'failed' | 'canceled'

export type BulkOperationListItem = {
  id: string
  kind: string
  status: BulkOperationStatus
  created_at: number
  updated_at: number
  started_at: number | null
  ended_at: number | null
  canceled_at: number | null
  total: number
  queued: number
  running: number
  success: number
  failed: number
  canceled: number
}

export type BulkOperationItemDetail = {
  op_id: string
  agent_id: string
  agent_name: string | null
  status: BulkOperationItemStatus
  attempts: number
  created_at: number
  updated_at: number
  started_at: number | null
  ended_at: number | null
  last_error_kind: string | null
  last_error: string | null
}

export type BulkOperationDetail = {
  id: string
  kind: string
  status: BulkOperationStatus
  created_by_user_id: number | null
  selector: unknown
  payload: unknown
  created_at: number
  updated_at: number
  started_at: number | null
  ended_at: number | null
  canceled_at: number | null
  total: number
  queued: number
  running: number
  success: number
  failed: number
  canceled: number
  items: BulkOperationItemDetail[]
}

export type BulkSelectorRequest =
  | { node_ids: string[] }
  | { labels: string[]; labels_mode?: 'and' | 'or' }

export type CreateBulkOperationRequest =
  | {
      kind: 'agent_labels_add' | 'agent_labels_remove'
      selector: BulkSelectorRequest
      payload: { labels: string[] }
    }
  | {
      kind: 'sync_config_now'
      selector: BulkSelectorRequest
      payload?: unknown
    }
  | {
      kind: 'webdav_secret_distribute'
      selector: BulkSelectorRequest
      payload: { name: string; overwrite?: boolean }
    }

export type WebdavDistributePreviewItem = {
  agent_id: string
  agent_name: string | null
  action: 'skip' | 'update'
  note: string | null
}

export type WebdavDistributePreviewResponse = {
  kind: 'webdav_secret_distribute'
  secret_name: string
  overwrite: boolean
  items: WebdavDistributePreviewItem[]
}

export const useBulkOperationsStore = defineStore('bulkOperations', () => {
  async function list(): Promise<BulkOperationListItem[]> {
    return await apiFetch<BulkOperationListItem[]>('/api/bulk-operations')
  }

  async function get(opId: string): Promise<BulkOperationDetail> {
    return await apiFetch<BulkOperationDetail>(`/api/bulk-operations/${encodeURIComponent(opId)}`)
  }

  async function previewWebdavSecretDistribute(params: {
    selector: BulkSelectorRequest
    payload: { name: string; overwrite?: boolean }
  }): Promise<WebdavDistributePreviewResponse> {
    const csrf = await ensureCsrfToken()
    return await apiFetch<WebdavDistributePreviewResponse>('/api/bulk-operations/preview', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({
        kind: 'webdav_secret_distribute',
        selector: params.selector,
        payload: {
          name: params.payload.name,
          overwrite: params.payload.overwrite ?? false,
        },
      }),
    })
  }

  async function create(req: CreateBulkOperationRequest): Promise<string> {
    const csrf = await ensureCsrfToken()
    const res = await apiFetch<{ op_id: string }>('/api/bulk-operations', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify(req),
    })
    return res.op_id
  }

  async function cancel(opId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/bulk-operations/${encodeURIComponent(opId)}/cancel`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function retryFailed(opId: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/bulk-operations/${encodeURIComponent(opId)}/retry-failed`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  return { list, get, previewWebdavSecretDistribute, create, cancel, retryFailed }
})
