export function isAbortError(error: unknown): boolean {
  if (!error || typeof error !== 'object') return false
  if (typeof DOMException !== 'undefined' && error instanceof DOMException) return error.name === 'AbortError'
  return 'name' in error && (error as { name?: unknown }).name === 'AbortError'
}

export function createDebouncedTask(task: () => void, delayMs: number) {
  let timer: ReturnType<typeof setTimeout> | null = null

  function schedule(): void {
    if (timer != null) clearTimeout(timer)
    timer = setTimeout(() => {
      timer = null
      task()
    }, delayMs)
  }

  function cancel(): void {
    if (timer != null) {
      clearTimeout(timer)
      timer = null
    }
  }

  return {
    schedule,
    cancel,
  }
}
