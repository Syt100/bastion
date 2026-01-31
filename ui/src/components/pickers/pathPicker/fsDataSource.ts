import { apiFetch } from '@/lib/api'
import { toApiErrorInfo, type ApiErrorInfo } from '@/lib/errors'

import type {
  PathPickerCapabilities,
  PathPickerEntry,
  PathPickerErrorInfo,
  PathPickerErrorKind,
  PathPickerListRequest,
  PathPickerListResponse,
  PickerDataSource,
  Translator,
} from '@/components/pickers/pathPicker/types'

type FsListEntry = {
  name: string
  path: string
  kind: string
  size: number
  mtime?: number | null
}

type FsListResponse = {
  path: string
  entries: FsListEntry[]
  next_cursor?: string | null
  total?: number | null
}

type FsPickerContext = { nodeId: 'hub' | string } | 'hub' | string

function nodeIdFromCtx(ctx: unknown): string {
  if (typeof ctx === 'string') return ctx
  if (!ctx || typeof ctx !== 'object') return ''
  if (!('nodeId' in ctx)) return ''
  const nodeId = (ctx as { nodeId?: unknown }).nodeId
  return typeof nodeId === 'string' ? nodeId : ''
}

function sizeUnitMultiplier(unit: string): number {
  if (unit === 'KB') return 1024
  if (unit === 'MB') return 1024 * 1024
  if (unit === 'GB') return 1024 * 1024 * 1024
  return 1
}

function computeParentPath(p: string): string {
  const raw = p.trim()
  if (!raw) return raw

  const trimmed = raw.replace(/[\\/]+$/, '')
  if (trimmed === '' || trimmed === '/') return '/'
  if (/^[A-Za-z]:$/.test(trimmed)) return `${trimmed}\\`

  const idxSlash = Math.max(trimmed.lastIndexOf('/'), trimmed.lastIndexOf('\\'))
  if (idxSlash <= 0) {
    if (/^[A-Za-z]:/.test(trimmed)) return trimmed.slice(0, 2) + '\\'
    return '/'
  }
  return trimmed.slice(0, idxSlash) || '/'
}

function joinPath(base: string, child: string): string {
  const b = base.trim()
  const c = child.trim()
  if (!b) return c
  if (!c) return b
  if (c.startsWith('/') || c.startsWith('\\') || /^[A-Za-z]:/.test(c)) return c

  const sep = b.includes('\\') || /^[A-Za-z]:/.test(b) ? '\\' : '/'
  const trimmedBase = b.replace(/[\\/]+$/, '')
  return `${trimmedBase}${sep}${c}`
}

function mapFsEntries(entries: FsListEntry[]): PathPickerEntry[] {
  return entries.map((e) => ({
    name: e.name,
    path: e.path,
    kind: e.kind,
    size: e.size,
    mtime: e.mtime ?? null,
  }))
}

function shouldFallbackNotDirectory(info: ApiErrorInfo): boolean {
  const code = info.code
  if (code === 'not_directory') return true
  if (code !== 'agent_fs_list_failed') return false
  const msgLower = (info.message || '').toLowerCase()
  return msgLower.includes('not a directory')
}

