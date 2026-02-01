import type { ComputedRef, Ref } from 'vue'

type MaybeRef<T> = T | Ref<T> | ComputedRef<T>

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

export function formatUnixSecondsYmdHms(ts: number | null): string {
  if (!ts) return '-'
  const d = new Date(ts * 1000)
  const yyyy = d.getFullYear()
  const mm = pad2(d.getMonth() + 1)
  const dd = pad2(d.getDate())
  const hh = pad2(d.getHours())
  const mi = pad2(d.getMinutes())
  const ss = pad2(d.getSeconds())
  return `${yyyy}-${mm}-${dd} ${hh}:${mi}:${ss}`
}

export function formatUnixSecondsYmdHm(ts: number | null): string {
  if (!ts) return '-'
  const d = new Date(ts * 1000)
  const yyyy = d.getFullYear()
  const mm = pad2(d.getMonth() + 1)
  const dd = pad2(d.getDate())
  const hh = pad2(d.getHours())
  const mi = pad2(d.getMinutes())
  return `${yyyy}-${mm}-${dd} ${hh}:${mi}`
}

export function useUnixSecondsFormatter(_locale: MaybeRef<string>) {
  void _locale

  function formatUnixSeconds(ts: number | null): string {
    return formatUnixSecondsYmdHms(ts)
  }

  return { formatUnixSeconds }
}
