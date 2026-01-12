import type { JobEditorField, JobEditorForm } from './types'

export function createInitialJobEditorForm(): JobEditorForm {
  return {
    id: null,
    name: '',
    node: 'hub',
    schedule: '',
    overlapPolicy: 'queue',
    jobType: 'filesystem',
    encryptionEnabled: false,
    encryptionKeyName: 'default',
    fsPaths: [],
    fsInclude: '',
    fsExclude: '',
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
  }
}

export function resetJobEditorForm(target: JobEditorForm): void {
  Object.assign(target, createInitialJobEditorForm())
}

export function createInitialJobEditorFieldErrors(): Record<JobEditorField, string | null> {
  return {
    name: null,
    schedule: null,
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

