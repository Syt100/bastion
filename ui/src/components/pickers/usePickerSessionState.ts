import { ref, type Ref } from 'vue'

export function usePickerPanelState() {
  const filtersPopoverOpen = ref<boolean>(false)
  const filtersDrawerOpen = ref<boolean>(false)
  const shortcutsPopoverOpen = ref<boolean>(false)
  const shortcutsDrawerOpen = ref<boolean>(false)
  const selectionPopoverOpen = ref<boolean>(false)
  const selectionDrawerOpen = ref<boolean>(false)

  function resetPanels(): void {
    filtersPopoverOpen.value = false
    filtersDrawerOpen.value = false
    shortcutsPopoverOpen.value = false
    shortcutsDrawerOpen.value = false
    selectionPopoverOpen.value = false
    selectionDrawerOpen.value = false
  }

  return {
    filtersPopoverOpen,
    filtersDrawerOpen,
    shortcutsPopoverOpen,
    shortcutsDrawerOpen,
    selectionPopoverOpen,
    selectionDrawerOpen,
    resetPanels,
  }
}

export function openPickerSession(options: {
  show: Ref<boolean>
  reset: () => void
  refresh: () => Promise<void> | void
  refreshMode?: 'immediate' | 'after-open-frame'
  scheduleRefresh?: (refresh: () => void) => void
}): void {
  const refreshMode = options.refreshMode ?? 'immediate'

  const scheduleRefresh =
    options.scheduleRefresh ??
    ((run: () => void) => {
      setTimeout(run, 0)
    })

  options.reset()
  options.show.value = true
  if (refreshMode === 'immediate') {
    void options.refresh()
    return
  }
  scheduleRefresh(() => {
    if (!options.show.value) return
    void options.refresh()
  })
}
