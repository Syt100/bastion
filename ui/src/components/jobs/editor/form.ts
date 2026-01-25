import type { JobEditorField, JobEditorForm } from './types'

export function createInitialJobEditorForm(): JobEditorForm {
  return {
    id: null,
    name: '',
    node: 'hub',
    schedule: '',
    scheduleTimezone: 'UTC',
    scheduleMode: 'manual',
    simpleScheduleKind: 'daily',
    simpleEveryMinutes: 15,
    simpleAtHour: 0,
    simpleAtMinute: 0,
    simpleWeekday: 1,
    simpleMonthday: 1,
    overlapPolicy: 'queue',
    jobType: 'filesystem',
    artifactFormat: 'archive_v1',
    encryptionEnabled: false,
    encryptionKeyName: 'default',
    fsPaths: [],
    fsInclude: '',
    fsExclude: '',
    fsPreScan: true,
    fsSymlinkPolicy: 'keep',
    fsHardlinkPolicy: 'copy',
    fsErrorPolicy: 'fail_fast',
    sqlitePath: '',
    sqliteIntegrityCheck: false,
    vaultwardenDataDir: '',
    targetType: 'webdav',
    webdavBaseUrl: '',
    webdavSecretName: '',
    localBaseDir: '',
    partSizeMiB: 256,
    notifyMode: 'inherit',
    notifyWecomBots: [],
    notifyEmails: [],
    retentionEnabled: false,
    retentionKeepLast: null,
    retentionKeepDays: null,
    retentionMaxDeletePerTick: 50,
    retentionMaxDeletePerDay: 200,
  }
}

export function resetJobEditorForm(target: JobEditorForm): void {
  Object.assign(target, createInitialJobEditorForm())
}

export function createInitialJobEditorFieldErrors(): Record<JobEditorField, string | null> {
  return {
    name: null,
    scheduleTimezone: null,
    schedule: null,
    retentionKeepLast: null,
    retentionKeepDays: null,
    retentionMaxDeletePerTick: null,
    retentionMaxDeletePerDay: null,
    fsPaths: null,
    sqlitePath: null,
    vaultwardenDataDir: null,
    webdavBaseUrl: null,
    webdavSecretName: null,
    localBaseDir: null,
    partSizeMiB: null,
    encryptionKeyName: null,
  }
}
