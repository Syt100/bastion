import type { RouteLocationNormalizedLoaded } from 'vue-router'

import type { CommandCenterRangePreset } from '@/stores/commandCenter'
import { parseScopeQueryValue, type ScopeValue } from '@/lib/scope'

export const COMMAND_CENTER_RANGE_OPTIONS: Array<{ value: CommandCenterRangePreset; labelKey: string }> = [
  { value: '24h', labelKey: 'commandCenter.range.last24h' },
  { value: '7d', labelKey: 'commandCenter.range.last7d' },
  { value: '30d', labelKey: 'commandCenter.range.last30d' },
]

export function parseCommandCenterRangePreset(value: unknown): CommandCenterRangePreset {
  if (Array.isArray(value)) return parseCommandCenterRangePreset(value[0])
  if (value === '7d' || value === '30d') return value
  return '24h'
}

export function resolveRouteScope(
  route: RouteLocationNormalizedLoaded,
  preferredScope: ScopeValue,
): ScopeValue {
  return parseScopeQueryValue(route.query.scope) ?? preferredScope
}
