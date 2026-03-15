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
  resolve: vi.fn((location: unknown) => {
    if (!location || typeof location !== 'object' || !('path' in location)) {
      return { fullPath: '/' }
    }
    const path = String((location as { path: string }).path)
    const query = (location as { query?: Record<string, string> }).query ?? {}
    const suffix = new URLSearchParams(query).toString()
    return { fullPath: suffix ? `${path}?${suffix}` : path }
  }),
}
vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({
    locale: 'en-US',
    preferredScope: 'all',
    jobsSavedViews: [],
  }),
}))

const jobsApi = {
  getJobWorkspace: vi.fn(),
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
    routeApi.params = { jobId: 'job1', runId: 'run1' }
    routeApi.path = '/jobs/job1/history/runs/run1'
    routeApi.query = { q: '1', scope: 'hub', view: 'failed-recently' }
    routeApi.hash = '#x'

    jobsApi.getJobWorkspace.mockResolvedValue({
      job: {
        id: 'job1',
        name: 'Job 1',
        agent_id: null,
        schedule: null,
        schedule_timezone: 'UTC',
        overlap_policy: 'queue',
        created_at: 1,
        updated_at: 2,
        archived_at: null,
        latest_run_id: 'run1',
        latest_run_status: 'failed',
        latest_run_started_at: 10,
        latest_run_ended_at: 11,
        spec: { v: 1, type: 'filesystem' },
      },
      summary: {
        latest_success_at: 1,
        latest_failure_at: 11,
        latest_run_status: 'failed',
        latest_run_started_at: 10,
        latest_run_ended_at: 11,
        next_run_at: null,
        target_label: 'Local',
        target_type: 'local_dir',
        schedule_label: 'Manual',
      },
      readiness: {
        state: 'warning',
        last_success_at: 1,
      },
      recent_runs: [{ id: 'run1', status: 'failed', started_at: 10, ended_at: 11, error: 'boom' }],
      warnings: ['latest_run_failed'],
      capabilities: {
        can_run_now: true,
        can_edit: true,
        can_archive: true,
        can_unarchive: false,
        can_delete: true,
        can_deploy: true,
      },
    })
  })

  it('uses an internal scroll region for section content on desktop', async () => {
    const wrapper = mount(JobWorkspaceView, {
      global: {
        stubs: {
          AppEmptyState: true,
          JobWorkspaceSupportPane: true,
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

    expect(wrapper.find('[data-testid="job-section-scroll"]').classes()).toContain('overflow-y-auto')
  })

  it('opens the run drawer when runId is present in the route', async () => {
    const wrapper = mount(JobWorkspaceView, {
      global: {
        stubs: {
          AppEmptyState: true,
          JobWorkspaceSupportPane: true,
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
          AppEmptyState: true,
          JobWorkspaceSupportPane: true,
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

    expect(routerApi.push).toHaveBeenCalledWith({ path: '/jobs/job1/history', query: { q: '1', scope: 'hub', view: 'failed-recently' }, hash: '#x' })
  })

  it('shows compact collection context and object-first actions near the job header', async () => {
    const wrapper = mount(JobWorkspaceView, {
      global: {
        stubs: {
          AppEmptyState: true,
          JobWorkspaceSupportPane: true,
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

    expect(wrapper.find('[data-testid="job-workspace-context-row"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="job-workspace-object-header"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('jobs.workspace.actions.backToList')
    expect(wrapper.text()).toContain('jobs.workspace.support.openLatestRun')
  })
})
