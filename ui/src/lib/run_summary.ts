type RecordValue = Record<string, unknown>

export type ParsedRunSummary = {
  targetType: string | null
  targetLocation: string | null
  entriesCount: number | null
  partsCount: number | null
  warningsTotal: number | null
  errorsTotal: number | null
  consistencyChangedTotal: number | null
  sqlitePath: string | null
  sqliteSnapshotName: string | null
  vaultwardenDataDir: string | null
  vaultwardenDb: string | null
}

function asRecord(value: unknown): RecordValue | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as RecordValue
}

function asString(value: unknown): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}

function asNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null
}

export function parseRunSummary(summary: unknown): ParsedRunSummary {
  const empty: ParsedRunSummary = {
    targetType: null,
    targetLocation: null,
    entriesCount: null,
    partsCount: null,
    warningsTotal: null,
    errorsTotal: null,
    consistencyChangedTotal: null,
    sqlitePath: null,
    sqliteSnapshotName: null,
    vaultwardenDataDir: null,
    vaultwardenDb: null,
  }

  const obj = asRecord(summary)
  if (!obj) return empty

  const target = asRecord(obj.target)
  const targetType = asString(target?.type)
  const runDir = asString(target?.run_dir)
  const runUrl = asString(target?.run_url)

  const filesystem = asRecord(obj.filesystem)
  const warningsTotal = asNumber(filesystem?.warnings_total)
  const errorsTotal = asNumber(filesystem?.errors_total)

  const sqlite = asRecord(obj.sqlite)
  const vaultwarden = asRecord(obj.vaultwarden)

  const consistency = asRecord(filesystem?.consistency) ?? asRecord(vaultwarden?.consistency)
  const consistencyChangedTotal = consistency
    ? (asNumber(consistency.changed_total) ?? 0) +
      (asNumber(consistency.replaced_total) ?? 0) +
      (asNumber(consistency.deleted_total) ?? 0) +
      (asNumber(consistency.read_error_total) ?? 0)
    : null

  return {
    targetType,
    targetLocation: runDir ?? runUrl,
    entriesCount: asNumber(obj.entries_count),
    partsCount: asNumber(obj.parts),
    warningsTotal,
    errorsTotal,
    consistencyChangedTotal,
    sqlitePath: asString(sqlite?.path),
    sqliteSnapshotName: asString(sqlite?.snapshot_name),
    vaultwardenDataDir: asString(vaultwarden?.data_dir),
    vaultwardenDb: asString(vaultwarden?.db),
  }
}
