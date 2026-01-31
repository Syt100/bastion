import { defineStore } from 'pinia'
import { ref } from 'vue'

import { apiFetch } from '@/lib/api'
import type { RunStatus } from '@/stores/jobs'

export type DashboardOverviewResponse = {
  stats: {
    agents: {
      total: number
      active: number
      online: number
      offline: number
      revoked: number
    }
    jobs: {
      active: number
      archived: number
    }
    runs: {
      running: number
      queued: number
      success_24h: number
      failed_24h: number
      rejected_24h: number
    }
    notifications: {
      queued: number
      sending: number
      failed: number
      canceled: number
    }
  }
  trend_7d: Array<{
    day: string
    success: number
    failed: number
  }>
  recent_runs: Array<{
    run_id: string
    job_id: string
    job_name: string
    node_id: string
    node_name?: string | null
    status: RunStatus | string
    started_at: number
    ended_at?: number | null
    error?: string | null
    executed_offline: boolean
  }>
}

export const useDashboardStore = defineStore('dashboard', () => {
  const loading = ref<boolean>(false)
  const overview = ref<DashboardOverviewResponse | null>(null)

  async function refresh(): Promise<void> {
    loading.value = true
    try {
      overview.value = await apiFetch<DashboardOverviewResponse>('/api/dashboard/overview')
    } finally {
      loading.value = false
    }
  }

  return { loading, overview, refresh }
})
