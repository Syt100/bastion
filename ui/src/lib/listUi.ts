export const LIST_QUERY_DEBOUNCE_MS = 260
export const LIST_PAGE_SIZE_OPTIONS = [20, 50, 100] as const
export const DEFAULT_LIST_PAGE_SIZE: number = LIST_PAGE_SIZE_OPTIONS[0]

export type ListRangeSummary = {
  start: number
  end: number
  total: number
}

export function buildListRangeSummary(totalItems: number, page: number, pageSize: number): ListRangeSummary {
  const total = Math.max(0, Math.trunc(totalItems))
  const normalizedPage = Math.max(1, Math.trunc(page) || 1)
  const normalizedPageSize = Math.max(1, Math.trunc(pageSize) || 1)

  if (total === 0) {
    return { start: 0, end: 0, total: 0 }
  }

  const start = (normalizedPage - 1) * normalizedPageSize + 1
  const end = Math.min(total, start + normalizedPageSize - 1)

  return {
    start: Math.min(start, total),
    end,
    total,
  }
}
