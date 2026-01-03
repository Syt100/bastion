import { onBeforeUnmount } from 'vue'

export type LatestRequestHandle = {
  signal: AbortSignal
  isStale: () => boolean
}

export type LatestRequestController = {
  next: () => LatestRequestHandle
  abort: () => void
}

export function useLatestRequest(): LatestRequestController {
  let controller: AbortController | null = null
  let generation = 0

  function abort(): void {
    controller?.abort()
    controller = null
  }

  function next(): LatestRequestHandle {
    generation += 1
    abort()
    controller = new AbortController()
    const g = generation
    return { signal: controller.signal, isStale: () => generation !== g }
  }

  onBeforeUnmount(() => abort())

  return { next, abort }
}

