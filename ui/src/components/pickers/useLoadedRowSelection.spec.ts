import { ref } from 'vue'
import { describe, expect, it } from 'vitest'

import { useLoadedRowSelection } from './useLoadedRowSelection'

describe('useLoadedRowSelection', () => {
  it('keeps non-loaded selections while toggling loaded rows', () => {
    const selected = ref<Set<string>>(new Set(['outside', 'a']))
    const loaded = ref(['a', 'b'])
    const shiftPressed = ref(false)
    const lastRangeAnchor = ref<string | null>(null)

    const selection = useLoadedRowSelection({
      getSelected: () => new Set(selected.value),
      setSelected: (next) => {
        selected.value = new Set(next)
      },
      getLoaded: () => loaded.value,
      shiftPressed,
      lastRangeAnchor,
    })

    selection.invertLoadedRowsSelection()
    expect(Array.from(selected.value).sort()).toEqual(['b', 'outside'])
  })

  it('supports shift range selection using loaded order', () => {
    const selected = ref<Set<string>>(new Set())
    const loaded = ref(['a', 'b', 'c', 'd'])
    const shiftPressed = ref(false)
    const lastRangeAnchor = ref<string | null>(null)

    const selection = useLoadedRowSelection({
      getSelected: () => new Set(selected.value),
      setSelected: (next) => {
        selected.value = new Set(next)
      },
      getLoaded: () => loaded.value,
      shiftPressed,
      lastRangeAnchor,
    })

    selection.updateCheckedRowKeys(['b'])
    expect(lastRangeAnchor.value).toBe('b')

    shiftPressed.value = true
    selection.updateCheckedRowKeys(['b', 'd'])
    expect(new Set(selected.value)).toEqual(new Set(['b', 'c', 'd']))
  })

  it('normalizes loaded and checked keys when normalizeKey is provided', () => {
    const selected = ref<Set<string>>(new Set())
    const loaded = ref([' /a ', '/b', '/b'])
    const shiftPressed = ref(false)
    const lastRangeAnchor = ref<string | null>(null)

    const selection = useLoadedRowSelection({
      getSelected: () => new Set(selected.value),
      setSelected: (next) => {
        selected.value = new Set(next)
      },
      getLoaded: () => loaded.value,
      shiftPressed,
      lastRangeAnchor,
      normalizeKey: (key) => key.trim(),
    })

    selection.selectAllLoadedRows()
    expect(Array.from(selected.value).sort()).toEqual(['/a', '/b'])
  })
})
