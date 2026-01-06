export type ApiErrorBody = {
  error: string
  message?: string
  details?: unknown
}

export class ApiError extends Error {
  status: number
  body?: ApiErrorBody
  requestId?: string

  constructor(status: number, message: string, body?: ApiErrorBody, requestId?: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.body = body
    this.requestId = requestId
  }
}

export async function apiFetch<T>(
  input: RequestInfo | URL,
  init: RequestInit & { expectedStatus?: number } = {},
): Promise<T> {
  const expectedStatus = init.expectedStatus ?? 200

  const response = await fetch(input, {
    ...init,
    credentials: 'include',
    headers: {
      ...(init.headers ?? {}),
      Accept: 'application/json',
    },
  })

  if (response.status !== expectedStatus) {
    const requestId = response.headers.get('x-request-id')?.trim() || undefined

    if (response.status === 401 && typeof window !== 'undefined') {
      try {
        window.dispatchEvent(new CustomEvent('bastion:unauthorized'))
      } catch {
        // ignore
      }
    }

    let body: ApiErrorBody | undefined
    try {
      body = (await response.json()) as ApiErrorBody
    } catch {
      // ignore
    }
    throw new ApiError(response.status, body?.message ?? `HTTP ${response.status}`, body, requestId)
  }

  if (expectedStatus === 204) {
    return undefined as T
  }

  return (await response.json()) as T
}
