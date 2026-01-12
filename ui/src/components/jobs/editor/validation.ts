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
      return 1
    case 'fsPaths':
    case 'sqlitePath':
    case 'vaultwardenDataDir':
      return 2
    case 'webdavBaseUrl':
    case 'webdavSecretName':
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
    return issues
  }

  if (step === 2) {
    if (form.jobType === 'filesystem') {
      if (form.fsPaths.every((p) => !p.trim())) {
        issues.push({ field: 'fsPaths', message: t('errors.sourcePathsRequired') })
      }
    } else if (form.jobType === 'sqlite') {
      if (!form.sqlitePath.trim()) {
        issues.push({ field: 'sqlitePath', message: t('errors.sqlitePathRequired') })
      }
    } else {
      if (!form.vaultwardenDataDir.trim()) {
        issues.push({ field: 'vaultwardenDataDir', message: t('errors.vaultwardenDataDirRequired') })
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
