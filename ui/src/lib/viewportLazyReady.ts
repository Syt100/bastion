import { onBeforeUnmount, ref, watch, type Ref } from 'vue'

type ViewportLazyReadyOptions = {
  rootMargin?: string
}

export type ViewportLazyReadyController = {
  target: Ref<HTMLElement | null>
  ready: Ref<boolean>
  ensure: () => void
  stop: () => void
}

export function useViewportLazyReady(
  enabled: Ref<boolean>,
  options: ViewportLazyReadyOptions = {},
): ViewportLazyReadyController {
  const target = ref<HTMLElement | null>(null)
  const ready = ref(false)

  let observer: IntersectionObserver | null = null
  let observedTarget: HTMLElement | null = null

  function stop(): void {
    observer?.disconnect()
    observer = null
    observedTarget = null
  }

  function markReady(): void {
    if (!ready.value) {
      ready.value = true
    }
    stop()
  }

  function ensure(): void {
    if (ready.value || !enabled.value) return
    const el = target.value
    if (!el) return

    if (typeof window === 'undefined' || typeof window.IntersectionObserver === 'undefined') {
      markReady()
      return
    }

    if (observer && observedTarget === el) return

    stop()

    observer = new window.IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return
        markReady()
      },
      {
        root: null,
        rootMargin: options.rootMargin ?? '0px',
      },
    )

    observedTarget = el
    observer.observe(el)
  }

  watch(
    [enabled, target],
    ([isEnabled]) => {
      if (!isEnabled) {
        ready.value = false
        stop()
        return
      }
      ensure()
    },
    { immediate: true },
  )

  onBeforeUnmount(() => {
    stop()
  })

  return { target, ready, ensure, stop }
}
