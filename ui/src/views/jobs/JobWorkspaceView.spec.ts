// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string, opts?: { respectShow?: boolean }) =>
    vue.defineComponent({
      name,
      props: ['show', 'value', 'loading', 'title', 'bordered', 'size', 'placement', 'width', 'preset', 'style', 'options', 'trigger'],
      emits: ['update:show', 'update:value', 'select'],
      setup(props, { slots }) {
        return () => {
          if (opts?.respectShow && 'show' in props && !(props as { show?: boolean }).show) {
            return vue.h('div', { 'data-stub': name })
          }
          return vue.h('div', { 'data-stub': name }, [slots.header?.(), slots.default?.(), slots.footer?.()])
        }
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
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  return {
    NButton: button,
    NCard: stub('NCard'),
    NCheckbox: stub('NCheckbox'),
    NCode: stub('NCode'),
    NDropdown: stub('NDropdown'),
    NDrawer: stub('NDrawer', { respectShow: true }),
    NDrawerContent: stub('NDrawerContent'),
    NModal: stub('NModal', { respectShow: true }),
    NTabs: stub('NTabs'),
    NTabPane: stub('NTabPane'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

const routeApi = {
  params: {} as Record<string, unknown>,
  path: '',
  query: {} as Record<string, unknown>,
  hash: '',
}
const routerApi = {
  push: vi.fn(),
}
vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

const jobsApi = {
  getJob: vi.fn(),
  runNow: vi.fn(),
  archiveJob: vi.fn(),
  unarchiveJob: vi.fn(),
  deleteJob: vi.fn(),
}
vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

import JobWorkspaceView from './JobWorkspaceView.vue'

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

describe('JobWorkspaceView run drawer routing', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
    routeApi.params = { nodeId: 'hub', jobId: 'job1', runId: 'run1' }
    routeApi.path = '/n/hub/jobs/job1/history/runs/run1'
    routeApi.query = { q: '1' }
    routeApi.hash = '#x'

    jobsApi.getJob.mockResolvedValue({
      id: 'job1',
      name: 'Job 1',
      agent_id: null,
      schedule: null,
      schedule_timezone: 'UTC',
      overlap_policy: 'queue',
      created_at: 1,
      updated_at: 2,
      archived_at: null,
      spec: { v: 1, type: 'filesystem' },
    })
  })

  it('opens the run drawer when runId is present in the route', async () => {
    const wrapper = mount(JobWorkspaceView, {
      global: {
        stubs: {
          NodeContextTag: true,
          MobileTopBar: true,
          JobEditorModal: true,
          JobDeployModal: true,
          RunDetailPanel: { template: '<div data-testid="run-panel" />' },
          'router-view': true,
        },
      },
    })
    await flushPromises()

    expect(wrapper.find('[data-stub="NDrawer"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="run-panel"]').exists()).toBe(true)
  })

  it('closing the run drawer navigates back to the parent section route', async () => {
    const wrapper = mount(JobWorkspaceView, {
      global: {
        stubs: {
          NodeContextTag: true,
          MobileTopBar: true,
          JobEditorModal: true,
          JobDeployModal: true,
          RunDetailPanel: { template: '<div />' },
          'router-view': true,
        },
      },
    })
    await flushPromises()

    const drawer = wrapper.findComponent({ name: 'NDrawer' })
    expect(drawer.exists()).toBe(true)

    drawer.vm.$emit('update:show', false)
    await flushPromises()

    expect(routerApi.push).toHaveBeenCalledWith({ path: '/n/hub/jobs/job1/history', query: { q: '1' }, hash: '#x' })
  })
})
