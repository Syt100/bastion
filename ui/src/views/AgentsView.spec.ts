// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  info: vi.fn(),
}

const agentsApi = {
  loading: false,
  items: [],
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

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'show', 'loading', 'columns', 'data', 'options', 'title', 'subtitle'],
      emits: ['update:value', 'update:show', 'update:checked', 'update:checked-row-keys'],
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

  const input = vue.defineComponent({
    name: 'NInput',
    inheritAttrs: false,
    props: ['value', 'type', 'readonly'],
    setup(props) {
      return () => {
        const type = (props as { type?: string }).type
        const value = (props as { value?: string }).value
        const tag = type === 'textarea' ? 'textarea' : 'input'
        return vue.h(tag, { 'data-stub': 'NInput', value: value ?? '' })
      }
    },
  })

  return {
    NButton: button,
    NCard: stub('NCard'),
    NCheckbox: stub('NCheckbox'),
    NDataTable: stub('NDataTable'),
    NForm: stub('NForm'),
    NFormItem: formItem,
    NInput: input,
    NInputNumber: stub('NInputNumber'),
    NModal: modal,
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
    agentsApi.createEnrollmentToken.mockResolvedValue({ token: 'tok1', expires_at: 1234 })
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

    const labels = wrapper.findAll('[data-stub=\"NFormItemLabel\"]').map((n) => n.text())
    expect(labels).toContain('agents.tokenModal.enrollCommand')

    const inputs = wrapper.findAll('textarea[data-stub=\"NInput\"], input[data-stub=\"NInput\"]')
    const values = inputs.map((n) => String((n.element as HTMLInputElement).value))
    const command = values.find((v) => v.includes('bastion agent')) ?? null

    expect(command).not.toBeNull()
    expect(command!).toContain('--hub-url')
    expect(command!).toContain('--enroll-token tok1')
  })
})
