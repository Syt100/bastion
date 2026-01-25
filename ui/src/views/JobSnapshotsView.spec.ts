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

  return {
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NDataTable: stub('NDataTable'),
    NSpace: stub('NSpace'),
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
})
