// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { createNaiveButtonStub, createNaiveInputStub, createNaiveStub } from '@/test-utils/naiveUiStubs'
import type { FleetListResponse } from '@/stores/fleet'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  info: vi.fn(),
}

const agentsApi = {
  listLabelIndex: vi.fn().mockResolvedValue([]),
  createEnrollmentToken: vi.fn(),
}

const controlPlaneApi = {
  getPublicMetadata: vi.fn(),
}

const fleetApi = {
  list: vi.fn(),
}

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/bulkOperations', () => ({
  useBulkOperationsStore: () => ({ create: vi.fn() }),
}))

vi.mock('@/stores/controlPlane', () => ({
  useControlPlaneStore: () => controlPlaneApi,
}))

vi.mock('@/stores/fleet', () => ({
  useFleetStore: () => fleetApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('@/components/AppEmptyState.vue', () => ({
  default: {
    props: ['title', 'description', 'loading'],
    template:
      '<div data-stub="AppEmptyState"><div>{{ title }}</div><div>{{ description }}</div><slot /><slot name="actions" /></div>',
  },
}))

vi.mock('@/components/list/AppPagination.vue', () => ({
  default: {
    props: ['page', 'pageSize', 'itemCount', 'totalLabel'],
    template: '<div data-stub="AppPagination">{{ totalLabel }}</div>',
  },
}))

vi.mock('@/lib/clipboard', () => ({
  copyText: vi.fn().mockResolvedValue(true),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

const routerApi = {
  push: vi.fn(),
}
const routeApi = {
  query: {} as Record<string, unknown>,
}

const mediaState = {
  desktop: true,
}

vi.mock('vue-router', () => ({
  useRouter: () => routerApi,
  useRoute: () => routeApi,
}))

vi.mock('@/lib/media', async () => {
  const vue = await import('vue')
  return { useMediaQuery: () => vue.computed(() => mediaState.desktop) }
})

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const modal = vue.defineComponent({
    name: 'NModal',
    props: ['show'],
    emits: ['update:show'],
    setup(props, { slots }) {
      return () =>
        (props as { show?: boolean }).show
          ? vue.h('div', { 'data-stub': 'NModal' }, [slots.default?.(), slots.footer?.()])
          : null
    },
  })

  const formItem = vue.defineComponent({
    name: 'NFormItem',
    props: ['label'],
    setup(props, { slots }) {
      return () =>
        vue.h('div', { 'data-stub': 'NFormItem' }, [
          props.label ? vue.h('div', { 'data-stub': 'NFormItemLabel' }, String(props.label)) : null,
          slots.default?.(),
        ])
    },
  })

  const stub = (name: string) =>
    createNaiveStub(name, {
      props: ['value', 'show', 'loading', 'columns', 'data', 'options', 'title', 'subtitle'],
      emits: ['update:value', 'update:show', 'update:checked', 'update:checked-row-keys'],
    })

  return {
    NAlert: stub('NAlert'),
    NButton: createNaiveButtonStub(),
    NCard: stub('NCard'),
    NCheckbox: stub('NCheckbox'),
    NDataTable: stub('NDataTable'),
    NForm: stub('NForm'),
    NFormItem: formItem,
    NInput: createNaiveInputStub(),
    NInputNumber: stub('NInputNumber'),
    NModal: modal,
    NPagination: stub('NPagination'),
    NPopconfirm: stub('NPopconfirm'),
    NRadioButton: stub('NRadioButton'),
    NRadioGroup: stub('NRadioGroup'),
    NSelect: stub('NSelect'),
    NSpin: stub('NSpin'),
    NSpace: stub('NSpace'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

import AgentsView from './AgentsView.vue'

function buildFleetResponse(overrides: Partial<FleetListResponse> = {}): FleetListResponse {
  return {
    summary: { total: 0, online: 0, offline: 0, revoked: 0, drifted: 0 },
    onboarding: { public_base_url: 'https://backup.example.com', command_generation_ready: true },
    items: [],
    page: 1,
    page_size: 20,
    total: 0,
    ...overrides,
  }
}

describe('AgentsView enrollment token modal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    routeApi.query = {}
    mediaState.desktop = true
    agentsApi.createEnrollmentToken.mockResolvedValue({ token: 'tok1', expires_at: 1234 })
    controlPlaneApi.getPublicMetadata.mockResolvedValue({
      public_base_url: 'https://backup.example.com',
      source: 'db',
      command_generation_ready: true,
    })
    fleetApi.list.mockResolvedValue(buildFleetResponse())
  })

  it('shows pagination when the filtered list is large', async () => {
    fleetApi.list.mockResolvedValue(buildFleetResponse({
      items: Array.from({ length: 25 }, (_, idx) => ({
        id: `agent-${idx}`,
        name: `Agent ${idx}`,
        status: idx % 2 === 0 ? 'online' : 'offline',
        last_seen_at: null,
        labels: [],
        config_sync: { state: 'offline', last_attempt_at: null, last_error_kind: null, last_error: null },
        assigned_jobs_total: 1,
        pending_tasks_total: 0,
      })),
      total: 25,
      page_size: 20,
      summary: { total: 25, online: 13, offline: 12, revoked: 0, drifted: 0 },
    }))

    const wrapper = mount(AgentsView)
    await flushPromises()

    expect(wrapper.find('[data-stub="AppPagination"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('common.paginationRange')
  })

  it('renders an enroll command template that includes hub url and token', async () => {
    const wrapper = mount(AgentsView)
    await flushPromises()

    const openBtn = wrapper.findAll('button').find((b) => b.text() === 'agents.newToken')
    expect(openBtn).toBeTruthy()
    await openBtn!.trigger('click')

    const createBtn = wrapper.findAll('button').find((b) => b.text() === 'agents.tokenModal.create')
    expect(createBtn).toBeTruthy()
    await createBtn!.trigger('click')
    await flushPromises()

    const labels = wrapper.findAll('[data-stub="NFormItemLabel"]').map((n) => n.text())
    expect(labels).toContain('agents.tokenModal.enrollCommand')

    const inputs = wrapper.findAll('textarea[data-stub="NInput"], input[data-stub="NInput"]')
    const values = inputs.map((n) => String((n.element as HTMLInputElement).value))
    const command = values.find((v) => v.includes('bastion agent')) ?? null

    expect(command).not.toBeNull()
    expect(command!).toContain('--hub-url')
    expect(command!).toContain('--enroll-token tok1')
    expect(command!).toContain('https://backup.example.com')
  })

  it('renders mobile progressive disclosure affordance for secondary metadata', async () => {
    mediaState.desktop = false
    fleetApi.list.mockResolvedValue(buildFleetResponse({
      items: [
        {
        id: 'agent-mobile-1',
        name: 'Mobile Agent',
        status: 'online',
        last_seen_at: 1234,
        labels: ['edge'],
        config_sync: { state: 'offline', last_attempt_at: null, last_error_kind: null, last_error: null },
        assigned_jobs_total: 2,
        pending_tasks_total: 1,
        },
      ],
      total: 1,
      summary: { total: 1, online: 1, offline: 0, revoked: 0, drifted: 1 },
    }))

    const wrapper = mount(AgentsView)
    await flushPromises()

    expect(wrapper.text()).toContain('agents.mobile.moreDetails')
  })

  it('shows active filter chip labels when route filter is pre-applied', async () => {
    routeApi.query = { status: 'offline' }

    const wrapper = mount(AgentsView)
    await flushPromises()

    expect(wrapper.text()).toContain('agents.columns.status: agents.status.offline')
  })

  it('renders guided onboarding copy in the empty state', async () => {
    const wrapper = mount(AgentsView)
    await flushPromises()

    expect(wrapper.text()).toContain('agents.empty.guideTitle')
    expect(wrapper.text()).toContain('agents.empty.steps.createToken')
    expect(wrapper.text()).toContain('https://backup.example.com')
  })

  it('shows setup guidance instead of defaulting to browser origin when public base url is absent', async () => {
    controlPlaneApi.getPublicMetadata.mockResolvedValue({
      public_base_url: null,
      source: 'default',
      command_generation_ready: false,
    })
    fleetApi.list.mockResolvedValue(buildFleetResponse({
      onboarding: { public_base_url: null, command_generation_ready: false },
    }))

    const wrapper = mount(AgentsView)
    await flushPromises()

    expect(wrapper.text()).toContain('fleet.onboarding.missingDescription')
    expect(wrapper.text()).not.toContain('agents.empty.commandExample')
  })
})
