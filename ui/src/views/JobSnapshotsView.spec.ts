// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
}

const i18nDict: Record<string, string> = {}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'show', 'loading', 'columns', 'data', 'title', 'subtitle', 'options'],
      emits: ['update:value', 'update:show'],
      setup(_, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    setup(_, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            onClick: (attrs as { onClick?: ((evt: MouseEvent) => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  const card = vue.defineComponent({
    name: 'NCard',
    setup(_, { slots }) {
      return () => vue.h('div', { 'data-stub': 'NCard' }, [slots.header?.(), slots.default?.(), slots.footer?.()])
    },
  })

  const modal = vue.defineComponent({
    name: 'NModal',
    props: ['show'],
    emits: ['update:show'],
    setup(props, { slots }) {
      return () => (props.show ? vue.h('div', { 'data-stub': 'NModal' }, [slots.header?.(), slots.default?.(), slots.footer?.()]) : null)
    },
  })

  const dropdown = vue.defineComponent({
    name: 'NDropdown',
    props: ['options'],
    emits: ['select'],
    setup(props, { slots, emit }) {
      return () =>
        vue.h('div', { 'data-stub': 'NDropdown' }, [
          slots.default?.(),
          ...(() => {
            const raw = (props as { options?: unknown }).options
            const list = Array.isArray(raw) ? raw : []
            return list
              .filter((o): o is Record<string, unknown> => !!o && typeof o === 'object')
              .filter((o) => o.type !== 'divider' && 'key' in o)
              .map((o) => {
                const key = typeof o.key === 'string' || typeof o.key === 'number' ? o.key : String(o.key)
                const label = typeof o.label === 'string' ? o.label : String(key)
                return vue.h('button', { onClick: () => emit('select', key) }, label)
              })
          })(),
        ])
    },
  })

  return {
    NButton: button,
    NCard: card,
    NCheckbox: stub('NCheckbox'),
    NCode: stub('NCode'),
    NDataTable: stub('NDataTable'),
    NDropdown: dropdown,
    NIcon: stub('NIcon'),
    NInput: stub('NInput'),
    NModal: modal,
    NPopover: stub('NPopover'),
    NSelect: stub('NSelect'),
    NSpace: stub('NSpace'),
    NSpin: stub('NSpin'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => i18nDict[key] ?? key,
  }),
}))

const routeApi = {
  params: { nodeId: 'hub', jobId: 'j1' } as Record<string, unknown>,
}
const routerApi = {
  push: vi.fn(),
}
vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

const jobsApi = {
  getJob: vi.fn().mockResolvedValue({ id: 'j1', name: 'Job', agent_id: null, schedule: null, schedule_timezone: 'UTC', overlap_policy: 'queue', created_at: 0, updated_at: 0, spec: { v: 1, type: 'filesystem' } }),
  listJobSnapshots: vi.fn().mockResolvedValue({ items: [], next_cursor: null }),
  deleteJobSnapshot: vi.fn().mockResolvedValue(undefined),
  deleteJobSnapshotsBulk: vi.fn().mockResolvedValue(undefined),
  pinJobSnapshot: vi.fn().mockResolvedValue(undefined),
  unpinJobSnapshot: vi.fn().mockResolvedValue(undefined),
  getJobSnapshotDeleteTask: vi.fn().mockResolvedValue(null),
  getJobSnapshotDeleteEvents: vi.fn().mockResolvedValue([]),
  retryJobSnapshotDeleteNow: vi.fn().mockResolvedValue(undefined),
  ignoreJobSnapshotDeleteTask: vi.fn().mockResolvedValue(undefined),
}
vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

const agentsApi = {
  items: [],
}
vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'zh-CN' }),
}))

import JobSnapshotsView from './JobSnapshotsView.vue'

function stubMatchMedia(matches: boolean): void {
  vi.stubGlobal(
    'matchMedia',
    ((query: string) => ({
      matches,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })) as unknown as typeof window.matchMedia,
  )
}

