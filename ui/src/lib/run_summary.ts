type RecordValue = Record<string, unknown>

export type ParsedConsistencySample = {
  path: string
  reason: string
  error: string | null
}

export type ParsedConsistencyReport = {
  v: number | null
  changedTotal: number
  replacedTotal: number
  deletedTotal: number
  readErrorTotal: number
  total: number
  sampleTruncated: boolean
  sample: ParsedConsistencySample[]
}

export type ParsedRunSummary = {
  targetType: string | null
  targetLocation: string | null
  entriesCount: number | null
  partsCount: number | null
  warningsTotal: number | null
  errorsTotal: number | null
  consistencyChangedTotal: number | null
  consistency: ParsedConsistencyReport | null
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

function parseConsistencyReport(value: unknown): ParsedConsistencyReport | null {
  const obj = asRecord(value)
  if (!obj) return null

  const changedTotal = asNumber(obj.changed_total) ?? 0
  const replacedTotal = asNumber(obj.replaced_total) ?? 0
  const deletedTotal = asNumber(obj.deleted_total) ?? 0
  const readErrorTotal = asNumber(obj.read_error_total) ?? 0
  const total = changedTotal + replacedTotal + deletedTotal + readErrorTotal

  const sampleRaw = Array.isArray(obj.sample) ? obj.sample : []
  const sample: ParsedConsistencySample[] = []
  for (const item of sampleRaw) {
    const it = asRecord(item)
    if (!it) continue
    const path = asString(it.path)
    const reason = asString(it.reason)
    if (!path || !reason) continue
    sample.push({ path, reason, error: asString(it.error) })
  }

  return {
    v: asNumber(obj.v),
    changedTotal,
    replacedTotal,
    deletedTotal,
    readErrorTotal,
    total,
    sampleTruncated: typeof obj.sample_truncated === 'boolean' ? obj.sample_truncated : false,
    sample,
  }
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
    consistency: null,
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

  const consistency =
    parseConsistencyReport(filesystem?.consistency) ?? parseConsistencyReport(vaultwarden?.consistency)
  const consistencyChangedTotal = consistency ? consistency.total : null

  return {
    targetType,
    targetLocation: runDir ?? runUrl,
    entriesCount: asNumber(obj.entries_count),
    partsCount: asNumber(obj.parts),
    warningsTotal,
    errorsTotal,
    consistencyChangedTotal,
    consistency,
    sqlitePath: asString(sqlite?.path),
    sqliteSnapshotName: asString(sqlite?.snapshot_name),
    vaultwardenDataDir: asString(vaultwarden?.data_dir),
    vaultwardenDb: asString(vaultwarden?.db),
  }
}
