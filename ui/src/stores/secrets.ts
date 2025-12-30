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

export const useSecretsStore = defineStore('secrets', () => {
  const webdav = ref<SecretListItem[]>([])
  const loadingWebdav = ref<boolean>(false)

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

  return { webdav, loadingWebdav, refreshWebdav, getWebdav, upsertWebdav, deleteWebdav }
})

