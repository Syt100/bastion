import type { CreateOrUpdateJobRequest, JobDetail } from '@/stores/jobs'

import type {
  ArtifactFormat,
  ConsistencyPolicy,
  FsErrorPolicy,
  FsHardlinkPolicy,
  FsSymlinkPolicy,
  JobEditorForm,
  SnapshotMode,
  WebdavRawTreeDirectMode,
} from './types'
import { cronToSimpleSchedule } from './schedule'

function parseStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value.filter((v): v is string => typeof v === 'string')
}

function parseLines(text: string): string[] {
  return text
    .split(/\r?\n/g)
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
}

function normalizeSymlinkPolicy(value: unknown): FsSymlinkPolicy {
  if (value === 'follow') return 'follow'
  if (value === 'skip') return 'skip'
  return 'keep'
}

function normalizeHardlinkPolicy(value: unknown): FsHardlinkPolicy {
  if (value === 'keep') return 'keep'
  return 'copy'
}

function normalizeErrorPolicy(value: unknown): FsErrorPolicy {
  if (value === 'skip_fail') return 'skip_fail'
  if (value === 'skip_ok') return 'skip_ok'
  return 'fail_fast'
}

function normalizeConsistencyPolicy(value: unknown): ConsistencyPolicy {
  if (value === 'fail') return 'fail'
  if (value === 'ignore') return 'ignore'
  return 'warn'
}

function normalizeSnapshotMode(value: unknown): SnapshotMode {
  if (value === 'auto') return 'auto'
  if (value === 'required') return 'required'
  return 'off'
}

function normalizeArtifactFormat(value: unknown): ArtifactFormat {
  if (value === 'raw_tree_v1') return 'raw_tree_v1'
  return 'archive_v1'
}

function normalizeWebdavRawTreeDirectMode(value: unknown): WebdavRawTreeDirectMode {
  if (value === 'auto') return 'auto'
  if (value === 'on') return 'on'
  return 'off'
}

function normalizeOptionalPositiveInt(value: number | null): number | null {
  if (typeof value !== 'number') return null
  const n = Math.floor(value)
  return n > 0 ? n : null
}