async function listOnce(nodeId: string, req: PathPickerListRequest): Promise<PathPickerListResponse> {
  const params = new URLSearchParams()
  params.set('path', req.path)
  params.set('limit', String(req.limit))

  if (req.cursor && req.cursor.trim()) params.set('cursor', req.cursor.trim())
  if (req.sortBy) params.set('sort_by', String(req.sortBy))
  if (req.sortDir) params.set('sort_dir', String(req.sortDir))

  if (req.kind && req.kind !== 'all') params.set('kind', req.kind)
  if (req.q) params.set('q', req.q)
  if (req.hideDotfiles) params.set('hide_dotfiles', 'true')
  if (req.typeSort) params.set('type_sort', req.typeSort)

  if (req.size) {
    const mult = sizeUnitMultiplier(req.size.unit)
    const minBytes =
      req.size.min != null && Number.isFinite(req.size.min) ? Math.max(0, Math.floor(req.size.min * mult)) : null
    const maxBytes =
      req.size.max != null && Number.isFinite(req.size.max) ? Math.max(0, Math.floor(req.size.max * mult)) : null
    if (minBytes != null) params.set('size_min_bytes', String(minBytes))
    if (maxBytes != null) params.set('size_max_bytes', String(maxBytes))
  }

  const url = `/api/nodes/${encodeURIComponent(nodeId)}/fs/list?${params.toString()}`
  const res = await apiFetch<FsListResponse>(url)
  return {
    path: res.path,
    entries: mapFsEntries(res.entries ?? []),
    nextCursor: res.next_cursor ?? null,
    total: typeof res.total === 'number' ? res.total : null,
  }
}

function mapErrorKind(info: ApiErrorInfo): PathPickerErrorKind {
  const code = info.code
  if (code === 'agent_offline') return 'offline'
  if (code === 'permission_denied') return 'permission_denied'
  if (code === 'path_not_found') return 'not_found'
  if (code === 'invalid_cursor') return 'invalid_cursor'
  if (code === 'not_directory') return 'not_directory'

  if (code === 'agent_fs_list_failed') {
    const msgLower = (info.message || '').toLowerCase()
    if (msgLower.includes('no such file') || msgLower.includes('not found') || msgLower.includes('cannot find the')) {
      return 'not_found'
    }
    if (msgLower.includes('permission denied') || msgLower.includes('access is denied')) {
      return 'permission_denied'
    }
    if (msgLower.includes('not a directory')) {
      return 'not_directory'
    }
  }

  return 'error'
}

export const fsPickerDataSource: PickerDataSource = {
  id: 'fs',
  i18nPrefix: 'fsPicker',
  persistenceNamespace: 'fsPicker',

  contextKey: (ctx: unknown) => nodeIdFromCtx(ctx),
  defaultPath: () => '/',

  normalizePath: (path: string) => path.trim(),
  parentPath: (path: string) => computeParentPath(path),
  joinPath: (base: string, child: string) => joinPath(base, child),

  capabilities: (): PathPickerCapabilities => ({
    search: true,
    kindFilter: { values: ['dir', 'file', 'symlink'], default: 'all' },
    hideDotfiles: true,
    sizeFilter: { units: ['B', 'KB', 'MB', 'GB'], defaultUnit: 'MB' },
    typeSort: { default: 'dir_first' },
    sort: { by: ['name', 'mtime', 'size'], dir: ['asc', 'desc'], defaultBy: 'name', defaultDir: 'asc' },
    columns: { kind: true, size: true, mtime: true },
    pagination: { pageSize: 200 },
  }),

  list: async (ctx: unknown, req: PathPickerListRequest, t?: Translator): Promise<PathPickerListResponse> => {
    const nodeId = nodeIdFromCtx(ctx)
    if (!nodeId) throw new Error('Fs picker requires a node id')

    try {
      return await listOnce(nodeId, req)
    } catch (error) {
      const info = toApiErrorInfo(error, t)
      if (req.mode === 'multi_paths' && !req.cursor && shouldFallbackNotDirectory(info)) {
        const parent = computeParentPath(req.path)
        if (parent && parent !== req.path) {
          return await listOnce(nodeId, { ...req, path: parent, cursor: null })
        }
      }
      throw error
    }
  },

  mapError: (error: unknown, ctx: unknown, t?: Translator): PathPickerErrorInfo => {
    void ctx
    const info = toApiErrorInfo(error, t)
    const kind = mapErrorKind(info)
    return { kind, message: info.message, code: info.code }
  },
}

export function fsPickerCtx(nodeId: 'hub' | string): FsPickerContext {
  return { nodeId }
}
