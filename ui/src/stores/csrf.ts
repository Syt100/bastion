import { useAuthStore } from '@/stores/auth'

export async function ensureCsrfToken(): Promise<string> {
  const auth = useAuthStore()
  if (!auth.csrfToken) {
    await auth.refreshSession()
  }
  if (!auth.csrfToken) {
    throw new Error('Missing CSRF token')
  }
  return auth.csrfToken
}