export function jobDetailToEditorForm(job: JobDetail): JobEditorForm {
  const spec = job.spec as Record<string, unknown>

  const pipeline = spec.pipeline as Record<string, unknown> | undefined
  const artifactFormat = normalizeArtifactFormat(pipeline?.format)
  const enc = pipeline?.encryption as Record<string, unknown> | undefined
  const encType = typeof enc?.type === 'string' ? enc.type : 'none'
  const encryptionEnabled = artifactFormat === 'raw_tree_v1' ? false : encType === 'age_x25519'
  const encryptionKeyName =
    encryptionEnabled && typeof enc?.key_name === 'string' && enc.key_name.trim() ? enc.key_name : 'default'

  const pipelineWebdav = pipeline?.webdav as Record<string, unknown> | undefined
  const rawTreeDirect = pipelineWebdav?.raw_tree_direct as Record<string, unknown> | undefined
  const webdavRawTreeDirectMode = normalizeWebdavRawTreeDirectMode(rawTreeDirect?.mode)
  const webdavRawTreeDirectResumeBySize =
    typeof rawTreeDirect?.resume_by_size === 'boolean' ? rawTreeDirect.resume_by_size : true
  const rawTreeDirectLimits = rawTreeDirect?.limits as Record<string, unknown> | undefined
  const webdavRawTreeDirectConcurrency =
    typeof rawTreeDirectLimits?.concurrency === 'number' && rawTreeDirectLimits.concurrency > 0
      ? Math.floor(rawTreeDirectLimits.concurrency)
      : 4
  const webdavRawTreeDirectPutQps =
    typeof rawTreeDirectLimits?.put_qps === 'number' && rawTreeDirectLimits.put_qps > 0
      ? Math.floor(rawTreeDirectLimits.put_qps)
      : 20
  const webdavRawTreeDirectHeadQps =
    typeof rawTreeDirectLimits?.head_qps === 'number' && rawTreeDirectLimits.head_qps > 0
      ? Math.floor(rawTreeDirectLimits.head_qps)
      : 50
  const webdavRawTreeDirectMkcolQps =
    typeof rawTreeDirectLimits?.mkcol_qps === 'number' && rawTreeDirectLimits.mkcol_qps > 0
      ? Math.floor(rawTreeDirectLimits.mkcol_qps)
      : 50
  const webdavRawTreeDirectBurst =
    typeof rawTreeDirectLimits?.burst === 'number' && rawTreeDirectLimits.burst > 0
      ? Math.floor(rawTreeDirectLimits.burst)
      : 10

  const target = spec.target as Record<string, unknown> | undefined
  const targetType = target?.type === 'local_dir' ? 'local_dir' : 'webdav'
  const partSizeMiB =
    typeof target?.part_size_bytes === 'number' && target.part_size_bytes > 0
      ? Math.max(1, Math.round(target.part_size_bytes / (1024 * 1024)))
      : 256

  const source = spec.source as Record<string, unknown> | undefined
  const paths = parseStringArray(source?.paths)
  const fsPaths =
    paths.length > 0
      ? paths
      : (() => {
          const legacyRoot = typeof source?.root === 'string' ? source.root : ''
          return legacyRoot.trim() ? [legacyRoot] : []
        })()
  const fsPreScan = typeof source?.pre_scan === 'boolean' ? source.pre_scan : true
  const fsSnapshotMode = normalizeSnapshotMode(source?.snapshot_mode)
  const fsSnapshotProviderRaw = typeof source?.snapshot_provider === 'string' ? source.snapshot_provider : ''
  const fsSnapshotProvider = fsSnapshotMode === 'off' ? '' : fsSnapshotProviderRaw
  const fsConsistencyPolicy = normalizeConsistencyPolicy(source?.consistency_policy)
  const fsConsistencyFailThreshold =
    typeof source?.consistency_fail_threshold === 'number' && source.consistency_fail_threshold >= 0
      ? Math.floor(source.consistency_fail_threshold)
      : 0
  const fsUploadOnConsistencyFailure = typeof source?.upload_on_consistency_failure === 'boolean' ? source.upload_on_consistency_failure : false

  const notif = spec.notifications as Record<string, unknown> | undefined
  const notifyMode = typeof notif?.mode === 'string' && notif.mode === 'custom' ? 'custom' : 'inherit'

  const retention = spec.retention as Record<string, unknown> | undefined
  const retentionEnabled = typeof retention?.enabled === 'boolean' ? retention.enabled : false
  const retentionKeepLast = typeof retention?.keep_last === 'number' ? retention.keep_last : null
  const retentionKeepDays = typeof retention?.keep_days === 'number' ? retention.keep_days : null
  const retentionMaxDeletePerTick =
    typeof retention?.max_delete_per_tick === 'number' && retention.max_delete_per_tick > 0 ? retention.max_delete_per_tick : 50
  const retentionMaxDeletePerDay =
    typeof retention?.max_delete_per_day === 'number' && retention.max_delete_per_day > 0 ? retention.max_delete_per_day : 200

  const schedule = job.schedule ?? ''
  const scheduleTimezone = job.schedule_timezone || 'UTC'
  const simple = schedule.trim() ? cronToSimpleSchedule(schedule) : null

  const vaultwardenConsistencyPolicy = normalizeConsistencyPolicy(source?.consistency_policy)
  const vaultwardenConsistencyFailThreshold =
    typeof source?.consistency_fail_threshold === 'number' && source.consistency_fail_threshold >= 0
      ? Math.floor(source.consistency_fail_threshold)
      : 0
  const vaultwardenUploadOnConsistencyFailure =
    typeof source?.upload_on_consistency_failure === 'boolean' ? source.upload_on_consistency_failure : false

  return {
    id: job.id,
    name: job.name,
    node: job.agent_id ? job.agent_id : 'hub',
    schedule,
    scheduleTimezone,
    scheduleMode: schedule.trim() ? (simple ? 'simple' : 'cron') : 'manual',
    simpleScheduleKind: simple?.kind ?? 'daily',
    simpleEveryMinutes: simple?.everyMinutes ?? 15,
    simpleAtHour: simple?.atHour ?? 0,
    simpleAtMinute: simple?.atMinute ?? 0,
    simpleWeekday: simple?.weekday ?? 1,
    simpleMonthday: simple?.monthday ?? 1,
    overlapPolicy: job.overlap_policy,
    jobType: job.spec.type,
    artifactFormat,
    encryptionEnabled,
    encryptionKeyName,
    fsPaths,
    fsInclude: parseStringArray(source?.include).join('\n'),
    fsExclude: parseStringArray(source?.exclude).join('\n'),
    fsPreScan,
    fsSymlinkPolicy: normalizeSymlinkPolicy(source?.symlink_policy),
    fsHardlinkPolicy: normalizeHardlinkPolicy(source?.hardlink_policy),
    fsErrorPolicy: normalizeErrorPolicy(source?.error_policy),
    fsSnapshotMode,
    fsSnapshotProvider,
    fsConsistencyPolicy,
    fsConsistencyFailThreshold,
    fsUploadOnConsistencyFailure,
    sqlitePath: typeof source?.path === 'string' ? source.path : '',
    sqliteIntegrityCheck: typeof source?.integrity_check === 'boolean' ? source.integrity_check : false,
    vaultwardenDataDir: typeof source?.data_dir === 'string' ? source.data_dir : '',
    vaultwardenConsistencyPolicy,
    vaultwardenConsistencyFailThreshold,
    vaultwardenUploadOnConsistencyFailure,
    targetType,
    webdavBaseUrl: typeof target?.base_url === 'string' ? target.base_url : '',
    webdavSecretName: typeof target?.secret_name === 'string' ? target.secret_name : '',
    webdavRawTreeDirectMode,
    webdavRawTreeDirectResumeBySize,
    webdavRawTreeDirectConcurrency,
    webdavRawTreeDirectPutQps,
    webdavRawTreeDirectHeadQps,
    webdavRawTreeDirectMkcolQps,
    webdavRawTreeDirectBurst,
    localBaseDir: typeof target?.base_dir === 'string' ? target.base_dir : '',
    partSizeMiB,
    notifyMode,
    notifyWecomBots: parseStringArray(notif?.['wecom_bot']),
    notifyEmails: parseStringArray(notif?.['email']),

    retentionEnabled,
    retentionKeepLast,
    retentionKeepDays,
    retentionMaxDeletePerTick,
    retentionMaxDeletePerDay,
  }
}

