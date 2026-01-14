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

export const useAgentsStore = defineStore('agents', () => {
  const items = ref<AgentListItem[]>([])
  const loading = ref<boolean>(false)

  async function refresh(filters?: { labels?: string[]; labelsMode?: AgentsLabelsMode }): Promise<void> {
    loading.value = true
    try {
      const q = new URLSearchParams()
      if (filters?.labels?.length) {
        for (const label of filters.labels) q.append('labels[]', label)
      }
      if (filters?.labelsMode) q.set('labels_mode', filters.labelsMode)
      const suffix = q.toString() ? `?${q.toString()}` : ''

      items.value = await apiFetch<AgentListItem[]>(`/api/agents${suffix}`)
    } finally {
      loading.value = false
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

  return {
    items,
    loading,
    refresh,
    listLabelIndex,
    setAgentLabels,
    createEnrollmentToken,
    revokeAgent,
    rotateAgentKey,
  }
})
