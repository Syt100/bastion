import type { JobEditorField, JobEditorForm } from './types'

export type JobEditorValidationIssue = {
  field: JobEditorField
  message: string
}

type TranslateFn = (key: string, params?: Record<string, unknown>) => string

function cronLooksValid(expr: string): boolean {
  const trimmed = expr.trim()
  if (!trimmed) return true
  const parts = trimmed.split(/\s+/g).filter(Boolean)
  if (parts.length !== 5) return false
  return parts.every((part) => /^[0-9*/,\-]+$/.test(part))
}

export function stepForJobEditorField(field: JobEditorField): number {
  switch (field) {
    case 'name':
    case 'scheduleTimezone':
    case 'schedule':
    case 'retentionKeepLast':
    case 'retentionKeepDays':
    case 'retentionMaxDeletePerTick':
    case 'retentionMaxDeletePerDay':
      return 1
    case 'fsPaths':
    case 'fsSnapshotMode':
    case 'fsSnapshotProvider':
    case 'fsConsistencyPolicy':
    case 'fsConsistencyFailThreshold':
    case 'fsUploadOnConsistencyFailure':
    case 'sqlitePath':
    case 'vaultwardenDataDir':
    case 'vaultwardenConsistencyPolicy':
    case 'vaultwardenConsistencyFailThreshold':
    case 'vaultwardenUploadOnConsistencyFailure':
      return 2
    case 'webdavBaseUrl':
    case 'webdavSecretName':
    case 'webdavRawTreeDirectMode':
    case 'webdavRawTreeDirectConcurrency':
    case 'webdavRawTreeDirectPutQps':
    case 'webdavRawTreeDirectHeadQps':
    case 'webdavRawTreeDirectMkcolQps':
    case 'webdavRawTreeDirectBurst':
    case 'localBaseDir':
    case 'partSizeMiB':
      return 3
    case 'encryptionKeyName':
      return 4
  }
}

