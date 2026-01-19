export type Translator = (key: string, params?: Record<string, unknown>) => string

export type PathPickerMode = 'multi_paths' | 'single_dir'

export type PathPickerEntryKind = 'dir' | 'file' | 'symlink' | string

export type PathPickerEntry = {
  name: string
  path: string
  kind: PathPickerEntryKind
  size?: number | null
  mtime?: number | null
}

export type PathPickerSortDir = 'asc' | 'desc'

export type PathPickerSortBy = 'name' | 'mtime' | 'size' | string

export type PathPickerTypeSort = 'dir_first' | 'file_first'

export type PathPickerSizeUnit = 'B' | 'KB' | 'MB' | 'GB'

export type PathPickerSizeRange = {
  min: number | null
  max: number | null
  unit: PathPickerSizeUnit
}

export type PathPickerListRequest = {
  path: string
  cursor: string | null
  limit: number
  mode: PathPickerMode

  q?: string
  kind?: 'all' | 'dir' | 'file' | 'symlink'
  hideDotfiles?: boolean
  typeSort?: PathPickerTypeSort
  sortBy?: PathPickerSortBy
  sortDir?: PathPickerSortDir
  size?: PathPickerSizeRange
}

export type PathPickerListResponse = {
  path: string
  entries: PathPickerEntry[]
  nextCursor?: string | null
  total?: number | null
}

export type PathPickerErrorKind =
  | 'offline'
  | 'permission_denied'
  | 'not_found'
  | 'not_directory'
  | 'invalid_cursor'
  | 'error'

export type PathPickerErrorInfo = {
  kind: PathPickerErrorKind
  message: string
  code?: string
}

export type PathPickerCapabilities = {
  search?: boolean
  kindFilter?: {
    values: Array<'dir' | 'file' | 'symlink'>
    default: 'all' | 'dir' | 'file' | 'symlink'
  }
  hideDotfiles?: boolean
  sizeFilter?: {
    units: PathPickerSizeUnit[]
    defaultUnit: PathPickerSizeUnit
  }
  typeSort?: {
    default: PathPickerTypeSort
  }
  sort?: {
    by: PathPickerSortBy[]
    dir: PathPickerSortDir[]
    defaultBy: PathPickerSortBy
    defaultDir: PathPickerSortDir
  }
  columns?: {
    kind?: boolean
    size?: boolean
    mtime?: boolean
  }
  pagination?: {
    pageSize: number
  }
}

export type PathPickerOpenOptions = {
  path?: string
  mode?: PathPickerMode
}

export type PickerDataSource = {
  /** Unique identifier for debugging/telemetry. */
  id: string
  /** i18n prefix used by the generic picker for data-source specific labels. */
  i18nPrefix: string
  /** localStorage namespace used for persisting last-dir and filter state. */
  persistenceNamespace: string

  /** Stable context key (e.g. node id) used for persistence. */
  contextKey: (ctx: unknown) => string
  /** Default path when no explicit path or persisted last-dir is available. */
  defaultPath: (ctx: unknown) => string

  normalizePath: (path: string, ctx: unknown) => string
  parentPath: (path: string, ctx: unknown) => string
  joinPath: (base: string, child: string, ctx: unknown) => string

  capabilities: (ctx: unknown) => PathPickerCapabilities
  list: (ctx: unknown, req: PathPickerListRequest, t?: Translator) => Promise<PathPickerListResponse>
  mapError: (error: unknown, ctx: unknown, t?: Translator) => PathPickerErrorInfo
}

