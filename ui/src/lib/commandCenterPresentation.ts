import type { CommandCenterItem, CommandCenterReadinessBlocker } from '@/stores/commandCenter'

type TranslateFn = (key: string, params?: Record<string, unknown>) => string

const MACHINE_ERROR_PATTERN = /^[a-z0-9_.:-]+$/i

function readableError(error: string | null | undefined): string | null {
  const normalized = error?.trim()
  if (!normalized) return null
  return MACHINE_ERROR_PATTERN.test(normalized) ? null : normalized
}

function actionLabelKey(label: string): string | null {
  switch (label) {
    case 'Open run':
      return 'commandCenter.actions.openRun'
    case 'Open jobs':
      return 'commandCenter.actions.openJobs'
    case 'Open queue':
      return 'commandCenter.actions.openQueue'
    case 'Open fleet':
      return 'commandCenter.actions.openFleet'
    default:
      return null
  }
}

function jobLabel(item: CommandCenterItem): string {
  return item.context.job_name?.trim() || item.context.job_id || item.title
}

function nodeLabel(item: CommandCenterItem): string {
  return item.context.node_name?.trim() || item.context.node_id || item.scope
}

function notificationChannel(item: CommandCenterItem): string {
  return item.context.channel?.trim() || 'notification'
}

function localizeRunItem(item: CommandCenterItem, status: string, t: TranslateFn) {
  const job = jobLabel(item)
  const summaryOverride = readableError(item.context.error)

  switch (status) {
    case 'failed':
      return {
        title: t('commandCenter.items.run.failed.title', { job }),
        summary: summaryOverride ?? t('commandCenter.items.run.failed.summary'),
      }
    case 'rejected':
      return {
        title: t('commandCenter.items.run.rejected.title', { job }),
        summary: summaryOverride ?? t('commandCenter.items.run.rejected.summary'),
      }
    case 'success':
      return {
        title: t('commandCenter.items.run.success.title', { job }),
        summary: t('commandCenter.items.run.success.summary'),
      }
    case 'running':
      return {
        title: t('commandCenter.items.run.running.title', { job }),
        summary: t('commandCenter.items.run.running.summary'),
      }
    case 'queued':
      return {
        title: t('commandCenter.items.run.queued.title', { job }),
        summary: t('commandCenter.items.run.queued.summary'),
      }
    case 'canceled':
      return {
        title: t('commandCenter.items.run.canceled.title', { job }),
        summary: t('commandCenter.items.run.canceled.summary'),
      }
    default:
      return {
        title: t('commandCenter.items.run.unknown.title', { job }),
        summary: summaryOverride ?? t('commandCenter.items.run.unknown.summary'),
      }
  }
}

function localizeOperationItem(item: CommandCenterItem, operation: string, status: string, t: TranslateFn) {
  const job = jobLabel(item)
  const summaryOverride = readableError(item.context.error)
  const prefix =
    operation === 'verify'
      ? 'commandCenter.items.operation.verify'
      : operation === 'restore'
        ? 'commandCenter.items.operation.restore'
        : null

  if (!prefix) {
    return {
      title: t('commandCenter.items.operation.generic.title', { job }),
      summary: summaryOverride ?? t('commandCenter.items.operation.generic.summary'),
    }
  }

  switch (status) {
    case 'success':
    case 'failed':
    case 'running':
      return {
        title: t(`${prefix}.${status}.title`, { job }),
        summary: summaryOverride ?? t(`${prefix}.${status}.summary`),
      }
    default:
      return {
        title: t(`${prefix}.unknown.title`, { job }),
        summary: summaryOverride ?? t(`${prefix}.unknown.summary`),
      }
  }
}

export function formatCommandCenterScopeLabel(scope: string, t: TranslateFn): string {
  if (scope === 'all') return t('nav.scopePicker.all')
  if (scope === 'hub') return t('nav.scopePicker.hub')
  if (scope.startsWith('agent:')) return scope.slice('agent:'.length)
  return scope
}

export function presentCommandCenterItem(item: CommandCenterItem, t: TranslateFn) {
  let localized = {
    title: item.title,
    summary: readableError(item.summary) ?? item.summary,
  }

  if (item.kind.startsWith('run_')) {
    localized = localizeRunItem(item, item.kind.slice('run_'.length), t)
  } else if (item.kind === 'notification_failed') {
    localized = {
      title: t('commandCenter.items.notification.failed.title', { job: jobLabel(item) }),
      summary: readableError(item.context.error) ?? t('commandCenter.items.notification.failed.summary'),
    }
  } else if (item.kind === 'notification_queued') {
    localized = {
      title: t('commandCenter.items.notification.queued.title', { job: jobLabel(item) }),
      summary: t('commandCenter.items.notification.queued.summary', { channel: notificationChannel(item) }),
    }
  } else if (item.kind === 'agent_offline') {
    localized = {
      title: t('commandCenter.items.agent.offline.title', { node: nodeLabel(item) }),
      summary: t('commandCenter.items.agent.offline.summary'),
    }
  } else if (item.kind === 'agent_revoked') {
    localized = {
      title: t('commandCenter.items.agent.revoked.title', { node: nodeLabel(item) }),
      summary: t('commandCenter.items.agent.revoked.summary'),
    }
  } else if (item.kind.startsWith('operation_')) {
    const [, operation = 'generic', status = 'unknown'] = item.kind.split('_')
    localized = localizeOperationItem(item, operation, status, t)
  }

  const primaryActionKey = actionLabelKey(item.primary_action.label)
  const secondaryActionKey = item.secondary_action ? actionLabelKey(item.secondary_action.label) : null

  return {
    ...localized,
    primaryActionLabel: primaryActionKey ? t(primaryActionKey) : item.primary_action.label,
    secondaryActionLabel: item.secondary_action
      ? secondaryActionKey
        ? t(secondaryActionKey)
        : item.secondary_action.label
      : null,
  }
}

export function presentCommandCenterBlocker(blocker: CommandCenterReadinessBlocker, t: TranslateFn) {
  const prefix = `commandCenter.blockers.${blocker.kind}`
  switch (blocker.kind) {
    case 'missing_backup':
    case 'partial_backup_coverage':
    case 'missing_verification':
    case 'partial_verification_coverage':
    case 'verify_older_than_backup':
    case 'section_unavailable':
      return {
        title: t(`${prefix}.title`),
        summary: t(`${prefix}.summary`),
      }
    default:
      return {
        title: blocker.title,
        summary: readableError(blocker.summary) ?? blocker.summary,
      }
  }
}
