import { getCurrentInstance, onBeforeUnmount, ref, type Ref } from 'vue'

export type ObservedElementHeightPx = {
  heightPx: Ref<number>
  measure: () => void
  start: () => void
  stop: () => void
}

export function useObservedElementHeightPx(
  el: Ref<HTMLElement | null>,
  compute?: (el: HTMLElement) => number,
): ObservedElementHeightPx {
  const heightPx = ref<number>(0)
  let resizeObserver: ResizeObserver | null = null

  function measure(): void {
    const target = el.value
    if (!target) return
    const raw = compute ? compute(target) : target.clientHeight
    const next = Math.floor(raw)
    if (!Number.isFinite(next) || next <= 0) return
    heightPx.value = next
  }

  function stop(): void {
    resizeObserver?.disconnect()
    resizeObserver = null
  }

  function start(): void {
    measure()
    if (typeof ResizeObserver === 'undefined') return
    stop()
    resizeObserver = new ResizeObserver(() => {
      measure()
    })
    if (el.value) resizeObserver.observe(el.value)
  }

  // Avoid noisy warnings in unit tests when the composable is used outside of a component setup().
  if (getCurrentInstance()) onBeforeUnmount(stop)

  return { heightPx, measure, start, stop }
}
