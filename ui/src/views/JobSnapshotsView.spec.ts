// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'show', 'loading', 'columns', 'data', 'title', 'subtitle'],
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
            onClick: (attrs as any).onClick,
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

  return {
    NButton: button,
    NCard: card,
    NCheckbox: stub('NCheckbox'),
    NCode: stub('NCode'),
    NDataTable: stub('NDataTable'),
    NInput: stub('NInput'),
    NModal: modal,
    NSpace: stub('NSpace'),
    NSpin: stub('NSpin'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
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
  getJobSnapshotDeleteTask: vi.fn().mockResolvedValue(null),
  getJobSnapshotDeleteEvents: vi.fn().mockResolvedValue([]),
  retryJobSnapshotDeleteNow: vi.fn().mockResolvedValue(undefined),
  ignoreJobSnapshotDeleteTask: vi.fn().mockResolvedValue(undefined),
}
vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
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
    stubMatchMedia(true)
  })

  it('loads job and snapshots on mount', async () => {
    mount(JobSnapshotsView)
    expect(jobsApi.getJob).toHaveBeenCalledWith('j1')
    await flushPromises()
    expect(jobsApi.listJobSnapshots).toHaveBeenCalledWith('j1', { limit: 200 })
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

    expect(jobsApi.deleteJobSnapshot).toHaveBeenCalledWith('j1', 'r1')
  })
})
