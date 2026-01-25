// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'show', 'loading', 'columns', 'data'],
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

  const checkbox = vue.defineComponent({
    name: 'NCheckbox',
    props: ['checked'],
    emits: ['update:checked'],
    setup(props, { slots, emit }) {
      return () =>
        vue.h('label', { 'data-stub': 'NCheckbox' }, [
          vue.h('input', {
            type: 'checkbox',
            checked: !!(props as any).checked,
            onChange: (e: Event) => emit('update:checked', (e.target as HTMLInputElement).checked),
          }),
          slots.default?.(),
        ])
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
      return () => ((props as any).show ? vue.h('div', { 'data-stub': 'NModal' }, slots.default?.()) : null)
    },
  })

  return {
    NAlert: stub('NAlert'),
    NBadge: stub('NBadge'),
    NButton: button,
    NCard: card,
    NCheckbox: checkbox,
    NCode: stub('NCode'),
    NDataTable: stub('NDataTable'),
    NDropdown: stub('NDropdown'),
    NDrawer: stub('NDrawer'),
    NDrawerContent: stub('NDrawerContent'),
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NIcon: stub('NIcon'),
    NInput: stub('NInput'),
    NInputNumber: stub('NInputNumber'),
    NModal: modal,
    NPopover: stub('NPopover'),
    NPopconfirm: stub('NPopconfirm'),
    NSelect: stub('NSelect'),
    NRadioButton: stub('NRadioButton'),
    NRadioGroup: stub('NRadioGroup'),
    NSpace: stub('NSpace'),
    NSpin: stub('NSpin'),
    NStep: stub('NStep'),
    NSteps: stub('NSteps'),
    NSwitch: stub('NSwitch'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

const routeApi = {
  params: {} as Record<string, unknown>,
}
const routerApi = {
  push: vi.fn(),
}
vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

const jobsApi = {
  items: [] as unknown[],
  loading: false,
  refresh: vi.fn().mockResolvedValue(undefined),
  listRunEvents: vi.fn(),
  listRuns: vi.fn(),
  runNow: vi.fn(),
  getJob: vi.fn(),
  createJob: vi.fn(),
  updateJob: vi.fn(),
  deleteJob: vi.fn(),
  archiveJob: vi.fn().mockResolvedValue(undefined),
  unarchiveJob: vi.fn().mockResolvedValue(undefined),
}
vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

const operationsApi = {
  getOperation: vi.fn(),
  listEvents: vi.fn(),
}
vi.mock('@/stores/operations', () => ({
  useOperationsStore: () => operationsApi,
}))

const agentsApi = {
  items: [],
  refresh: vi.fn().mockResolvedValue(undefined),
}
vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

const bulkOpsApi = {
  previewJobDeploy: vi.fn(),
  create: vi.fn(),
}
vi.mock('@/stores/bulkOperations', () => ({
  useBulkOperationsStore: () => bulkOpsApi,
}))

const secretsApi = {
  webdav: [],
  refreshWebdav: vi.fn().mockResolvedValue(undefined),
}
vi.mock('@/stores/secrets', () => ({
  useSecretsStore: () => secretsApi,
}))

const notificationsApi = {
  destinations: [],
  refreshDestinations: vi.fn().mockResolvedValue(undefined),
}
vi.mock('@/stores/notifications', () => ({
  useNotificationsStore: () => notificationsApi,
}))

const systemApi = {
  hubTimezone: 'UTC',
}
vi.mock('@/stores/system', () => ({
  useSystemStore: () => systemApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'zh-CN' }),
}))

import JobsView from './JobsView.vue'

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

describe('JobsView responsive lists', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
    routeApi.params = {}
    jobsApi.items = []
  })

  it('shows desktop table when viewport is >= md', () => {
    stubMatchMedia(true)
    const wrapper = mount(JobsView)
    expect(wrapper.find('[data-testid=\"jobs-table\"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid=\"jobs-cards\"]').exists()).toBe(false)
  })

  it('shows mobile cards when viewport is < md', () => {
    stubMatchMedia(false)
    const wrapper = mount(JobsView)
    expect(wrapper.find('[data-testid=\"jobs-cards\"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid=\"jobs-table\"]').exists()).toBe(false)
  })

  it('filters jobs by node context', () => {
    routeApi.params = { nodeId: 'hub' }
    jobsApi.items = [
      { id: '1', name: 'Hub', agent_id: null, schedule: null, overlap_policy: 'queue', updated_at: 0 },
      { id: '2', name: 'Agent', agent_id: 'a1', schedule: null, overlap_policy: 'queue', updated_at: 0 },
    ]

    const wrapper = mount(JobsView)
    const dataTable = wrapper.findComponent({ name: 'NDataTable' })

    expect(dataTable.exists()).toBe(true)
    expect((dataTable.props('data') as unknown[]).length).toBe(1)
    expect(((dataTable.props('data') as unknown[])[0] as { id: string }).id).toBe('1')
  })

  it('hides node column in node context', () => {
    routeApi.params = { nodeId: 'a1' }
    jobsApi.items = [
      { id: '2', name: 'Agent', agent_id: 'a1', schedule: null, overlap_policy: 'queue', updated_at: 0 },
    ]

    const wrapper = mount(JobsView)
    const dataTable = wrapper.findComponent({ name: 'NDataTable' })
    const columns = dataTable.props('columns') as Array<{ key?: string }>

    expect(columns.some((c) => c.key === 'agent_id')).toBe(false)
  })

  it('archives with cascade when archive checkbox is checked (mobile)', async () => {
    stubMatchMedia(false)
    routeApi.params = {}

    jobsApi.items = [
      {
        id: '1',
        name: 'Job',
        agent_id: null,
        schedule: null,
        overlap_policy: 'queue',
        schedule_timezone: 'UTC',
        updated_at: 0,
        archived_at: null,
      },
    ]

    const wrapper = mount(JobsView)

    const deleteBtn = wrapper.findAll('button').find((b) => b.text() === 'common.delete')
    expect(deleteBtn).toBeTruthy()
    await deleteBtn!.trigger('click')
    await wrapper.vm.$nextTick()

    // Toggle cascade checkbox.
    const cb = wrapper.find('input[type=\"checkbox\"]')
    expect(cb.exists()).toBe(true)
    await cb.setValue(true)
    await wrapper.vm.$nextTick()

    const archiveBtn = wrapper.findAll('button').find((b) => b.text() === 'jobs.actions.archive')
    expect(archiveBtn).toBeTruthy()
    await archiveBtn!.trigger('click')

    expect(jobsApi.archiveJob).toHaveBeenCalledWith('1', { cascadeSnapshots: true })
  })
})
