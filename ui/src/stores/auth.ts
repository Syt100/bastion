import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { apiFetch, ApiError } from '@/lib/api'

export type SessionStatus = 'unknown' | 'authenticated' | 'anonymous'

export const useAuthStore = defineStore('auth', () => {
  const status = ref<SessionStatus>('unknown')
  const csrfToken = ref<string | null>(null)

  const isAuthenticated = computed(() => status.value === 'authenticated')

  async function refreshSession(): Promise<void> {
    try {
      const data = await apiFetch<{ authenticated: boolean; csrf_token?: string }>('/api/session')
      status.value = data.authenticated ? 'authenticated' : 'anonymous'
      csrfToken.value = data.csrf_token ?? null
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) {
        status.value = 'anonymous'
        csrfToken.value = null
        return
      }

      status.value = 'anonymous'
      csrfToken.value = null
    }
  }

  async function login(username: string, password: string): Promise<void> {
    const data = await apiFetch<{ csrf_token: string }>(
      '/api/auth/login',
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      },
    )

    status.value = 'authenticated'
    csrfToken.value = data.csrf_token
  }

  async function logout(): Promise<void> {
    await apiFetch<void>('/api/auth/logout', {
      method: 'POST',
      headers: csrfToken.value ? { 'X-CSRF-Token': csrfToken.value } : undefined,
      expectedStatus: 204,
    })

    status.value = 'anonymous'
    csrfToken.value = null
  }

  return { status, csrfToken, isAuthenticated, refreshSession, login, logout }
})