export function editorFormToRequest(form: JobEditorForm): CreateOrUpdateJobRequest {
  const partSizeMiB = Math.max(1, Math.floor(form.partSizeMiB || 1))
  const partSizeBytes = partSizeMiB * 1024 * 1024

  const pipeline = {
    format: form.artifactFormat,
    encryption:
      form.artifactFormat === 'raw_tree_v1'
        ? ({ type: 'none' as const } as const)
        : form.encryptionEnabled
          ? ({ type: 'age_x25519' as const, key_name: form.encryptionKeyName.trim() } as const)
          : ({ type: 'none' as const } as const),
    webdav: {
      raw_tree_direct: {
        mode: form.webdavRawTreeDirectMode,
        resume_by_size: form.webdavRawTreeDirectResumeBySize,
        ...(form.webdavRawTreeDirectMode !== 'off'
          ? {
              limits: {
                concurrency: Math.max(1, Math.floor(form.webdavRawTreeDirectConcurrency || 1)),
                put_qps: normalizeOptionalPositiveInt(form.webdavRawTreeDirectPutQps),
                head_qps: normalizeOptionalPositiveInt(form.webdavRawTreeDirectHeadQps),
                mkcol_qps: normalizeOptionalPositiveInt(form.webdavRawTreeDirectMkcolQps),
                burst: normalizeOptionalPositiveInt(form.webdavRawTreeDirectBurst),
              },
            }
          : {}),
      },
    },
  }

  const notifications =
    form.notifyMode === 'custom'
      ? {
          mode: 'custom' as const,
          wecom_bot: form.notifyWecomBots,
          email: form.notifyEmails,
        }
      : ({ mode: 'inherit' as const } as const)

  const source =
    form.jobType === 'filesystem'
      ? {
          pre_scan: form.fsPreScan,
          paths: form.fsPaths.map((p) => p.trim()).filter((p) => p.length > 0),
          include: parseLines(form.fsInclude),
          exclude: parseLines(form.fsExclude),
          symlink_policy: form.fsSymlinkPolicy,
          hardlink_policy: form.fsHardlinkPolicy,
          error_policy: form.fsErrorPolicy,
          snapshot_mode: form.fsSnapshotMode,
          ...(form.fsSnapshotMode !== 'off' && form.fsSnapshotProvider.trim()
            ? { snapshot_provider: form.fsSnapshotProvider.trim() }
            : {}),
          consistency_policy: form.fsConsistencyPolicy,
          ...(form.fsConsistencyPolicy === 'fail'
            ? {
                consistency_fail_threshold: Math.max(0, Math.floor(form.fsConsistencyFailThreshold || 0)),
                upload_on_consistency_failure: form.fsUploadOnConsistencyFailure,
              }
            : {}),
        }
      : form.jobType === 'sqlite'
        ? { path: form.sqlitePath.trim(), integrity_check: form.sqliteIntegrityCheck }
        : {
            data_dir: form.vaultwardenDataDir.trim(),
            consistency_policy: form.vaultwardenConsistencyPolicy,
            ...(form.vaultwardenConsistencyPolicy === 'fail'
              ? {
                  consistency_fail_threshold: Math.max(0, Math.floor(form.vaultwardenConsistencyFailThreshold || 0)),
                  upload_on_consistency_failure: form.vaultwardenUploadOnConsistencyFailure,
                }
              : {}),
          }

  const target =
    form.targetType === 'webdav'
      ? ({
          type: 'webdav' as const,
          base_url: form.webdavBaseUrl.trim(),
          secret_name: form.webdavSecretName.trim(),
          part_size_bytes: partSizeBytes,
        } as const)
      : ({
          type: 'local_dir' as const,
          base_dir: form.localBaseDir.trim(),
          part_size_bytes: partSizeBytes,
        } as const)

  const retentionKeepLast = normalizeOptionalPositiveInt(form.retentionKeepLast)
  const retentionKeepDays = normalizeOptionalPositiveInt(form.retentionKeepDays)
  const retentionMaxDeletePerTick = Math.max(1, Math.floor(form.retentionMaxDeletePerTick || 1))
  const retentionMaxDeletePerDay = Math.max(1, Math.floor(form.retentionMaxDeletePerDay || 1))
  const retention = {
    enabled: form.retentionEnabled,
    keep_last: retentionKeepLast,
    keep_days: retentionKeepDays,
    max_delete_per_tick: retentionMaxDeletePerTick,
    max_delete_per_day: retentionMaxDeletePerDay,
  }

  const includeRetention =
    retention.enabled ||
    retentionKeepLast !== null ||
    retentionKeepDays !== null ||
    retentionMaxDeletePerTick !== 50 ||
    retentionMaxDeletePerDay !== 200

  return {
    name: form.name.trim(),
    agent_id: form.node === 'hub' ? null : form.node,
    schedule: form.schedule.trim() ? form.schedule.trim() : null,
    schedule_timezone: form.scheduleTimezone.trim() || 'UTC',
    overlap_policy: form.overlapPolicy,
    spec: {
      v: 1 as const,
      type: form.jobType,
      pipeline,
      notifications,
      source,
      target,
      ...(includeRetention ? { retention } : {}),
    },
  }
}
