import { defineStore } from 'pinia'
import { ref } from 'vue'

import { apiFetch } from '@/lib/api'
import { ensureCsrfToken } from '@/stores/csrf'

export type NotificationsSettings = {
  enabled: boolean
  channels: {
    wecom_bot: { enabled: boolean }
    email: { enabled: boolean }
  }
  templates: {
    wecom_markdown: string
    email_subject: string
    email_body: string
  }
}

export type NotificationChannel = 'wecom_bot' | 'email'

export type NotificationDestinationListItem = {
  channel: NotificationChannel
  name: string
  enabled: boolean
  updated_at: number
}

export type NotificationQueueItem = {
  id: string
  run_id: string
  job_id: string
  job_name: string
  channel: NotificationChannel
  destination: string
  status: string
  attempts: number
  next_attempt_at: number
  created_at: number
  updated_at: number
  last_error?: string | null
  destination_deleted: boolean
  destination_enabled: boolean
}

export type NotificationQueueResponse = {
  items: NotificationQueueItem[]
  page: number
  page_size: number
  total: number
}

export const useNotificationsStore = defineStore('notifications', () => {
  const settings = ref<NotificationsSettings | null>(null)
  const loadingSettings = ref(false)

  const destinations = ref<NotificationDestinationListItem[]>([])
  const loadingDestinations = ref(false)

  async function refreshSettings(): Promise<void> {
    loadingSettings.value = true
    try {
      settings.value = await apiFetch<NotificationsSettings>('/api/notifications/settings')
    } finally {
      loadingSettings.value = false
    }
  }

  async function saveSettings(next: NotificationsSettings): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>('/api/notifications/settings', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify(next),
      expectedStatus: 204,
    })
    settings.value = next
  }

  async function refreshDestinations(): Promise<void> {
    loadingDestinations.value = true
    try {
      destinations.value = await apiFetch<NotificationDestinationListItem[]>(
        '/api/notifications/destinations',
      )
    } finally {
      loadingDestinations.value = false
    }
  }

  async function setDestinationEnabled(
    channel: NotificationChannel,
    name: string,
    enabled: boolean,
  ): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(
      `/api/notifications/destinations/${encodeURIComponent(channel)}/${encodeURIComponent(name)}/enabled`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-CSRF-Token': csrf,
        },
        body: JSON.stringify({ enabled }),
        expectedStatus: 204,
      },
    )
  }

  async function testDestination(channel: NotificationChannel, name: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(
      `/api/notifications/destinations/${encodeURIComponent(channel)}/${encodeURIComponent(name)}/test`,
      {
        method: 'POST',
        headers: { 'X-CSRF-Token': csrf },
        expectedStatus: 204,
      },
    )
  }

  async function listQueue(params: {
    status?: string
    channel?: NotificationChannel
    page?: number
    pageSize?: number
  }): Promise<NotificationQueueResponse> {
    const q = new URLSearchParams()
    if (params.status) q.set('status', params.status)
    if (params.channel) q.set('channel', params.channel)
    if (params.page) q.set('page', String(params.page))
    if (params.pageSize) q.set('page_size', String(params.pageSize))
    const suffix = q.toString() ? `?${q.toString()}` : ''
    return await apiFetch<NotificationQueueResponse>(`/api/notifications/queue${suffix}`)
  }

  async function retryNow(id: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/notifications/queue/${encodeURIComponent(id)}/retry-now`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function cancel(id: string): Promise<void> {
    const csrf = await ensureCsrfToken()
    await apiFetch<void>(`/api/notifications/queue/${encodeURIComponent(id)}/cancel`, {
      method: 'POST',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  return {
    settings,
    loadingSettings,
    refreshSettings,
    saveSettings,
    destinations,
    loadingDestinations,
    refreshDestinations,
    setDestinationEnabled,
    testDestination,
    listQueue,
    retryNow,
    cancel,
  }
})

