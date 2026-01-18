import { onBeforeUnmount, ref, watch, type Ref } from 'vue'

export function useShiftKeyPressed(show: Ref<boolean>) {
  const shiftPressed = ref<boolean>(false)

  const onKeydown = (e: KeyboardEvent) => {
    if (!show.value) return
    if (e.key === 'Shift') shiftPressed.value = true
  }

  const onKeyup = (e: KeyboardEvent) => {
    if (e.key === 'Shift') shiftPressed.value = false
  }

  const onBlur = () => {
    shiftPressed.value = false
  }

  watch(show, (open) => {
    shiftPressed.value = false
    if (open) {
      window.addEventListener('keydown', onKeydown)
      window.addEventListener('keyup', onKeyup)
      window.addEventListener('blur', onBlur)
      return
    }
    window.removeEventListener('keydown', onKeydown)
    window.removeEventListener('keyup', onKeyup)
    window.removeEventListener('blur', onBlur)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKeydown)
    window.removeEventListener('keyup', onKeyup)
    window.removeEventListener('blur', onBlur)
  })

  return { shiftPressed }
}

