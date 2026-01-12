import type { JobType, OverlapPolicy } from '@/stores/jobs'

export type FsSymlinkPolicy = 'keep' | 'follow' | 'skip'
export type FsHardlinkPolicy = 'copy' | 'keep'
export type FsErrorPolicy = 'fail_fast' | 'skip_fail' | 'skip_ok'

export type JobTargetType = 'webdav' | 'local_dir'
export type NotifyMode = 'inherit' | 'custom'
export type NodeIdOrHub = 'hub' | string

export type JobEditorMode = 'create' | 'edit'

export type JobEditorField =
  | 'name'
  | 'schedule'
  | 'fsPaths'
  | 'sqlitePath'
  | 'vaultwardenDataDir'
  | 'webdavBaseUrl'
  | 'webdavSecretName'
  | 'localBaseDir'
  | 'partSizeMiB'
  | 'encryptionKeyName'

export type JobEditorForm = {
  id: string | null
  name: string
  node: NodeIdOrHub
  schedule: string
  overlapPolicy: OverlapPolicy
  jobType: JobType
  encryptionEnabled: boolean
  encryptionKeyName: string
  fsPaths: string[]
  fsInclude: string
  fsExclude: string
  fsSymlinkPolicy: FsSymlinkPolicy
  fsHardlinkPolicy: FsHardlinkPolicy
  fsErrorPolicy: FsErrorPolicy
  sqlitePath: string
  sqliteIntegrityCheck: boolean
  vaultwardenDataDir: string
  targetType: JobTargetType
  webdavBaseUrl: string
  webdavSecretName: string
  localBaseDir: string
  partSizeMiB: number
  notifyMode: NotifyMode
  notifyWecomBots: string[]
  notifyEmails: string[]
}

export const JOB_EDITOR_STEPS_TOTAL = 6
export const JOB_EDITOR_LAST_INPUT_STEP = 5

