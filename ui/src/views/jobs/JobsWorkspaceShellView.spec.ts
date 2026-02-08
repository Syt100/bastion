// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { reactive } from 'vue'

import type { JobListItem } from '@/stores/jobs'
import type { JobsWorkspaceLayoutMode, JobsWorkspaceListView } from '@/stores/ui'

const messageApi = {
  error: vi.fn(),
  warning: vi.fn(),
  success: vi.fn(),
}

const jobsStore = reactive({
  items: [] as JobListItem[],
  loading: false,
  total: 0,
  page: 1,
  pageSize: 20,
  refresh: vi.fn().mockResolvedValue(undefined),
  runNow: vi.fn().mockResolvedValue({ run_id: 'r1', status: 'success' }),
  archiveJob: vi.fn().mockResolvedValue(undefined),
  unarchiveJob: vi.fn().mockResolvedValue(undefined),
})

const agentsStore = reactive({
  items: [] as Array<{ id: string; name: string | null; online: boolean; revoked: boolean }>,
  refresh: vi.fn().mockResolvedValue(undefined),
})

const uiStore = reactive({
  jobsWorkspaceLayoutMode: 'split' as JobsWorkspaceLayoutMode,
  jobsWorkspaceListView: 'list' as JobsWorkspaceListView,
  jobsWorkspaceSplitListWidthPx: 360,
  setJobsWorkspaceLayoutMode: vi.fn(),
  setJobsWorkspaceListView: vi.fn(),
  setJobsWorkspaceSplitListWidthPx: vi.fn(),
})

const routeApi = reactive<{ params: Record<string, unknown>; path: string }>({ params: {}, path: '' })
const routerApi = { push: vi.fn() }

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: [
        'show',
        'value',
        'options',
        'bordered',
        'size',
        'checked',
        'checkedRowKeys',
        'rowKey',
        'columns',
        'data',
        'loading',
      ],
      emits: ['update:show', 'update:value', 'update:expanded-keys', 'update:checked', 'update:checked-row-keys', 'update:sorter', 'select', 'close'],
      setup(_props, { slots, attrs }) {
        return () => vue.h('div', { 'data-stub': name, ...attrs }, slots.default?.())
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    setup(_props, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            disabled: Boolean((attrs as { disabled?: unknown }).disabled),
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  const tag = vue.defineComponent({
    name: 'NTag',
    props: {
      closable: Boolean,
      size: String,
      bordered: Boolean,
      type: String,
    },
    emits: ['close'],
    setup(props, { slots, emit, attrs }) {
      return () =>
        vue.h('span', { 'data-stub': 'NTag', ...attrs }, [
          slots.default?.(),
          props.closable ? vue.h('button', { 'data-testid': 'tag-close', onClick: () => emit('close') }, 'x') : null,
        ])
    },
  })

  const modal = vue.defineComponent({
    name: 'NModal',
    props: ['show'],
    emits: ['update:show'],
    setup(props, { slots }) {
      return () => {
        if (!props.show) return vue.h('div', { 'data-stub': 'NModal' })
        return vue.h('div', { 'data-stub': 'NModal' }, slots.default?.())
      }
    },
  })

  const checkbox = vue.defineComponent({
    name: 'NCheckbox',
    props: ['checked'],
    emits: ['update:checked'],
    setup(props, { emit, attrs, slots }) {
      return () =>
        vue.h('label', { 'data-stub': 'NCheckbox' }, [
          vue.h('input', {
            type: 'checkbox',
            checked: Boolean(props.checked),
            onChange: (e: Event) => emit('update:checked', (e.target as HTMLInputElement).checked),
            ...(attrs as Record<string, unknown>),
          }),
          slots.default?.(),
        ])
    },
  })

  return {
    NBadge: stub('NBadge'),
    NButton: button,
    NCard: stub('NCard'),
    NDataTable: stub('NDataTable'),
    NCheckbox: checkbox,
    NDrawer: stub('NDrawer'),
    NDrawerContent: stub('NDrawerContent'),
    NIcon: stub('NIcon'),
    NInput: stub('NInput'),
    NModal: modal,
    NPagination: stub('NPagination'),
    NPopover: stub('NPopover'),
    NRadioButton: stub('NRadioButton'),
    NRadioGroup: stub('NRadioGroup'),
    NSelect: stub('NSelect'),
    NDropdown: stub('NDropdown'),
    NSwitch: stub('NSwitch'),
    NTag: tag,
    useMessage: () => messageApi,
  }
})

vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('vue-i18n', async () => {
  const vue = await import('vue')
  return {
    useI18n: () => ({ t: (key: string) => key }),
    createI18n: () => ({
      global: {
        locale: vue.ref('en-US'),
        t: (key: string) => key,
      },
    }),
  }
})

vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsStore,
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsStore,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => uiStore,
}))

import JobsWorkspaceShellView from './JobsWorkspaceShellView.vue'

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

describe('JobsWorkspaceShellView desktop scrolling', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
    routeApi.path = '/n/hub/jobs'
    routeApi.params = { nodeId: 'hub' }

    uiStore.jobsWorkspaceLayoutMode = 'split'
    uiStore.jobsWorkspaceListView = 'list'
    uiStore.setJobsWorkspaceLayoutMode = vi.fn((v: unknown) => {
      uiStore.jobsWorkspaceLayoutMode = v as JobsWorkspaceLayoutMode
    })
    uiStore.setJobsWorkspaceListView = vi.fn((v: unknown) => {
      uiStore.jobsWorkspaceListView = v as JobsWorkspaceListView
    })

    jobsStore.items = [
      {
        id: 'job1',
        name: 'Job 1',
        agent_id: null,
        schedule: null,
        schedule_timezone: 'UTC',
        overlap_policy: 'queue',
        created_at: 1,
        updated_at: 1,
        archived_at: null,
        latest_run_id: null,
        latest_run_status: null,
        latest_run_started_at: null,
        latest_run_ended_at: null,
      },
    ]
    jobsStore.loading = false
    jobsStore.total = jobsStore.items.length
  })

  it('shows pagination when filtered jobs exceed one page', () => {
    jobsStore.total = 25
    jobsStore.items = Array.from({ length: 25 }, (_, idx) => ({
      id: `job-${idx}`,
      name: `Job ${idx}`,
      agent_id: null,
      schedule: null,
      schedule_timezone: 'UTC',
      overlap_policy: 'queue',
      created_at: idx + 1,
      updated_at: idx + 1,
      archived_at: null,
      latest_run_id: null,
      latest_run_status: null,
      latest_run_started_at: null,
      latest_run_ended_at: null,
    }))

    const wrapper = mount(JobsWorkspaceShellView, {
      global: {
        stubs: {
          PageHeader: true,
          NodeContextTag: true,
          AppEmptyState: true,
          ListToolbar: true,
          JobEditorModal: true,
          'router-view': true,
        },
      },
    })

    expect(wrapper.find('[data-stub="NPagination"]').exists()).toBe(true)
  })

  it('renders a scrollable job list container on desktop', () => {
    const wrapper = mount(JobsWorkspaceShellView, {
      global: {
        stubs: {
          PageHeader: true,
          NodeContextTag: true,
          AppEmptyState: true,
          ListToolbar: true,
          JobEditorModal: true,
          'router-view': true,
        },
      },
    })

    const list = wrapper.find('[data-testid="jobs-list-scroll"]')
    expect(list.exists()).toBe(true)
    expect(list.classes()).toContain('overflow-y-auto')
  })

  it('hides the job workspace pane in list-only layout', () => {
    routeApi.params = { nodeId: 'hub', jobId: 'job1' }
    uiStore.jobsWorkspaceLayoutMode = 'list'

    const wrapper = mount(JobsWorkspaceShellView, {
      global: {
        stubs: {
          PageHeader: true,
          NodeContextTag: true,
          AppEmptyState: true,
          ListToolbar: true,
          JobEditorModal: true,
          'router-view': true,
        },
      },
    })

    expect(wrapper.find('router-view-stub').exists()).toBe(false)
    expect(wrapper.find('[data-testid="jobs-list-scroll"]').exists()).toBe(true)
  })

  it('selecting table view forces list-only layout', async () => {
    const wrapper = mount(JobsWorkspaceShellView, {
      global: {
        stubs: {
          NodeContextTag: true,
          AppEmptyState: true,
          JobEditorModal: true,
          'router-view': true,
        },
      },
    })

    const groups = wrapper.findAllComponents({ name: 'NRadioGroup' })
    expect(groups.length).toBeGreaterThanOrEqual(2)
    groups[1]!.vm.$emit('update:value', 'table')

    expect(uiStore.setJobsWorkspaceLayoutMode).toHaveBeenCalledWith('list')
    expect(uiStore.setJobsWorkspaceListView).toHaveBeenCalledWith('table')
  })

  it('renders results count and active filter chips; closing a chip clears the filter', async () => {
    const wrapper = mount(JobsWorkspaceShellView, {
      global: {
        stubs: {
          PageHeader: true,
          NodeContextTag: true,
          AppEmptyState: true,
          JobEditorModal: true,
          // Render the toolbar named slots so search input exists.
          ListToolbar: { template: '<div><slot name=\"search\" /><slot name=\"filters\" /><slot name=\"sort\" /><slot name=\"actions\" /></div>' },
          'router-view': true,
        },
      },
    })

    expect(wrapper.text()).toContain('jobs.workspace.filters.resultsCount')

    const input = wrapper.findComponent({ name: 'NInput' })
    input.vm.$emit('update:value', 'abc')
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('common.search: abc')

    const close = wrapper.find('[data-testid=\"tag-close\"]')
    expect(close.exists()).toBe(true)
    await close.trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).not.toContain('common.search: abc')
  })

  it('bulk run now skips archived jobs', async () => {
    uiStore.jobsWorkspaceLayoutMode = 'list'
    uiStore.jobsWorkspaceListView = 'table'
    jobsStore.items = [
      {
        id: 'job1',
        name: 'Job 1',
        agent_id: null,
        schedule: null,
        schedule_timezone: 'UTC',
        overlap_policy: 'queue',
        created_at: 1,
        updated_at: 1,
        archived_at: null,
        latest_run_id: null,
        latest_run_status: null,
        latest_run_started_at: null,
        latest_run_ended_at: null,
      },
      {
        id: 'job2',
        name: 'Job 2',
        agent_id: null,
        schedule: null,
        schedule_timezone: 'UTC',
        overlap_policy: 'queue',
        created_at: 1,
        updated_at: 1,
        archived_at: 123,
        latest_run_id: null,
        latest_run_status: null,
        latest_run_started_at: null,
        latest_run_ended_at: null,
      },
    ]

    const wrapper = mount(JobsWorkspaceShellView, {
      global: {
        stubs: {
          PageHeader: true,
          NodeContextTag: true,
          AppEmptyState: true,
          ListToolbar: true,
          JobEditorModal: true,
          'router-view': true,
        },
      },
    })

    const table = wrapper.findComponent({ name: 'NDataTable' })
    table.vm.$emit('update:checked-row-keys', ['job1', 'job2'])
    await wrapper.vm.$nextTick()

    const toolbar = wrapper.find('.app-selection-toolbar')
    expect(toolbar.exists()).toBe(true)

    const runBtn = toolbar.findAll('button').find((b) => b.text() === 'jobs.actions.runNow')
    expect(runBtn).toBeTruthy()
    await runBtn!.trigger('click')
    await flushPromises()

    expect(jobsStore.runNow).toHaveBeenCalledTimes(1)
    expect(jobsStore.runNow).toHaveBeenCalledWith('job1')
  })

  it('bulk archive requires confirmation and only archives eligible jobs', async () => {
    uiStore.jobsWorkspaceLayoutMode = 'list'
    uiStore.jobsWorkspaceListView = 'table'
    jobsStore.items = [
      {
        id: 'job1',
        name: 'Job 1',
        agent_id: null,
        schedule: null,
        schedule_timezone: 'UTC',
        overlap_policy: 'queue',
        created_at: 1,
        updated_at: 1,
        archived_at: null,
        latest_run_id: null,
        latest_run_status: null,
        latest_run_started_at: null,
        latest_run_ended_at: null,
      },
      {
        id: 'job2',
        name: 'Job 2',
        agent_id: null,
        schedule: null,
        schedule_timezone: 'UTC',
        overlap_policy: 'queue',
        created_at: 1,
        updated_at: 1,
        archived_at: 123,
        latest_run_id: null,
        latest_run_status: null,
        latest_run_started_at: null,
        latest_run_ended_at: null,
      },
    ]

    const wrapper = mount(JobsWorkspaceShellView, {
      global: {
        stubs: {
          PageHeader: true,
          NodeContextTag: true,
          AppEmptyState: true,
          ListToolbar: true,
          JobEditorModal: true,
          'router-view': true,
        },
      },
    })

    wrapper.findComponent({ name: 'NDataTable' }).vm.$emit('update:checked-row-keys', ['job1', 'job2'])
    await wrapper.vm.$nextTick()

    const toolbar = wrapper.find('.app-selection-toolbar')
    expect(toolbar.exists()).toBe(true)

    const archiveBtn = toolbar.findAll('button').find((b) => b.text() === 'jobs.actions.archive')
    expect(archiveBtn).toBeTruthy()
    await archiveBtn!.trigger('click')
    await wrapper.vm.$nextTick()

    expect(jobsStore.archiveJob).not.toHaveBeenCalled()
    expect(wrapper.text()).toContain('jobs.workspace.bulk.archiveConfirm')

    const modal = wrapper.find('[data-stub=\"NModal\"]')
    const confirm = modal.findAll('button').find((b) => b.text() === 'jobs.actions.archive')
    expect(confirm).toBeTruthy()
    await confirm!.trigger('click')
    await flushPromises()

    expect(jobsStore.archiveJob).toHaveBeenCalledTimes(1)
    expect(jobsStore.archiveJob).toHaveBeenCalledWith('job1', { cascadeSnapshots: false })
  })
})
