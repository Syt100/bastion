import { computed, ref, type Ref } from 'vue'

import type { JobsWorkspaceLayoutMode } from '@/stores/ui'

const SPLIT_LIST_MIN_PX = 280
const SPLIT_LIST_MAX_PX = 640
const SPLIT_DETAIL_MIN_PX = 360

function clampInt(n: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.round(n)))
}

export type UseSplitWorkspaceResizeOptions = {
  layoutMode: Ref<JobsWorkspaceLayoutMode>
  persistedListWidthPx: Ref<number>
  setPersistedListWidthPx: (nextPx: number) => void
}

export function useSplitWorkspaceResize(options: UseSplitWorkspaceResizeOptions) {
  const splitGridEl = ref<HTMLElement | null>(null)
  const splitResizeActive = ref<boolean>(false)
  const splitListWidthDraftPx = ref<number | null>(null)
  let splitResizeCleanup: (() => void) | null = null

  const splitListWidthPx = computed<number>(() => splitListWidthDraftPx.value ?? options.persistedListWidthPx.value)
  const gridStyle = computed<Record<string, string> | undefined>(() => {
    if (options.layoutMode.value !== 'split') return undefined
    return {
      gridTemplateColumns: `minmax(0, ${splitListWidthPx.value}px) minmax(0, 1fr)`,
    }
  })

  function onSplitResizePointerDown(event: PointerEvent): void {
    if (options.layoutMode.value !== 'split') return
    const el = splitGridEl.value
    if (!el) return
    splitResizeCleanup?.()

    const handle = event.currentTarget as HTMLElement | null
    handle?.setPointerCapture?.(event.pointerId)

    const startX = event.clientX
    const startWidth = splitListWidthPx.value
    splitResizeActive.value = true
    splitListWidthDraftPx.value = startWidth

    const rect = el.getBoundingClientRect()
    const style = window.getComputedStyle(el)
    const colGap = Number.parseFloat(style.columnGap || '0') || 0
    const maxByContainer = rect.width - colGap - SPLIT_DETAIL_MIN_PX
    const maxWidth = clampInt(Math.min(SPLIT_LIST_MAX_PX, maxByContainer), SPLIT_LIST_MIN_PX, SPLIT_LIST_MAX_PX)

    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'

    let raf: number | null = null
    const onMove = (e: PointerEvent) => {
      const dx = e.clientX - startX
      const next = clampInt(startWidth + dx, SPLIT_LIST_MIN_PX, maxWidth)
      if (raf != null) cancelAnimationFrame(raf)
      raf = requestAnimationFrame(() => {
        raf = null
        splitListWidthDraftPx.value = next
      })
    }

    const cleanup = () => {
      window.removeEventListener('pointermove', onMove)
      splitResizeActive.value = false
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
      const next = splitListWidthDraftPx.value
      splitListWidthDraftPx.value = null
      splitResizeCleanup = null
      if (typeof next === 'number') {
        options.setPersistedListWidthPx(next)
      }
    }

    const onUp = () => {
      window.removeEventListener('pointerup', onUp)
      cleanup()
    }

    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
    splitResizeCleanup = cleanup
  }

  function cleanupSplitResize(): void {
    splitResizeCleanup?.()
  }

  return {
    splitGridEl,
    splitResizeActive,
    splitListWidthPx,
    gridStyle,
    onSplitResizePointerDown,
    cleanupSplitResize,
  }
}
