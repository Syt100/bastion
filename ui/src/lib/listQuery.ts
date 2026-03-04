type ListSizeRange = {
  min?: number | null
  max?: number | null
  unit: string
}

type AppendListFilterParamsOptions = {
  q?: string | null
  kind?: string | null
  kindAllValue?: string
  hideDotfiles?: boolean
  typeSort?: string | null
  typeSortDefault?: string
  sortBy?: string | null
  sortDir?: string | null
  size?: ListSizeRange | null
  sizeMinKey?: string
  sizeMaxKey?: string
}

export function sizeUnitMultiplier(unit: string): number {
  if (unit === 'KB') return 1024
  if (unit === 'MB') return 1024 * 1024
  if (unit === 'GB') return 1024 * 1024 * 1024
  return 1
}

function normalizeQueryText(value: string | null | undefined): string | null {
  if (typeof value !== 'string') return null
  const normalized = value.trim()
  return normalized.length > 0 ? normalized : null
}

function normalizeSizeBytes(value: number | null | undefined, unit: string): number | null {
  if (value == null || !Number.isFinite(value)) return null
  return Math.max(0, Math.floor(value * sizeUnitMultiplier(unit)))
}

export function appendListFilterParams(params: URLSearchParams, options: AppendListFilterParamsOptions): void {
  const q = normalizeQueryText(options.q)
  if (q) params.set('q', q)

  const kind = normalizeQueryText(options.kind)
  const kindAllValue = options.kindAllValue ?? 'all'
  if (kind && kind !== kindAllValue) params.set('kind', kind)

  if (options.hideDotfiles) params.set('hide_dotfiles', 'true')

  const typeSort = normalizeQueryText(options.typeSort)
  if (typeSort && (!options.typeSortDefault || typeSort !== options.typeSortDefault)) {
    params.set('type_sort', typeSort)
  }

  const sortBy = normalizeQueryText(options.sortBy)
  if (sortBy) params.set('sort_by', sortBy)

  const sortDir = normalizeQueryText(options.sortDir)
  if (sortDir) params.set('sort_dir', sortDir)

  const size = options.size
  if (!size) return

  const minSizeKey = options.sizeMinKey ?? 'size_min_bytes'
  const maxSizeKey = options.sizeMaxKey ?? 'size_max_bytes'
  const minBytes = normalizeSizeBytes(size.min, size.unit)
  const maxBytes = normalizeSizeBytes(size.max, size.unit)

  if (minBytes != null) params.set(minSizeKey, String(minBytes))
  if (maxBytes != null) params.set(maxSizeKey, String(maxBytes))
}
