import type { RunEvent } from '@/stores/jobs'

export type RunEventFilters = {
  query?: string | null | undefined
  level?: string | null | undefined
  kind?: string | null | undefined
}

function norm(s: string | null | undefined): string {
  return (s ?? '').trim().toLowerCase()
}

export function uniqueRunEventKinds(events: RunEvent[]): string[] {
  const set = new Set<string>()
  for (const e of events) {
    const k = (e.kind ?? '').trim()
    if (k) set.add(k)
  }
  return [...set].sort((a, b) => a.localeCompare(b))
}

export function filterRunEvents(events: RunEvent[], filters: RunEventFilters): RunEvent[] {
  const q = norm(filters.query)
  const level = norm(filters.level)
  const kind = (filters.kind ?? '').trim()

  if (!q && !level && !kind) return events

  return events.filter((e) => {
    if (level && norm(e.level) !== level) return false
    if (kind && e.kind !== kind) return false
    if (!q) return true
    const message = norm(e.message)
    const k = norm(e.kind)
    return message.includes(q) || k.includes(q)
  })
}

export function findFirstEventSeq(events: RunEvent[], predicate: (e: RunEvent) => boolean): number | null {
  for (const e of events) {
    if (predicate(e)) return e.seq
  }
  return null
}

