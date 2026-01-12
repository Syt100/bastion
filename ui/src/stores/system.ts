import { defineStore } from 'pinia'
import { ref } from 'vue'

import { apiFetch } from '@/lib/api'

export type SystemStatus = {
  version: string
  insecure_http: boolean
  hub_timezone: string
}

export const useSystemStore = defineStore('system', () => {
  const loading = ref<boolean>(false)
  const version = ref<string | null>(null)
  const insecureHttp = ref<boolean>(false)
  const hubTimezone = ref<string>('UTC')

  async function refresh(): Promise<void> {
    loading.value = true
    try {
      const status = await apiFetch<SystemStatus>('/api/system')
      version.value = status.version
      insecureHttp.value = status.insecure_http
      hubTimezone.value = status.hub_timezone || 'UTC'
    } finally {
      loading.value = false
    }
  }

  return { loading, version, insecureHttp, hubTimezone, refresh }
})
