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

  async function refresh(): Promise<void> {
    loading.value = true
    try {
      items.value = await apiFetch<AgentListItem[]>('/api/agents')
    } finally {
      loading.value = false
    }
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

  return { items, loading, refresh, createEnrollmentToken, revokeAgent, rotateAgentKey }
})