describe('JobSnapshotsView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    for (const key of Object.keys(i18nDict)) delete i18nDict[key]
    stubMatchMedia(true)
  })

  it('loads job and snapshots on mount', async () => {
    mount(JobSnapshotsView)
    expect(jobsApi.getJob).toHaveBeenCalledWith('j1')
    await flushPromises()
    expect(jobsApi.listJobSnapshots).toHaveBeenCalledWith('j1', expect.objectContaining({ limit: 50 }))
    expect(jobsApi.listJobSnapshots.mock.calls[0]?.[1]).not.toHaveProperty('cursor')
  })

  it('shows desktop table when viewport is >= md', () => {
    stubMatchMedia(true)
    const wrapper = mount(JobSnapshotsView)
    expect(wrapper.findComponent({ name: 'NDataTable' }).exists()).toBe(true)
  })

  it('wires delete flow via confirmation modal (mobile layout)', async () => {
    stubMatchMedia(false)
    jobsApi.listJobSnapshots.mockResolvedValueOnce({
      items: [
        {
          run_id: 'r1',
          job_id: 'j1',
          node_id: 'hub',
          target_type: 'local_dir',
          target_snapshot: { node_id: 'hub', target: { type: 'local_dir', base_dir: '/tmp' } },
          artifact_format: 'archive_v1',
          status: 'present',
          started_at: 0,
          ended_at: 10,
          transfer_bytes: 0,
        },
      ],
      next_cursor: null,
    })

    const wrapper = mount(JobSnapshotsView)
    await flushPromises()

    const deleteBtn = wrapper.findAll('button').find((b) => b.text() === 'snapshots.actions.delete')
    expect(deleteBtn).toBeTruthy()
    await deleteBtn!.trigger('click')
    await flushPromises()

    const confirmBtn = wrapper
      .findAll('button')
      .find((b) => b.text() === 'snapshots.actions.confirmDelete')
    expect(confirmBtn).toBeTruthy()
    await confirmBtn!.trigger('click')
    await flushPromises()

    expect(jobsApi.deleteJobSnapshot).toHaveBeenCalledWith('j1', 'r1', { force: false })
  })

  it('shows active filter chips and clears them from shared filter model', async () => {
    const wrapper = mount(JobSnapshotsView)
    await flushPromises()

    const selects = wrapper.findAllComponents({ name: 'NSelect' })
    expect(selects.length).toBeGreaterThanOrEqual(2)

    selects[1]!.vm.$emit('update:value', 'pinned')
    await flushPromises()

    expect(wrapper.text()).toContain('snapshots.filters.pinnedOnly')

    const clearBtn = wrapper.findAll('button').find((b) => b.text() === 'common.clear')
    expect(clearBtn).toBeTruthy()
    await clearBtn!.trigger('click')
    await flushPromises()

    expect(wrapper.text()).not.toContain('snapshots.filters.pinnedOnly')
  })

  it('prefers envelope diagnostics in the delete log modal', async () => {
    stubMatchMedia(false)
    i18nDict['diagnostics.message.execute.snapshot_cleanup_failed'] = 'Snapshot cleanup failed'
    i18nDict['diagnostics.hint.execute.snapshot_cleanup_failed'] = 'Clean up stale snapshots'

    jobsApi.listJobSnapshots.mockResolvedValueOnce({
      items: [
        {
          run_id: 'r1',
          job_id: 'j1',
          node_id: 'hub',
          target_type: 'local_dir',
          target_snapshot: { target: { type: 'local_dir', base_dir: '/tmp' } },
          artifact_format: 'archive_v1',
          status: 'present',
          started_at: 0,
          ended_at: 10,
          transfer_bytes: 0,
          delete_task: {
            status: 'retrying',
            attempts: 2,
            next_attempt_at: 20,
            last_error_kind: 'network',
            last_error: 'legacy delete error',
          },
        },
      ],
      next_cursor: null,
    })
    jobsApi.getJobSnapshotDeleteTask.mockResolvedValueOnce({
      run_id: 'r1',
      job_id: 'j1',
      node_id: 'hub',
      target_snapshot: {},
      target_type: 'local_dir',
      status: 'retrying',
      attempts: 2,
      created_at: 1,
      updated_at: 2,
      next_attempt_at: 20,
      last_error_kind: 'network',
      last_error: 'legacy delete error',
      ignored_at: null,
      ignored_by_user_id: null,
      ignore_reason: null,
    })
    jobsApi.getJobSnapshotDeleteEvents.mockResolvedValueOnce([
      {
        run_id: 'r1',
        seq: 1,
        ts: 1,
        level: 'warn',
        kind: 'failed',
        message: 'legacy event',
        fields: {
          error_envelope: {
            code: 'scheduler.execute.filesystem.snapshot_cleanup_failed',
            kind: 'io',
            retriable: { value: false, reason: null, retry_after_sec: null },
            hint: { key: 'diagnostics.hint.execute.snapshot_cleanup_failed', params: {} },
            message: { key: 'diagnostics.message.execute.snapshot_cleanup_failed', params: {} },
            transport: { protocol: 'file' },
          },
        },
      },
    ])

    const wrapper = mount(JobSnapshotsView)
    await flushPromises()

    const deleteLogBtn = wrapper.findAll('button').find((button) => button.text() === 'snapshots.actions.deleteLog')
    expect(deleteLogBtn).toBeTruthy()
    await deleteLogBtn!.trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain('Snapshot cleanup failed')
    expect(wrapper.text()).toContain('Clean up stale snapshots')
    expect(wrapper.text()).toContain('snapshots.deleteLog.lastErrorioSnapshot cleanup failedClean up stale snapshots')
  })
})
