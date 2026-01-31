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

type WebdavListEntry = {
  name: string
  path: string
  kind: string
  size: number
  mtime?: number | null
}

type WebdavListResponse = {
  path: string
  entries: WebdavListEntry[]
  next_cursor?: string | null
  total?: number | null
}

export type WebdavPickerContext = { nodeId: 'hub' | string; baseUrl: string; secretName: string }

function ctxFromUnknown(ctx: unknown): WebdavPickerContext | null {
  if (!ctx || typeof ctx !== 'object') return null
  const v = ctx as { nodeId?: unknown; baseUrl?: unknown; secretName?: unknown }
  if (typeof v.nodeId !== 'string') return null
  if (typeof v.baseUrl !== 'string') return null
  if (typeof v.secretName !== 'string') return null
  return { nodeId: v.nodeId, baseUrl: v.baseUrl, secretName: v.secretName }
}

function normalizePath(p: string): string {
  const raw = p.trim()
  if (!raw) return '/'
  let out = raw.startsWith('/') ? raw : `/${raw}`
  out = out.replace(/\/+$/, '')
  return out === '' ? '/' : out
}

function parentPath(p: string): string {
  const path = normalizePath(p)
  if (path === '/') return '/'
  const idx = path.lastIndexOf('/')
  if (idx <= 0) return '/'
  return path.slice(0, idx) || '/'
}

function joinPath(base: string, child: string): string {
  const b = normalizePath(base)
  const c = child.trim()
  if (!c) return b
  if (c.startsWith('/')) return normalizePath(c)
  if (b === '/') return normalizePath(`/${c}`)
  return normalizePath(`${b}/${c}`)
}

function mapEntries(entries: WebdavListEntry[]): PathPickerEntry[] {
  return entries.map((e) => ({
    name: e.name,
    path: e.path,
    kind: e.kind,
    size: e.size,
    mtime: e.mtime ?? null,
  }))
}

function sizeUnitMultiplier(unit: string): number {
  if (unit === 'KB') return 1024
  if (unit === 'MB') return 1024 * 1024
  if (unit === 'GB') return 1024 * 1024 * 1024
  return 1
}

async function listOnce(ctx: WebdavPickerContext, req: PathPickerListRequest): Promise<PathPickerListResponse> {
  const params = new URLSearchParams()
  params.set('base_url', ctx.baseUrl)
  params.set('secret_name', ctx.secretName)
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

  const url = `/api/nodes/${encodeURIComponent(ctx.nodeId)}/webdav/list?${params.toString()}`
  const res = await apiFetch<WebdavListResponse>(url)
  return {
    path: res.path,
    entries: mapEntries(res.entries ?? []),
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
  return 'error'
}

export const webdavPickerDataSource: PickerDataSource = {
  id: 'webdav',
  i18nPrefix: 'webdavPicker',
  persistenceNamespace: 'webdavPicker',

  contextKey: (ctx: unknown) => {
    const v = ctxFromUnknown(ctx)
    if (!v) return ''
    return `${v.nodeId}|${v.baseUrl.trim()}|${v.secretName.trim()}`
  },
  defaultPath: () => '/',

  normalizePath: (path: string) => normalizePath(path),
  parentPath: (path: string) => parentPath(path),
  joinPath: (base: string, child: string) => joinPath(base, child),

  capabilities: (): PathPickerCapabilities => ({
    search: true,
    kindFilter: { values: ['dir', 'file'], default: 'all' },
    hideDotfiles: true,
    sizeFilter: { units: ['B', 'KB', 'MB', 'GB'], defaultUnit: 'MB' },
    typeSort: { default: 'dir_first' },
    sort: { by: ['name', 'mtime', 'size'], dir: ['asc', 'desc'], defaultBy: 'name', defaultDir: 'asc' },
    columns: { kind: true, size: true, mtime: true },
    pagination: { pageSize: 200 },
  }),

  list: async (ctx: unknown, req: PathPickerListRequest): Promise<PathPickerListResponse> => {
    const v = ctxFromUnknown(ctx)
    if (!v) throw new Error('WebDAV picker requires nodeId/baseUrl/secretName')
    if (!v.baseUrl.trim()) throw new Error('WebDAV picker requires baseUrl')
    if (!v.secretName.trim()) throw new Error('WebDAV picker requires secretName')
    return await listOnce(v, req)
  },

  mapError: (error: unknown, ctx: unknown, t?: Translator): PathPickerErrorInfo => {
    void ctx
    const info = toApiErrorInfo(error, t)
    const kind = mapErrorKind(info)
    return { kind, message: info.message, code: info.code }
  },
}

export function webdavPickerCtx(nodeId: 'hub' | string, baseUrl: string, secretName: string): WebdavPickerContext {
  return { nodeId, baseUrl, secretName }
}
