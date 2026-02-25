// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const jobsApi = {
  getRun: vi.fn(),
  listRunEvents: vi.fn(),
  cancelRun: vi.fn(),
}

const operationsApi = {
  listRunOperations: vi.fn(),
}

const messageApi = {
  success: vi.fn(),
  error: vi.fn(),
}

vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

vi.mock('@/stores/operations', () => ({
  useOperationsStore: () => operationsApi,
}))

vi.mock('@/lib/errors', () => ({
  formatToastError: (fallback: string) => fallback,
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['disabled', 'show', 'loading', 'type'],
      emits: ['update:show', 'select'],
      setup(props, { slots, attrs }) {
        return () => vue.h('div', { 'data-stub': name, ...attrs }, slots.default?.())
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    props: ['disabled', 'loading', 'type'],
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

  const dropdown = vue.defineComponent({
    name: 'NDropdown',
    props: ['options'],
    emits: ['select'],
    setup(_props, { slots }) {
      return () => vue.h('div', { 'data-stub': 'NDropdown' }, slots.default?.())
    },
  })

  return {
    NButton: button,
    NDropdown: dropdown,
    NIcon: stub('NIcon'),
    NSpin: stub('NSpin'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('@/components/NodeContextTag.vue', () => ({
  default: {
    name: 'NodeContextTag',
    props: ['nodeId'],
    template: '<div data-stub="NodeContextTag" />',
  },
}))

vi.mock('@/components/runs/RunDetailSummaryCard.vue', () => ({
  default: {
    name: 'RunDetailSummaryCard',
    template: '<div data-stub="RunDetailSummaryCard" />',
  },
}))

vi.mock('@/components/runs/RunDetailDetailsTabs.vue', () => ({
  default: {
    name: 'RunDetailDetailsTabs',
    template: '<div data-stub="RunDetailDetailsTabs" />',
  },
}))

vi.mock('@/components/jobs/RestoreWizardModal.vue', () => ({
  default: {
    name: 'RestoreWizardModal',
    template: '<div data-stub="RestoreWizardModal" />',
  },
}))

vi.mock('@/components/jobs/VerifyWizardModal.vue', () => ({
  default: {
    name: 'VerifyWizardModal',
    template: '<div data-stub="VerifyWizardModal" />',
  },
}))

vi.mock('@/components/jobs/OperationModal.vue', () => ({
  default: {
    name: 'OperationModal',
    template: '<div data-stub="OperationModal" />',
  },
}))

import RunDetailPanel from './RunDetailPanel.vue'

async function flush(): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
}

class FakeWebSocket {
  static instances: FakeWebSocket[] = []
  url: string
  onopen: (() => void) | null = null
  onmessage: ((event: MessageEvent) => void) | null = null
  onerror: (() => void) | null = null
  onclose: (() => void) | null = null

  constructor(url: string) {
    this.url = url
    FakeWebSocket.instances.push(this)
  }

  close(): void {
    this.onclose?.()
  }
}

describe('RunDetailPanel cancel UX', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    FakeWebSocket.instances = []

    vi.spyOn(window, 'setInterval').mockReturnValue(1 as unknown as number)
    vi.spyOn(window, 'clearInterval').mockImplementation(() => undefined)
    vi.spyOn(window, 'setTimeout').mockReturnValue(1 as unknown as number)
    vi.spyOn(window, 'clearTimeout').mockImplementation(() => undefined)
    vi.spyOn(window, 'confirm').mockReturnValue(true)

    vi.stubGlobal('WebSocket', FakeWebSocket as unknown as typeof WebSocket)

    jobsApi.getRun.mockResolvedValue({
      id: 'run-1',
      job_id: 'job-1',
      status: 'running',
      started_at: 100,
      ended_at: null,
      cancel_requested_at: null,
      summary: null,
      error: null,
    })
    jobsApi.listRunEvents.mockResolvedValue([])
    jobsApi.cancelRun.mockResolvedValue({
      id: 'run-1',
      job_id: 'job-1',
      status: 'running',
      started_at: 100,
      ended_at: null,
      cancel_requested_at: 120,
      summary: null,
      error: null,
    })
    operationsApi.listRunOperations.mockResolvedValue([])
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('transitions to canceling state after cancel is requested', async () => {
    const wrapper = mount(RunDetailPanel, { props: { nodeId: 'hub', runId: 'run-1' } })
    await flush()

    expect(wrapper.get('[data-testid=\"run-status-tag\"]').text()).toContain('running')

    const cancelButton = wrapper.get('[data-testid=\"run-cancel-button\"]')
    expect(cancelButton.attributes('disabled')).toBeUndefined()
    await cancelButton.trigger('click')
    await flush()

    expect(jobsApi.cancelRun).toHaveBeenCalledWith('run-1')
    expect(wrapper.get('[data-testid=\"run-status-tag\"]').text()).toContain('runs.statuses.canceling')
    expect(wrapper.get('[data-testid=\"run-cancel-button\"]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-testid=\"run-cancel-button\"]').text()).toContain('runs.actions.canceling')
  })

  it('renders terminal canceled status and hides cancel action', async () => {
    jobsApi.getRun.mockResolvedValue({
      id: 'run-canceled',
      job_id: 'job-1',
      status: 'canceled',
      started_at: 100,
      ended_at: 130,
      cancel_requested_at: 105,
      summary: null,
      error: null,
    })

    const wrapper = mount(RunDetailPanel, { props: { nodeId: 'hub', runId: 'run-canceled' } })
    await flush()

    expect(wrapper.get('[data-testid=\"run-status-tag\"]').text()).toContain('canceled')
    expect(wrapper.find('[data-testid=\"run-cancel-button\"]').exists()).toBe(false)
  })
})
