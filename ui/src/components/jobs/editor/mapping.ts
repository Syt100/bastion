import type { CreateOrUpdateJobRequest, JobDetail } from '@/stores/jobs'

import type { ArtifactFormat, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy, JobEditorForm } from './types'
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

function normalizeArtifactFormat(value: unknown): ArtifactFormat {
  if (value === 'raw_tree_v1') return 'raw_tree_v1'
  return 'archive_v1'
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

  const notif = spec.notifications as Record<string, unknown> | undefined
  const notifyMode = typeof notif?.mode === 'string' && notif.mode === 'custom' ? 'custom' : 'inherit'

  const schedule = job.schedule ?? ''
  const scheduleTimezone = job.schedule_timezone || 'UTC'
  const simple = schedule.trim() ? cronToSimpleSchedule(schedule) : null

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
    fsSymlinkPolicy: normalizeSymlinkPolicy(source?.symlink_policy),
    fsHardlinkPolicy: normalizeHardlinkPolicy(source?.hardlink_policy),
    fsErrorPolicy: normalizeErrorPolicy(source?.error_policy),
    sqlitePath: typeof source?.path === 'string' ? source.path : '',
    sqliteIntegrityCheck: typeof source?.integrity_check === 'boolean' ? source.integrity_check : false,
    vaultwardenDataDir: typeof source?.data_dir === 'string' ? source.data_dir : '',
    targetType,
    webdavBaseUrl: typeof target?.base_url === 'string' ? target.base_url : '',
    webdavSecretName: typeof target?.secret_name === 'string' ? target.secret_name : '',
    localBaseDir: typeof target?.base_dir === 'string' ? target.base_dir : '',
    partSizeMiB,
    notifyMode,
    notifyWecomBots: parseStringArray(notif?.['wecom_bot']),
    notifyEmails: parseStringArray(notif?.['email']),
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
          paths: form.fsPaths.map((p) => p.trim()).filter((p) => p.length > 0),
          include: parseLines(form.fsInclude),
          exclude: parseLines(form.fsExclude),
          symlink_policy: form.fsSymlinkPolicy,
          hardlink_policy: form.fsHardlinkPolicy,
          error_policy: form.fsErrorPolicy,
        }
      : form.jobType === 'sqlite'
        ? { path: form.sqlitePath.trim(), integrity_check: form.sqliteIntegrityCheck }
        : { data_dir: form.vaultwardenDataDir.trim() }

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
    },
  }
}
