import { ref } from 'vue'
import { describe, expect, it, vi } from 'vitest'

import { openPickerSession, usePickerPanelState } from './usePickerSessionState'

describe('usePickerPanelState', () => {
  it('resets all overlay states to closed', () => {
    const state = usePickerPanelState()
    state.filtersPopoverOpen.value = true
    state.filtersDrawerOpen.value = true
    state.shortcutsPopoverOpen.value = true
    state.shortcutsDrawerOpen.value = true
    state.selectionPopoverOpen.value = true
    state.selectionDrawerOpen.value = true

    state.resetPanels()

    expect(state.filtersPopoverOpen.value).toBe(false)
    expect(state.filtersDrawerOpen.value).toBe(false)
    expect(state.shortcutsPopoverOpen.value).toBe(false)
    expect(state.shortcutsDrawerOpen.value).toBe(false)
    expect(state.selectionPopoverOpen.value).toBe(false)
    expect(state.selectionDrawerOpen.value).toBe(false)
  })
})

describe('openPickerSession', () => {
  it('resets state, opens modal, and triggers refresh', () => {
    const show = ref(false)
    const reset = vi.fn()
    const refresh = vi.fn()

    openPickerSession({ show, reset, refresh })

    expect(reset).toHaveBeenCalledTimes(1)
    expect(show.value).toBe(true)
    expect(refresh).toHaveBeenCalledTimes(1)
  })
})
