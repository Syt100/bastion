import type { Ref } from 'vue'

type UseLoadedRowSelectionOptions = {
  getSelected: () => Set<string>
  setSelected: (next: Set<string>) => void
  getLoaded: () => string[]
  shiftPressed: Ref<boolean>
  lastRangeAnchor: Ref<string | null>
  normalizeKey?: (key: string) => string
}

function normalizeLoadedKeys(loaded: string[], normalizeKey?: (key: string) => string): string[] {
  const out: string[] = []
  const seen = new Set<string>()

  for (const raw of loaded) {
    const key = normalizeKey ? normalizeKey(raw) : raw
    if (!key) continue
    if (seen.has(key)) continue
    seen.add(key)
    out.push(key)
  }

  return out
}

export function useLoadedRowSelection(options: UseLoadedRowSelectionOptions) {
  function loadedKeys(): string[] {
    return normalizeLoadedKeys(options.getLoaded(), options.normalizeKey)
  }

  function clearSelection(): void {
    options.setSelected(new Set())
    options.lastRangeAnchor.value = null
  }

  function selectAllLoadedRows(): void {
    const next = new Set(options.getSelected())
    for (const key of loadedKeys()) next.add(key)
    options.setSelected(next)
  }

  function invertLoadedRowsSelection(): void {
    const loaded = loadedKeys()
    const loadedSet = new Set(loaded)
    const next = new Set(options.getSelected())

    // Toggle loaded rows only; keep selection from other pages/paths.
    for (const key of loadedSet) {
      if (next.has(key)) next.delete(key)
      else next.add(key)
    }

    options.setSelected(next)
  }

  function updateCheckedRowKeys(keys: Array<string | number>): void {
    const loaded = loadedKeys()
    const loadedSet = new Set(loaded)
    const desiredLoaded = new Set<string>()

    for (const raw of keys) {
      const key = options.normalizeKey ? options.normalizeKey(String(raw)) : String(raw)
      if (!key || !loadedSet.has(key)) continue
      desiredLoaded.add(key)
    }

    const prev = options.getSelected()
    const next = new Set(prev)

    // Apply desired checked state for loaded rows; keep selection from other pages intact.
    for (const key of loaded) {
      if (desiredLoaded.has(key)) next.add(key)
      else next.delete(key)
    }

    const added: string[] = []
    const removed: string[] = []
    for (const key of loaded) {
      const was = prev.has(key)
      const now = next.has(key)
      if (!was && now) added.push(key)
      else if (was && !now) removed.push(key)
    }

    if (options.shiftPressed.value && options.lastRangeAnchor.value && added.length === 1 && removed.length === 0) {
      const anchor = options.lastRangeAnchor.value
      const focused = added[0]
      if (focused) {
        const idxA = loaded.indexOf(anchor)
        const idxB = loaded.indexOf(focused)
        if (idxA !== -1 && idxB !== -1) {
          const from = Math.min(idxA, idxB)
          const to = Math.max(idxA, idxB)
          for (const key of loaded.slice(from, to + 1)) next.add(key)
        }
      }
    }

    if (added.length === 1 && removed.length === 0) options.lastRangeAnchor.value = added[0] ?? null
    else if (removed.length === 1 && added.length === 0) options.lastRangeAnchor.value = removed[0] ?? null
    else if (next.size === 0) options.lastRangeAnchor.value = null

    options.setSelected(next)
  }

  return {
    clearSelection,
    selectAllLoadedRows,
    invertLoadedRowsSelection,
    updateCheckedRowKeys,
  }
}
