export type ScopeValue = 'all' | 'hub' | `agent:${string}`

export function isScopeValue(value: string | null | undefined): value is ScopeValue {
  if (!value) return false
  if (value === 'all' || value === 'hub') return true
  return value.startsWith('agent:') && value.slice('agent:'.length).trim().length > 0
}

export function parseScopeValue(value: unknown): ScopeValue | null {
  if (typeof value !== 'string') return null
  const normalized = value.trim()
  return isScopeValue(normalized) ? normalized : null
}

export function scopeFromNodeId(nodeId: string | null | undefined): ScopeValue {
  const normalized = nodeId?.trim()
  if (!normalized || normalized === 'hub') return 'hub'
  return `agent:${normalized}`
}

export function scopeToNodeId(scope: ScopeValue | null | undefined): string | null {
  if (!scope || scope === 'all') return null
  if (scope === 'hub') return 'hub'
  return scope.slice('agent:'.length)
}

export function agentIdFromScope(scope: ScopeValue | null | undefined): string | null {
  if (!scope || scope === 'all' || scope === 'hub') return null
  return scope.slice('agent:'.length)
}

export function parseScopeQueryValue(value: unknown): ScopeValue | null {
  if (Array.isArray(value)) return parseScopeValue(value[0])
  return parseScopeValue(value)
}
