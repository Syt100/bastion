// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const operationsApi = {
  getOperation: vi.fn(),
  listEvents: vi.fn(),
  cancelOperation: vi.fn(),
}

const messageApi = {
  success: vi.fn(),
  error: vi.fn(),
}

vi.mock('@/stores/operations', () => ({
  useOperationsStore: () => operationsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string, opts?: { respectShow?: boolean }) =>
    vue.defineComponent({
      name,
      props: ['show', 'title', 'type', 'disabled'],
      emits: ['update:show'],
      setup(props, { slots, attrs }) {
        return () => {
          if (opts?.respectShow && 'show' in props && !props.show) {
            return vue.h('div', { 'data-stub': name })
          }
          return vue.h('div', { 'data-stub': name, ...attrs }, slots.default?.())
        }
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    props: ['disabled', 'loading'],
    setup(props, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            disabled: !!props.disabled,
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
            ...attrs,
          },
          slots.default?.(),
        )
    },
  })

  return {
    NAlert: stub('NAlert'),
    NButton: button,
    NCode: stub('NCode'),
    NModal: stub('NModal', { respectShow: true }),
    NSpin: stub('NSpin'),
    NSpace: stub('NSpace'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

import OperationModal from './OperationModal.vue'

async function flush(): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
}

describe('OperationModal', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  beforeEach(() => {
    vi.clearAllMocks()
    vi.spyOn(window, 'setInterval').mockReturnValue(1 as unknown as number)
    vi.spyOn(window, 'clearInterval').mockImplementation(() => undefined)
    vi.spyOn(window, 'confirm').mockReturnValue(true)

    operationsApi.getOperation.mockResolvedValue({
      id: 'op1',
      kind: 'restore',
      status: 'success',
      created_at: 1,
      started_at: 100,
      ended_at: 120,
      progress: {
        stage: 'restore',
        ts: 110,
        done: { files: 0, dirs: 0, bytes: 100 },
        // Intentionally missing rate_bps to force average-speed fallback.
      },
      summary: null,
      error: null,
    })

    operationsApi.listEvents.mockResolvedValue([
      { op_id: 'op1', seq: 1, ts: 105, level: 'info', kind: 'progress_snapshot', message: 'restore', fields: null },
      { op_id: 'op1', seq: 2, ts: 115, level: 'info', kind: 'complete', message: 'complete', fields: null },
    ])
    operationsApi.cancelOperation.mockResolvedValue({
      id: 'op1',
      kind: 'restore',
      status: 'running',
      created_at: 1,
      started_at: 100,
      ended_at: null,
      cancel_requested_at: 130,
      summary: null,
      error: null,
    })
  })

  it('shows a computed final speed after completion when live rate is missing', async () => {
    const wrapper = mount(OperationModal)

    // Exposed method via defineExpose.
    await (wrapper.vm as unknown as { open: (id: string) => Promise<void> }).open('op1')
    await flush()

    // 100 B / (115 - 105) = 10 B/s.
    expect(wrapper.text()).toContain('10 B/s')
  })

  it('shows canceling state and disables cancel button after cancel is requested', async () => {
    operationsApi.getOperation.mockResolvedValue({
      id: 'op-running',
      kind: 'verify',
      status: 'running',
      created_at: 1,
      started_at: 100,
      ended_at: null,
      cancel_requested_at: null,
      summary: null,
      error: null,
    })
    operationsApi.listEvents.mockResolvedValue([])

    const wrapper = mount(OperationModal)
    await (wrapper.vm as unknown as { open: (id: string) => Promise<void> }).open('op-running')
    await flush()

    const cancel = wrapper.get('[data-testid=\"operation-cancel-button\"]')
    expect(cancel.attributes('disabled')).toBeUndefined()
    await cancel.trigger('click')
    await flush()

    expect(operationsApi.cancelOperation).toHaveBeenCalledWith('op-running')
    expect(wrapper.text()).toContain('operations.statuses.canceling')
    expect(wrapper.get('[data-testid=\"operation-cancel-button\"]').attributes('disabled')).toBeDefined()
  })

  it('renders terminal canceled status', async () => {
    operationsApi.getOperation.mockResolvedValue({
      id: 'op-canceled',
      kind: 'restore',
      status: 'canceled',
      created_at: 1,
      started_at: 100,
      ended_at: 120,
      cancel_requested_at: 105,
      summary: null,
      error: null,
    })
    operationsApi.listEvents.mockResolvedValue([])

    const wrapper = mount(OperationModal)
    await (wrapper.vm as unknown as { open: (id: string) => Promise<void> }).open('op-canceled')
    await flush()

    expect(wrapper.text()).toContain('canceled')
    expect(wrapper.find('[data-testid=\"operation-cancel-button\"]').exists()).toBe(false)
  })
})
