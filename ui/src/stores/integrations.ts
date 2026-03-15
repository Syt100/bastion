import { defineStore } from 'pinia'

import { apiFetch } from '@/lib/api'

export type IntegrationsDomainState = 'ready' | 'empty' | 'degraded'

export type StorageSummary = {
  items_total: number
  in_use_total: number
  invalid_total: number
}

export type NotificationsSummary = {
  destinations_total: number
  recent_failures_total: number
  queue_backlog_total: number
}

export type DistributionSummary = {
  coverage_total: number
  drifted_total: number
  failed_total: number
}

export type IntegrationsDomainSummary<T> = {
  state: IntegrationsDomainState
  summary: T
}

export type IntegrationsSummaryResponse = {
  storage: IntegrationsDomainSummary<StorageSummary>
  notifications: IntegrationsDomainSummary<NotificationsSummary>
  distribution: IntegrationsDomainSummary<DistributionSummary>
}

export const useIntegrationsStore = defineStore('integrations', () => {
  async function getSummary(signal?: AbortSignal): Promise<IntegrationsSummaryResponse> {
    return await apiFetch<IntegrationsSummaryResponse>('/api/integrations', { signal })
  }

  return {
    getSummary,
  }
})
