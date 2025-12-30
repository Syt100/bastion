import { defineStore } from 'pinia'
import { ref } from 'vue'

import { apiFetch } from '@/lib/api'

export type SystemStatus = {
  version: string
  insecure_http: boolean
}

export const useSystemStore = defineStore('system', () => {
  const loading = ref<boolean>(false)
  const version = ref<string | null>(null)
  const insecureHttp = ref<boolean>(false)

  async function refresh(): Promise<void> {
    loading.value = true
    try {
      const status = await apiFetch<SystemStatus>('/api/system')
      version.value = status.version
      insecureHttp.value = status.insecure_http
    } finally {
      loading.value = false
    }
  }

  return { loading, version, insecureHttp, refresh }
})

