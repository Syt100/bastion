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
    expect(form.artifactFormat).toBe('archive_v1')
    expect(form.fsPaths).toEqual(['/data'])
    expect(form.fsPreScan).toBe(true)
  })

  it('parses filesystem pre_scan and defaults to true when absent', () => {
    const job1 = createJobDetail({
      v: 1,
      type: 'filesystem',
      pipeline: { encryption: { type: 'none' } },
      notifications: { mode: 'inherit' },
      source: { paths: ['/tmp'], pre_scan: false },
      target: { type: 'local_dir', base_dir: '/backups', part_size_bytes: 256 * 1024 * 1024 },
    })
    expect(jobDetailToEditorForm(job1).fsPreScan).toBe(false)

    const job2 = createJobDetail({
      v: 1,
      type: 'filesystem',
      pipeline: { encryption: { type: 'none' } },
      notifications: { mode: 'inherit' },
      source: { paths: ['/tmp'] },
      target: { type: 'local_dir', base_dir: '/backups', part_size_bytes: 256 * 1024 * 1024 },
    })
    expect(jobDetailToEditorForm(job2).fsPreScan).toBe(true)
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

  it('parses raw-tree format and forces encryption off', () => {
    const job = createJobDetail({
      v: 1,
      type: 'filesystem',
      pipeline: { format: 'raw_tree_v1', encryption: { type: 'age_x25519', key_name: 'k' } },
      notifications: { mode: 'inherit' },
      source: { paths: ['/tmp'] },
      target: { type: 'local_dir', base_dir: '/backups', part_size_bytes: 256 * 1024 * 1024 },
    })
    const form = jobDetailToEditorForm(job)
    expect(form.artifactFormat).toBe('raw_tree_v1')
    expect(form.encryptionEnabled).toBe(false)
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

  it('forces pipeline.encryption=none for raw-tree', () => {
    const form = createInitialJobEditorForm()
    form.name = 'Demo'
    form.jobType = 'filesystem'
    form.fsPaths = ['/tmp']
    form.targetType = 'local_dir'
    form.localBaseDir = '/tmp/backups'
    form.artifactFormat = 'raw_tree_v1'
    form.encryptionEnabled = true
    form.encryptionKeyName = 'my-key'

    const req = editorFormToRequest(form)
    const spec = req.spec as Record<string, unknown>
    const pipeline = spec['pipeline'] as Record<string, unknown>
    expect(pipeline['format']).toBe('raw_tree_v1')
    expect(pipeline['encryption']).toEqual({ type: 'none' })
  })

  it('includes filesystem pre_scan in request spec', () => {
    const form = createInitialJobEditorForm()
    form.name = 'Demo'
    form.jobType = 'filesystem'
    form.fsPaths = ['/tmp']
    form.fsPreScan = false
    form.targetType = 'local_dir'
    form.localBaseDir = '/tmp/backups'

    const req = editorFormToRequest(form)
    const spec = req.spec as Record<string, unknown>
    const source = spec['source'] as Record<string, unknown>
    expect(source['pre_scan']).toBe(false)
  })
})
