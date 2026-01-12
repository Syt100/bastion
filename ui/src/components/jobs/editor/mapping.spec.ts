import { describe, expect, it } from 'vitest'

import type { JobDetail } from '@/stores/jobs'

import { editorFormToRequest, jobDetailToEditorForm } from './mapping'
import { createInitialJobEditorForm } from './form'

function createJobDetail(spec: Record<string, unknown>): JobDetail {
  return {
    id: 'job_1',
    name: 'Job 1',
    agent_id: null,
    schedule: null,
    schedule_timezone: 'UTC',
    overlap_policy: 'queue',
    created_at: 0,
    updated_at: 0,
    spec: spec as JobDetail['spec'],
  }
}

describe('jobDetailToEditorForm', () => {
  it('falls back to legacy filesystem root when paths are absent', () => {
    const job = createJobDetail({
      v: 1,
      type: 'filesystem',
      pipeline: { encryption: { type: 'none' } },
      notifications: { mode: 'inherit' },
      source: { root: '/data' },
      target: { type: 'local_dir', base_dir: '/backups', part_size_bytes: 256 * 1024 * 1024 },
    })
    const form = jobDetailToEditorForm(job)
    expect(form.jobType).toBe('filesystem')
    expect(form.fsPaths).toEqual(['/data'])
  })

  it('parses age encryption settings from pipeline', () => {
    const job = createJobDetail({
      v: 1,
      type: 'filesystem',
      pipeline: { encryption: { type: 'age_x25519', key_name: 'my-key' } },
      notifications: { mode: 'inherit' },
      source: { paths: ['/tmp'] },
      target: { type: 'local_dir', base_dir: '/backups', part_size_bytes: 256 * 1024 * 1024 },
    })
    const form = jobDetailToEditorForm(job)
    expect(form.encryptionEnabled).toBe(true)
    expect(form.encryptionKeyName).toBe('my-key')
  })

  it('converts part_size_bytes into MiB', () => {
    const job = createJobDetail({
      v: 1,
      type: 'filesystem',
      pipeline: { encryption: { type: 'none' } },
      notifications: { mode: 'inherit' },
      source: { paths: ['/tmp'] },
      target: { type: 'webdav', base_url: 'https://dav.example.com', secret_name: 's', part_size_bytes: 512 * 1024 * 1024 },
    })
    const form = jobDetailToEditorForm(job)
    expect(form.partSizeMiB).toBe(512)
    expect(form.targetType).toBe('webdav')
  })
})

describe('editorFormToRequest', () => {
  it('normalizes schedule and hub node handling', () => {
    const form = createInitialJobEditorForm()
    form.name = ' Demo '
    form.node = 'hub'
    form.schedule = ' '
    form.jobType = 'sqlite'
    form.sqlitePath = '/tmp/db.sqlite3'
    form.targetType = 'local_dir'
    form.localBaseDir = '/tmp/backups'
    form.partSizeMiB = 10.2

    const req = editorFormToRequest(form)
    expect(req.name).toBe('Demo')
    expect(req.agent_id).toBeNull()
    expect(req.schedule).toBeNull()
    expect(req.schedule_timezone).toBe('UTC')
    const spec = req.spec as Record<string, unknown>
    const target = spec['target'] as Record<string, unknown>
    expect(target['part_size_bytes']).toBe(10 * 1024 * 1024)
  })
})
