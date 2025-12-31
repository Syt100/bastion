import { defineStore } from 'pinia'
import { ref } from 'vue'

import { apiFetch } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'

export type SecretListItem = {
  name: string
  updated_at: number
}

export type WebdavSecret = {
  name: string
  username: string
  password: string
}

export type WecomBotSecret = {
  name: string
  webhook_url: string
}

export type SmtpTlsMode = 'none' | 'starttls' | 'implicit'

export type SmtpSecret = {
  name: string
  host: string
  port: number
  username: string
  password: string
  from: string
  to: string[]
  tls: SmtpTlsMode
}

export const useSecretsStore = defineStore('secrets', () => {
  const webdav = ref<SecretListItem[]>([])
  const loadingWebdav = ref<boolean>(false)
  const wecomBots = ref<SecretListItem[]>([])
  const loadingWecomBots = ref<boolean>(false)
  const smtp = ref<SecretListItem[]>([])
  const loadingSmtp = ref<boolean>(false)

  const auth = useAuthStore()

  async function ensureCsrf(): Promise<string> {
    if (!auth.csrfToken) {
      await auth.refreshSession()
    }
    if (!auth.csrfToken) {
      throw new Error('Missing CSRF token')
    }
    return auth.csrfToken
  }

  async function refreshWebdav(): Promise<void> {
    loadingWebdav.value = true
    try {
      webdav.value = await apiFetch<SecretListItem[]>('/api/secrets/webdav')
    } finally {
      loadingWebdav.value = false
    }
  }

  async function getWebdav(name: string): Promise<WebdavSecret> {
    return await apiFetch<WebdavSecret>(`/api/secrets/webdav/${encodeURIComponent(name)}`)
  }

  async function upsertWebdav(name: string, username: string, password: string): Promise<void> {
    const csrf = await ensureCsrf()
    await apiFetch<void>(`/api/secrets/webdav/${encodeURIComponent(name)}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({ username, password }),
      expectedStatus: 204,
    })
  }

  async function deleteWebdav(name: string): Promise<void> {
    const csrf = await ensureCsrf()
    await apiFetch<void>(`/api/secrets/webdav/${encodeURIComponent(name)}`, {
      method: 'DELETE',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function refreshWecomBots(): Promise<void> {
    loadingWecomBots.value = true
    try {
      wecomBots.value = await apiFetch<SecretListItem[]>('/api/secrets/wecom-bot')
    } finally {
      loadingWecomBots.value = false
    }
  }

  async function getWecomBot(name: string): Promise<WecomBotSecret> {
    return await apiFetch<WecomBotSecret>(`/api/secrets/wecom-bot/${encodeURIComponent(name)}`)
  }

  async function upsertWecomBot(name: string, webhookUrl: string): Promise<void> {
    const csrf = await ensureCsrf()
    await apiFetch<void>(`/api/secrets/wecom-bot/${encodeURIComponent(name)}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify({ webhook_url: webhookUrl }),
      expectedStatus: 204,
    })
  }

  async function deleteWecomBot(name: string): Promise<void> {
    const csrf = await ensureCsrf()
    await apiFetch<void>(`/api/secrets/wecom-bot/${encodeURIComponent(name)}`, {
      method: 'DELETE',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  async function refreshSmtp(): Promise<void> {
    loadingSmtp.value = true
    try {
      smtp.value = await apiFetch<SecretListItem[]>('/api/secrets/smtp')
    } finally {
      loadingSmtp.value = false
    }
  }

  async function getSmtp(name: string): Promise<SmtpSecret> {
    return await apiFetch<SmtpSecret>(`/api/secrets/smtp/${encodeURIComponent(name)}`)
  }

  async function upsertSmtp(
    name: string,
    secret: Omit<SmtpSecret, 'name'>,
  ): Promise<void> {
    const csrf = await ensureCsrf()
    await apiFetch<void>(`/api/secrets/smtp/${encodeURIComponent(name)}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrf,
      },
      body: JSON.stringify(secret),
      expectedStatus: 204,
    })
  }

  async function deleteSmtp(name: string): Promise<void> {
    const csrf = await ensureCsrf()
    await apiFetch<void>(`/api/secrets/smtp/${encodeURIComponent(name)}`, {
      method: 'DELETE',
      headers: { 'X-CSRF-Token': csrf },
      expectedStatus: 204,
    })
  }

  return {
    webdav,
    loadingWebdav,
    refreshWebdav,
    getWebdav,
    upsertWebdav,
    deleteWebdav,
    wecomBots,
    loadingWecomBots,
    refreshWecomBots,
    getWecomBot,
    upsertWecomBot,
    deleteWecomBot,
    smtp,
    loadingSmtp,
    refreshSmtp,
    getSmtp,
    upsertSmtp,
    deleteSmtp,
  }
})
