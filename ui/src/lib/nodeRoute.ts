const NODE_ROUTE_PREFIX_RE = /^\/n\/[^/]+/

export function isNodeScopedPath(path: string): boolean {
  return path.startsWith('/n/')
}

export function stripNodeScope(path: string): string {
  if (!isNodeScopedPath(path)) return path
  return path.replace(NODE_ROUTE_PREFIX_RE, '') || '/'
}

export function nodeScopedPath(nodeId: string, suffix: string): string {
  const normalizedSuffix = suffix.replace(/^\/+/, '')
  if (!normalizedSuffix) return `/n/${encodeURIComponent(nodeId)}`
  return `/n/${encodeURIComponent(nodeId)}/${normalizedSuffix}`
}

export function nodeJobsPath(nodeId: string): string {
  return nodeScopedPath(nodeId, 'jobs')
}

export function nodeStoragePath(nodeId: string): string {
  return nodeScopedPath(nodeId, 'settings/storage')
}
