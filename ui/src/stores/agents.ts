import { defineStore } from 'pinia'
import { ref } from 'vue'

import { apiFetch } from '@/lib/api'
import { ensureCsrfToken } from '@/stores/csrf'

export type AgentListItem = {
  id: string
  name: string | null
  revoked: boolean
  last_seen_at: number | null
  online: boolean
  labels: string[]
  desired_config_snapshot_id: string | null
  applied_config_snapshot_id: string | null
  config_sync_status: 'synced' | 'pending' | 'error' | 'offline'
  last_config_sync_attempt_at: number | null
  last_config_sync_error_kind: string | null
  last_config_sync_error: string | null
}

export type AgentsLabelsMode = 'and' | 'or'

export type AgentLabelIndexItem = {
  label: string
  count: number
}

export type EnrollmentToken = {
  token: string
  expires_at: number
  remaining_uses: number | null
}

export type RotateAgentKeyResponse = {
  agent_id: string
  agent_key: string
}

export type AgentDetail = {
  id: string
  name: string | null
  revoked: boolean
  created_at: number
  last_seen_at: number | null
  online: boolean
  capabilities_json: string | null
  labels: string[]
  desired_config_snapshot_id: string | null
  desired_config_snapshot_at: number | null
  applied_config_snapshot_id: string | null
  applied_config_snapshot_at: number | null
  config_sync_status: 'synced' | 'pending' | 'error' | 'offline'
  last_config_sync_attempt_at: number | null
  last_config_sync_error_kind: string | null
  last_config_sync_error: string | null
  last_config_sync_error_at: number | null
}

export type SyncConfigNowResponse = {
  outcome: 'sent' | 'unchanged' | 'pending_offline'
}

export const useAgentsStore = defineStore('agents', () => {
  const items = ref<AgentListItem[]>([])
  const loading = ref<boolean>(false)
  let refreshRequestSeq = 0
  let refreshAbortController: AbortController | null = null

  async function refresh(filters?: { labels?: string[]; labelsMode?: AgentsLabelsMode }): Promise<void> {
    const requestSeq = ++refreshRequestSeq
    refreshAbortController?.abort()
    const abortController = new AbortController()
    refreshAbortController = abortController
    loading.value = true
    try {
      const q = new URLSearchParams()
      if (filters?.labels?.length) {
        for (const label of filters.labels) q.append('labels[]', label)
      }
      if (filters?.labelsMode) q.set('labels_mode', filters.labelsMode)
      const suffix = q.toString() ? '?' + q.toString() : ''

      const nextItems = await apiFetch<AgentListItem[]>('/api/agents' + suffix, {
        signal: abortController.signal,
      })
      if (requestSeq !== refreshRequestSeq || abortController.signal.aborted) return
      items.value = nextItems
    } catch (error) {
      if (requestSeq !== refreshRequestSeq || abortController.signal.aborted) return
      throw error
    } finally {
      if (requestSeq === refreshRequestSeq) {
        loading.value = false
        refreshAbortController = null
      }
    }
  }

  async function listLabelIndex(): Promise<AgentLabelIndexItem[]> {
    return await apiFetch<AgentLabelIndexItem[]>('/api/agents/labels')
  }

  async function setAgentLabels(agentId: string, labels: string[]): Promise<void> {
    const csrf = await ensureCsrfToken()

    await apiFetch<void>(`/api/agents/${encodeURIComponent(agentId)}/labels`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({ labels }),
      expectedStatus: 204,
    })
  }

  async function createEnrollmentToken(params: {
    ttlSeconds: number
    remainingUses: number | null
  }): Promise<EnrollmentToken> {
    const csrf = await ensureCsrfToken()

    return await apiFetch<EnrollmentToken>('/api/agents/enrollment-tokens', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({
        ttl_seconds: params.ttlSeconds,
        remaining_uses: params.remainingUses,
      }),
    })
  }

  async function revokeAgent(agentId: string): Promise<void> {
    const csrf = await ensureCsrfToken()

    await apiFetch<void>(`/api/agents/${encodeURIComponent(agentId)}/revoke`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function rotateAgentKey(agentId: string): Promise<RotateAgentKeyResponse> {
    const csrf = await ensureCsrfToken()

    return await apiFetch<RotateAgentKeyResponse>(`/api/agents/${encodeURIComponent(agentId)}/rotate-key`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
    })
  }

  async function getAgent(agentId: string): Promise<AgentDetail> {
    return await apiFetch<AgentDetail>(`/api/agents/${encodeURIComponent(agentId)}`)
  }

  async function syncConfigNow(agentId: string): Promise<SyncConfigNowResponse> {
    const csrf = await ensureCsrfToken()
    return await apiFetch<SyncConfigNowResponse>(`/api/agents/${encodeURIComponent(agentId)}/sync-config-now`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
    })
  }

  return {
    items,
    loading,
    refresh,
    listLabelIndex,
    setAgentLabels,
    createEnrollmentToken,
    revokeAgent,
    rotateAgentKey,
    getAgent,
    syncConfigNow,
  }
})
