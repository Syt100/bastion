import { ref } from 'vue'

export type ColumnWidthMap = Record<string, number>

const DEFAULT_MIN_WIDTH = 40
const DEFAULT_MAX_WIDTH = 2000
const DEFAULT_DEBOUNCE_MS = 200

function clamp(n: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, n))
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object' && !Array.isArray(value)
}

export function usePersistentColumnWidths(
  storageKey: string,
  opts?: { minWidth?: number; maxWidth?: number; debounceMs?: number },
) {
  const minWidth = opts?.minWidth ?? DEFAULT_MIN_WIDTH
  const maxWidth = opts?.maxWidth ?? DEFAULT_MAX_WIDTH
  const debounceMs = opts?.debounceMs ?? DEFAULT_DEBOUNCE_MS

  const widths = ref<ColumnWidthMap>({})

  const hasStorage = typeof window !== 'undefined' && typeof window.localStorage !== 'undefined'

  function load(): void {
    if (!hasStorage) return
    try {
      const raw = localStorage.getItem(storageKey)
      if (!raw) return
      const parsed = JSON.parse(raw) as unknown
      if (!isRecord(parsed)) return
      const next: ColumnWidthMap = {}
      for (const [k, v] of Object.entries(parsed)) {
        if (typeof v !== 'number' || !Number.isFinite(v)) continue
        next[k] = clamp(Math.round(v), minWidth, maxWidth)
      }
      widths.value = next
    } catch {
      // ignore corrupted storage
    }
  }

  let persistTimer: number | undefined

  function persistDebounced(): void {
    if (!hasStorage) return
    if (persistTimer !== undefined) window.clearTimeout(persistTimer)
    persistTimer = window.setTimeout(() => {
      persistTimer = undefined
      try {
        localStorage.setItem(storageKey, JSON.stringify(widths.value))
      } catch {
        // ignore quota / storage failures
      }
    }, debounceMs)
  }

  function getWidth(columnKey: string): number | undefined {
    return widths.value[columnKey]
  }

  function setWidth(columnKey: string, width: number): void {
    if (!columnKey) return
    if (!Number.isFinite(width)) return
    widths.value = { ...widths.value, [columnKey]: clamp(Math.round(width), minWidth, maxWidth) }
    persistDebounced()
  }

  load()

  return { widths, getWidth, setWidth }
}

