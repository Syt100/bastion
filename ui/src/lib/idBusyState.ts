import { ref } from 'vue'

export function useIdBusyState<T extends string | number = string>() {
  const busyById = ref<Record<string, true>>({})

  function keyOf(id: T): string {
    return String(id)
  }

  function isBusy(id: T): boolean {
    return busyById.value[keyOf(id)] === true
  }

  function start(id: T): boolean {
    if (isBusy(id)) return false
    busyById.value = {
      ...busyById.value,
      [keyOf(id)]: true,
    }
    return true
  }

  function stop(id: T): void {
    const key = keyOf(id)
    if (busyById.value[key] !== true) return
    const next = { ...busyById.value }
    delete next[key]
    busyById.value = next
  }

  function clear(): void {
    if (Object.keys(busyById.value).length === 0) return
    busyById.value = {}
  }

  return {
    busyById,
    isBusy,
    start,
    stop,
    clear,
  }
}
