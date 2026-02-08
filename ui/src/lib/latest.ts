import { onBeforeUnmount } from 'vue'

export type LatestRequestHandle = {
  signal: AbortSignal
  isStale: () => boolean
  finish: () => void
}

export type LatestRequestController = {
  next: () => LatestRequestHandle
  abort: () => void
}

function createLatestRequestController(): LatestRequestController {
  let controller: AbortController | null = null
  let generation = 0

  function abort(): void {
    controller?.abort()
    controller = null
  }

  function next(): LatestRequestHandle {
    generation += 1
    abort()
    const g = generation
    const current = new AbortController()
    controller = current

    return {
      signal: current.signal,
      isStale: () => generation !== g,
      finish: () => {
        if (generation === g && controller === current) {
          controller = null
        }
      },
    }
  }

  return { next, abort }
}

export function createLatestRequest(): LatestRequestController {
  return createLatestRequestController()
}

export function useLatestRequest(): LatestRequestController {
  const latest = createLatestRequestController()
  onBeforeUnmount(() => latest.abort())
  return latest
}
