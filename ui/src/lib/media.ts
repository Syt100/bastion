import { onBeforeUnmount, onMounted, ref } from 'vue'

export function useMediaQuery(query: string) {
  let mql: MediaQueryList | null = null
  let removeListener: (() => void) | null = null

  const matches = ref<boolean>(false)
  if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
    mql = window.matchMedia(query)
    matches.value = Boolean(mql.matches)
  }

  function update(): void {
    matches.value = Boolean(mql?.matches)
  }

  onMounted(() => {
    if (!mql && (typeof window === 'undefined' || typeof window.matchMedia !== 'function')) {
      matches.value = false
      return
    }

    if (!mql) {
      mql = window.matchMedia(query)
      update()
    }

    const listener = () => update()
    if (typeof mql.addEventListener === 'function') {
      mql.addEventListener('change', listener)
      removeListener = () => mql?.removeEventListener('change', listener)
    } else if (typeof mql.addListener === 'function') {
      mql.addListener(listener)
      removeListener = () => mql?.removeListener(listener)
    }
  })

  onBeforeUnmount(() => {
    removeListener?.()
    removeListener = null
    mql = null
  })

  return matches
}
