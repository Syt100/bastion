import type { JobType, OverlapPolicy } from '@/stores/jobs'

export type FsSymlinkPolicy = 'keep' | 'follow' | 'skip'
export type FsHardlinkPolicy = 'copy' | 'keep'
export type FsErrorPolicy = 'fail_fast' | 'skip_fail' | 'skip_ok'
export type ConsistencyPolicy = 'warn' | 'fail' | 'ignore'
export type SnapshotMode = 'off' | 'auto' | 'required'

export type JobTargetType = 'webdav' | 'local_dir'
export type NotifyMode = 'inherit' | 'custom'
export type NodeIdOrHub = 'hub' | string
export type ArtifactFormat = 'archive_v1' | 'raw_tree_v1'
export type WebdavRawTreeDirectMode = 'off' | 'auto' | 'on'

export type JobEditorMode = 'create' | 'edit'

export type JobScheduleMode = 'manual' | 'simple' | 'cron'
export type JobSimpleScheduleKind = 'every_minutes' | 'hourly' | 'daily' | 'weekly' | 'monthly'

export type JobEditorField =
  | 'name'
  | 'scheduleTimezone'
  | 'schedule'
  | 'retentionKeepLast'
  | 'retentionKeepDays'
  | 'retentionMaxDeletePerTick'
  | 'retentionMaxDeletePerDay'
  | 'fsPaths'
  | 'fsSnapshotMode'
  | 'fsSnapshotProvider'
  | 'fsConsistencyPolicy'
  | 'fsConsistencyFailThreshold'
  | 'fsUploadOnConsistencyFailure'
  | 'sqlitePath'
  | 'vaultwardenDataDir'
  | 'vaultwardenConsistencyPolicy'
  | 'vaultwardenConsistencyFailThreshold'
  | 'vaultwardenUploadOnConsistencyFailure'
  | 'webdavBaseUrl'
  | 'webdavSecretName'
  | 'webdavRawTreeDirectMode'
  | 'webdavRawTreeDirectConcurrency'
  | 'webdavRawTreeDirectPutQps'
  | 'webdavRawTreeDirectHeadQps'
  | 'webdavRawTreeDirectMkcolQps'
  | 'webdavRawTreeDirectBurst'
  | 'localBaseDir'
  | 'partSizeMiB'
  | 'encryptionKeyName'

export type JobEditorForm = {
  id: string | null
  name: string
  node: NodeIdOrHub
  schedule: string
  scheduleTimezone: string
  scheduleMode: JobScheduleMode
  simpleScheduleKind: JobSimpleScheduleKind
  simpleEveryMinutes: number
  simpleAtHour: number
  simpleAtMinute: number
  simpleWeekday: number
  simpleMonthday: number
  overlapPolicy: OverlapPolicy
  jobType: JobType
  artifactFormat: ArtifactFormat
  encryptionEnabled: boolean
  encryptionKeyName: string
  fsPaths: string[]
  fsInclude: string
  fsExclude: string
  fsPreScan: boolean
  fsSymlinkPolicy: FsSymlinkPolicy
  fsHardlinkPolicy: FsHardlinkPolicy
  fsErrorPolicy: FsErrorPolicy
  fsSnapshotMode: SnapshotMode
  fsSnapshotProvider: string
  fsConsistencyPolicy: ConsistencyPolicy
  fsConsistencyFailThreshold: number
  fsUploadOnConsistencyFailure: boolean
  sqlitePath: string
  sqliteIntegrityCheck: boolean
  vaultwardenDataDir: string
  vaultwardenConsistencyPolicy: ConsistencyPolicy
  vaultwardenConsistencyFailThreshold: number
  vaultwardenUploadOnConsistencyFailure: boolean
  targetType: JobTargetType
  webdavBaseUrl: string
  webdavSecretName: string
  webdavRawTreeDirectMode: WebdavRawTreeDirectMode
  webdavRawTreeDirectResumeBySize: boolean
  webdavRawTreeDirectConcurrency: number
  webdavRawTreeDirectPutQps: number | null
  webdavRawTreeDirectHeadQps: number | null
  webdavRawTreeDirectMkcolQps: number | null
  webdavRawTreeDirectBurst: number | null
  localBaseDir: string
  partSizeMiB: number
  notifyMode: NotifyMode
  notifyWecomBots: string[]
  notifyEmails: string[]

  retentionEnabled: boolean
  retentionKeepLast: number | null
  retentionKeepDays: number | null
  retentionMaxDeletePerTick: number
  retentionMaxDeletePerDay: number
}

export const JOB_EDITOR_STEPS_TOTAL = 6
export const JOB_EDITOR_LAST_INPUT_STEP = 5
