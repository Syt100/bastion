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

type AppendPaginationParamsOptions = {
  page?: number
  pageSize?: number
  pageKey?: string
  pageSizeKey?: string
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

function normalizeQueryNumber(value: number | null | undefined): number | null {
  if (value == null || !Number.isFinite(value)) return null
  return Math.trunc(value)
}

function normalizeSizeBytes(value: number | null | undefined, unit: string): number | null {
  if (value == null || !Number.isFinite(value)) return null
  return Math.max(0, Math.floor(value * sizeUnitMultiplier(unit)))
}

export function appendQueryTextParam(
  params: URLSearchParams,
  key: string,
  value: string | null | undefined,
): void {
  const normalized = normalizeQueryText(value)
  if (normalized) params.set(key, normalized)
}

export function appendQueryArrayParam(
  params: URLSearchParams,
  key: string,
  values: Array<string> | null | undefined,
): void {
  if (!Array.isArray(values) || values.length === 0) return
  for (const value of values) {
    const normalized = normalizeQueryText(value)
    if (normalized) params.append(key, normalized)
  }
}

export function appendQueryBooleanParam(
  params: URLSearchParams,
  key: string,
  enabled: boolean | null | undefined,
  truthyValue = 'true',
): void {
  if (enabled) params.set(key, truthyValue)
}

export function appendQueryNumberParam(
  params: URLSearchParams,
  key: string,
  value: number | null | undefined,
): void {
  const normalized = normalizeQueryNumber(value)
  if (normalized != null) params.set(key, String(normalized))
}

export function appendPaginationParams(
  params: URLSearchParams,
  options: AppendPaginationParamsOptions,
): void {
  appendQueryNumberParam(params, options.pageKey ?? 'page', options.page)
  appendQueryNumberParam(params, options.pageSizeKey ?? 'page_size', options.pageSize)
}

export function buildQuerySuffix(params: URLSearchParams): string {
  const query = params.toString()
  return query.length > 0 ? `?${query}` : ''
}

export function appendListFilterParams(params: URLSearchParams, options: AppendListFilterParamsOptions): void {
  appendQueryTextParam(params, 'q', options.q)

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
