import type { LocationQuery, LocationQueryRaw, RouteLocationNormalizedLoaded, RouteLocationRaw } from 'vue-router'

import { parseScopeQueryValue, scopeFromNodeId, type ScopeValue } from '@/lib/scope'

export type JobsCollectionState = {
  scope: ScopeValue
  q: string
  status: string
  schedule: string
  includeArchived: boolean
  sort: string
  page: number
  pageSize: number
  view: string | null
}

export const DEFAULT_JOBS_COLLECTION_STATE: JobsCollectionState = {
  scope: 'all',
  q: '',
  status: 'all',
  schedule: 'all',
  includeArchived: false,
  sort: 'updated_desc',
  page: 1,
  pageSize: 20,
  view: null,
}

function firstQueryValue(value: LocationQuery[string] | undefined): string | null {
  if (Array.isArray(value)) return typeof value[0] === 'string' ? value[0] : null
  return typeof value === 'string' ? value : null
}

function parsePositiveInt(value: LocationQuery[string] | undefined, fallback: number): number {
  const raw = firstQueryValue(value)
  if (!raw) return fallback
  const parsed = Number.parseInt(raw, 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback
}

function parseBoolean(value: LocationQuery[string] | undefined, fallback = false): boolean {
  const raw = firstQueryValue(value)
  if (!raw) return fallback
  return raw === '1' || raw === 'true' || raw === 'yes'
}

export function resolveJobsScope(route: Pick<RouteLocationNormalizedLoaded, 'params' | 'query'>, fallbackScope: ScopeValue): ScopeValue {
  const nodeId = typeof route.params.nodeId === 'string' ? route.params.nodeId : null
  if (nodeId) return scopeFromNodeId(nodeId)
  return parseScopeQueryValue(route.query.scope) ?? fallbackScope
}

export function readJobsCollectionState(
  query: LocationQuery,
  fallbackScope: ScopeValue = DEFAULT_JOBS_COLLECTION_STATE.scope,
): JobsCollectionState {
  return {
    scope: parseScopeQueryValue(query.scope) ?? fallbackScope,
    q: firstQueryValue(query.q)?.trim() ?? '',
    status: firstQueryValue(query.status)?.trim() || DEFAULT_JOBS_COLLECTION_STATE.status,
    schedule: firstQueryValue(query.schedule)?.trim() || DEFAULT_JOBS_COLLECTION_STATE.schedule,
    includeArchived: parseBoolean(query.archived, DEFAULT_JOBS_COLLECTION_STATE.includeArchived),
    sort: firstQueryValue(query.sort)?.trim() || DEFAULT_JOBS_COLLECTION_STATE.sort,
    page: parsePositiveInt(query.page, DEFAULT_JOBS_COLLECTION_STATE.page),
    pageSize: parsePositiveInt(query.page_size, DEFAULT_JOBS_COLLECTION_STATE.pageSize),
    view: firstQueryValue(query.view)?.trim() || null,
  }
}

export function buildJobsCollectionQuery(state: Partial<JobsCollectionState>): LocationQueryRaw {
  const merged: JobsCollectionState = {
    ...DEFAULT_JOBS_COLLECTION_STATE,
    ...state,
  }

  return {
    ...(merged.scope !== DEFAULT_JOBS_COLLECTION_STATE.scope ? { scope: merged.scope } : {}),
    ...(merged.q ? { q: merged.q } : {}),
    ...(merged.status !== DEFAULT_JOBS_COLLECTION_STATE.status ? { status: merged.status } : {}),
    ...(merged.schedule !== DEFAULT_JOBS_COLLECTION_STATE.schedule ? { schedule: merged.schedule } : {}),
    ...(merged.includeArchived ? { archived: 'true' } : {}),
    ...(merged.sort !== DEFAULT_JOBS_COLLECTION_STATE.sort ? { sort: merged.sort } : {}),
    ...(merged.page > 1 ? { page: String(merged.page) } : {}),
    ...(merged.pageSize !== DEFAULT_JOBS_COLLECTION_STATE.pageSize ? { page_size: String(merged.pageSize) } : {}),
    ...(merged.view ? { view: merged.view } : {}),
  }
}

export function buildJobsCollectionLocation(state: Partial<JobsCollectionState> = {}): RouteLocationRaw {
  return {
    path: '/jobs',
    query: buildJobsCollectionQuery(state),
  }
}

export function buildJobSectionPath(jobId: string, section: 'overview' | 'history' | 'data' = 'overview'): string {
  return `/jobs/${encodeURIComponent(jobId)}/${section}`
}

export function buildJobSectionLocation(
  jobId: string,
  section: 'overview' | 'history' | 'data' = 'overview',
  query: Partial<JobsCollectionState> = {},
): RouteLocationRaw {
  return {
    path: buildJobSectionPath(jobId, section),
    query: buildJobsCollectionQuery(query),
  }
}

export function buildJobEditorLocation(
  mode: 'create' | 'edit',
  options: {
    jobId?: string
    collection?: Partial<JobsCollectionState>
  } = {},
): RouteLocationRaw {
  if (mode === 'create') {
    return {
      path: '/jobs/new',
      query: buildJobsCollectionQuery(options.collection ?? {}),
    }
  }

  return {
    path: `/jobs/${encodeURIComponent(options.jobId ?? '')}/edit`,
    query: buildJobsCollectionQuery(options.collection ?? {}),
  }
}

export function buildLegacyJobsRedirectLocation(
  nodeId: string,
  path: '/jobs' | `/jobs/${string}`,
  query: LocationQuery,
): RouteLocationRaw {
  const scope = scopeFromNodeId(nodeId)
  const collection = readJobsCollectionState(query, scope)
  return {
    path,
    query: buildJobsCollectionQuery({ ...collection, scope }),
  }
}