function validateStep(step: number, form: JobEditorForm, t: TranslateFn): JobEditorValidationIssue[] {
  const issues: JobEditorValidationIssue[] = []

  if (step === 1) {
    if (!form.name.trim()) {
      issues.push({ field: 'name', message: t('errors.jobNameRequired') })
    }
    if (!form.scheduleTimezone.trim()) {
      issues.push({ field: 'scheduleTimezone', message: t('errors.scheduleTimezoneRequired') })
    }
    if (form.scheduleMode === 'cron' && !form.schedule.trim()) {
      issues.push({ field: 'schedule', message: t('errors.cronRequired') })
    }
    if (!cronLooksValid(form.schedule)) {
      issues.push({ field: 'schedule', message: t('errors.invalidCron') })
    }

    if (form.retentionEnabled) {
      const keepLast = typeof form.retentionKeepLast === 'number' ? Math.floor(form.retentionKeepLast) : 0
      const keepDays = typeof form.retentionKeepDays === 'number' ? Math.floor(form.retentionKeepDays) : 0
      if (keepLast <= 0 && keepDays <= 0) {
        issues.push({ field: 'retentionKeepLast', message: t('errors.retentionRuleRequired') })
      }
      if (!Number.isFinite(form.retentionMaxDeletePerTick) || form.retentionMaxDeletePerTick <= 0) {
        issues.push({ field: 'retentionMaxDeletePerTick', message: t('errors.retentionLimitInvalid') })
      }
      if (!Number.isFinite(form.retentionMaxDeletePerDay) || form.retentionMaxDeletePerDay <= 0) {
        issues.push({ field: 'retentionMaxDeletePerDay', message: t('errors.retentionLimitInvalid') })
      }
    }

    return issues
  }

  if (step === 2) {
    if (form.jobType === 'filesystem') {
      if (form.fsPaths.every((p) => !p.trim())) {
        issues.push({ field: 'fsPaths', message: t('errors.sourcePathsRequired') })
      }
      if (form.fsSnapshotMode !== 'off') {
        const selected = form.fsPaths.map((p) => p.trim()).filter(Boolean)
        if (selected.length !== 1) {
          issues.push({ field: 'fsSnapshotMode', message: t('errors.snapshotRequiresSingleSourcePath') })
        }
      }
      if (form.fsConsistencyPolicy === 'fail') {
        if (!Number.isFinite(form.fsConsistencyFailThreshold) || form.fsConsistencyFailThreshold < 0) {
          issues.push({ field: 'fsConsistencyFailThreshold', message: t('errors.consistencyThresholdInvalid') })
        }
      }
    } else if (form.jobType === 'sqlite') {
      if (!form.sqlitePath.trim()) {
        issues.push({ field: 'sqlitePath', message: t('errors.sqlitePathRequired') })
      }
    } else {
      if (!form.vaultwardenDataDir.trim()) {
        issues.push({ field: 'vaultwardenDataDir', message: t('errors.vaultwardenDataDirRequired') })
      }
      if (form.vaultwardenConsistencyPolicy === 'fail') {
        if (!Number.isFinite(form.vaultwardenConsistencyFailThreshold) || form.vaultwardenConsistencyFailThreshold < 0) {
          issues.push({ field: 'vaultwardenConsistencyFailThreshold', message: t('errors.consistencyThresholdInvalid') })
        }
      }
    }
    return issues
  }

  if (step === 3) {
    if (form.targetType === 'webdav') {
      if (!form.webdavBaseUrl.trim()) {
        issues.push({ field: 'webdavBaseUrl', message: t('errors.webdavBaseUrlRequired') })
      }
      if (!form.webdavSecretName.trim()) {
        issues.push({ field: 'webdavSecretName', message: t('errors.webdavSecretRequired') })
      }
    } else {
      if (!form.localBaseDir.trim()) {
        issues.push({ field: 'localBaseDir', message: t('errors.localBaseDirRequired') })
      }
    }

    if (!Number.isFinite(form.partSizeMiB) || form.partSizeMiB <= 0) {
      issues.push({ field: 'partSizeMiB', message: t('errors.partSizeInvalid') })
    }

    const showDirect =
      form.jobType === 'filesystem' && form.targetType === 'webdav' && form.artifactFormat === 'raw_tree_v1'
    if (showDirect && form.webdavRawTreeDirectMode !== 'off') {
      const concurrency = Math.floor(form.webdavRawTreeDirectConcurrency || 0)
      if (!Number.isFinite(concurrency) || concurrency < 1 || concurrency > 128) {
        issues.push({
          field: 'webdavRawTreeDirectConcurrency',
          message: t('errors.webdavRawTreeDirectConcurrencyInvalid'),
        })
      }

      function validateOptionalPositiveInt(
        value: number | null,
        field: JobEditorField,
        max: number,
        messageKey: string,
      ): void {
        if (value == null) return
        const n = Math.floor(value)
        if (!Number.isFinite(n) || n < 1 || n > max) {
          issues.push({ field, message: t(messageKey, { max }) })
        }
      }

      validateOptionalPositiveInt(form.webdavRawTreeDirectPutQps, 'webdavRawTreeDirectPutQps', 10000, 'errors.webdavRawTreeDirectQpsInvalid')
      validateOptionalPositiveInt(form.webdavRawTreeDirectHeadQps, 'webdavRawTreeDirectHeadQps', 10000, 'errors.webdavRawTreeDirectQpsInvalid')
      validateOptionalPositiveInt(form.webdavRawTreeDirectMkcolQps, 'webdavRawTreeDirectMkcolQps', 10000, 'errors.webdavRawTreeDirectQpsInvalid')
      validateOptionalPositiveInt(form.webdavRawTreeDirectBurst, 'webdavRawTreeDirectBurst', 100000, 'errors.webdavRawTreeDirectBurstInvalid')
    }

    return issues
  }

  if (step === 4) {
    if (form.encryptionEnabled && !form.encryptionKeyName.trim()) {
      issues.push({ field: 'encryptionKeyName', message: t('errors.encryptionKeyNameRequired') })
    }
    return issues
  }

  return issues
}

export function validateJobEditorUpToStep(maxStep: number, form: JobEditorForm, t: TranslateFn): JobEditorValidationIssue[] {
  const issues: JobEditorValidationIssue[] = []
  const upper = Math.min(4, Math.max(1, maxStep))
  for (let s = 1; s <= upper; s += 1) {
    issues.push(...validateStep(s, form, t))
  }
  return issues
}
