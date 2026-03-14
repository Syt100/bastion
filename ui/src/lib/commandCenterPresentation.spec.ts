import { describe, expect, it } from 'vitest'

import { presentCommandCenterBlocker, presentCommandCenterItem } from './commandCenterPresentation'

const t = (key: string, params?: Record<string, unknown>) => `${key}${params ? JSON.stringify(params) : ''}`

describe('commandCenterPresentation', () => {
  it('localizes machine-like run failures and action labels', () => {
    const presented = presentCommandCenterItem({
      id: 'run:1',
      kind: 'run_failed',
      severity: 'critical',
      title: 'Nightly backup needs review',
      summary: 'run_failed',
      occurred_at: 100,
      scope: 'hub',
      context: {
        job_name: 'Nightly backup',
        error: 'run_failed',
      },
      primary_action: { label: 'Open run', href: '/runs/run-1' },
      secondary_action: { label: 'Open jobs', href: '/jobs?scope=hub' },
    }, t)

    expect(presented.title).toContain('commandCenter.items.run.failed.title')
    expect(presented.summary).toBe('commandCenter.items.run.failed.summary')
    expect(presented.primaryActionLabel).toBe('commandCenter.actions.openRun')
    expect(presented.secondaryActionLabel).toBe('commandCenter.actions.openJobs')
  })

  it('preserves readable backend errors when they add useful context', () => {
    const presented = presentCommandCenterItem({
      id: 'notification:1',
      kind: 'notification_failed',
      severity: 'warning',
      title: 'Notification delivery failed',
      summary: 'smtp delivery failed',
      occurred_at: 100,
      scope: 'hub',
      context: {
        job_name: 'Nightly backup',
        error: 'SMTP delivery failed with 429 Too Many Requests',
        channel: 'email',
      },
      primary_action: { label: 'Open queue', href: '/integrations/notifications/queue' },
    }, t)

    expect(presented.summary).toBe('SMTP delivery failed with 429 Too Many Requests')
    expect(presented.primaryActionLabel).toBe('commandCenter.actions.openQueue')
  })

  it('maps readiness blockers to locale keys', () => {
    const presented = presentCommandCenterBlocker({
      kind: 'missing_verification',
      title: 'Verification signal is missing',
      summary: 'Backups exist, but no successful verify operation has been recorded for this scope.',
      href: '/runs',
    }, t)

    expect(presented.title).toBe('commandCenter.blockers.missing_verification.title')
    expect(presented.summary).toBe('commandCenter.blockers.missing_verification.summary')
  })
})
