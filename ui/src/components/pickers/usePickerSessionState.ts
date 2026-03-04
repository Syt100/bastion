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
}): void {
  options.reset()
  options.show.value = true
  void options.refresh()
}
