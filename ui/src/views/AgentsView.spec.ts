// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { createNaiveButtonStub, createNaiveInputStub, createNaiveStub } from '@/test-utils/naiveUiStubs'
import type { AgentListItem } from '@/stores/agents'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  info: vi.fn(),
}

const agentsApi = {
  loading: false,
  items: [] as AgentListItem[],
  refresh: vi.fn().mockResolvedValue(undefined),
  listLabelIndex: vi.fn().mockResolvedValue([]),
  createEnrollmentToken: vi.fn(),
}

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/bulkOperations', () => ({
  useBulkOperationsStore: () => ({ create: vi.fn() }),
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
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
vi.mock('vue-router', () => ({
  useRouter: () => routerApi,
  useRoute: () => routeApi,
}))

vi.mock('@/lib/media', async () => {
  const vue = await import('vue')
  return { useMediaQuery: () => vue.ref(true) }
})

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const modal = vue.defineComponent({
    name: 'NModal',
    props: ['show'],
    emits: ['update:show'],
    setup(props, { slots }) {
      return () => ((props as { show?: boolean }).show ? vue.h('div', { 'data-stub': 'NModal' }, slots.default?.()) : null)
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
    NSpace: stub('NSpace'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

import AgentsView from './AgentsView.vue'

describe('AgentsView enrollment token modal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    routeApi.query = {}
    agentsApi.items = []
    agentsApi.createEnrollmentToken.mockResolvedValue({ token: 'tok1', expires_at: 1234 })
  })

  it('shows pagination when the filtered list is large', async () => {
    agentsApi.items = Array.from({ length: 25 }, (_, idx) => ({
      id: `agent-${idx}`,
      name: `Agent ${idx}`,
      revoked: false,
      last_seen_at: null,
      online: idx % 2 === 0,
      labels: [],
      desired_config_snapshot_id: null,
      applied_config_snapshot_id: null,
      config_sync_status: 'offline',
      last_config_sync_attempt_at: null,
      last_config_sync_error_kind: null,
      last_config_sync_error: null,
      last_config_sync_error_at: null,
    }))

    const wrapper = mount(AgentsView)
    await flushPromises()

    expect(wrapper.find('[data-stub="NPagination"]').exists()).toBe(true)
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
  })
})
