import { apiFetch } from '@/lib/api'
import { toApiErrorInfo, type ApiErrorInfo } from '@/lib/errors'
import { appendListFilterParams } from '@/lib/listQuery'

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

function extractStructuredAgentErrorCode(details: unknown): string | undefined {
  if (!details || typeof details !== 'object') return undefined

  const mapped = (details as { agent_error_code_mapped?: unknown }).agent_error_code_mapped
  if (typeof mapped === 'string' && mapped.trim()) return mapped.trim()

  const code = (details as { agent_error_code?: unknown }).agent_error_code
  if (typeof code === 'string' && code.trim()) return code.trim()
  return undefined
}

function shouldFallbackNotDirectory(info: ApiErrorInfo): boolean {
  if (info.code === 'not_directory') return true
  if (info.code !== 'agent_fs_list_failed') return false
  return extractStructuredAgentErrorCode(info.details) === 'not_directory'
}

async function listOnce(nodeId: string, req: PathPickerListRequest): Promise<PathPickerListResponse> {
  const params = new URLSearchParams()
  params.set('path', req.path)
  params.set('limit', String(req.limit))

  if (req.cursor && req.cursor.trim()) params.set('cursor', req.cursor.trim())
  appendListFilterParams(params, {
    q: req.q,
    kind: req.kind,
    hideDotfiles: req.hideDotfiles,
    typeSort: req.typeSort,
    sortBy: req.sortBy ? String(req.sortBy) : null,
    sortDir: req.sortDir ? String(req.sortDir) : null,
    size: req.size
      ? {
          min: req.size.min,
          max: req.size.max,
          unit: req.size.unit,
        }
      : null,
  })

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
    const agentCode = extractStructuredAgentErrorCode(info.details)
    if (agentCode === 'permission_denied') return 'permission_denied'
    if (agentCode === 'path_not_found') return 'not_found'
    if (agentCode === 'invalid_cursor') return 'invalid_cursor'
    if (agentCode === 'not_directory') return 'not_directory'
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
