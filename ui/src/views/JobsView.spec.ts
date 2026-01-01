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
      props: ['value', 'show', 'loading'],
      emits: ['update:value', 'update:show'],
      setup(_, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })

  return {
    NAlert: stub('NAlert'),
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NCode: stub('NCode'),
    NDataTable: stub('NDataTable'),
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NInput: stub('NInput'),
    NInputNumber: stub('NInputNumber'),
    NModal: stub('NModal'),
    NPopconfirm: stub('NPopconfirm'),
    NSelect: stub('NSelect'),
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

const jobsApi = {
  items: [],
  loading: false,
  refresh: vi.fn().mockResolvedValue(undefined),
  listRunEvents: vi.fn(),
  listRuns: vi.fn(),
  runNow: vi.fn(),
  getJob: vi.fn(),
  createJob: vi.fn(),
  updateJob: vi.fn(),
  deleteJob: vi.fn(),
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

const secretsApi = {
  webdav: [],
  refreshWebdav: vi.fn().mockResolvedValue(undefined),
}
vi.mock('@/stores/secrets', () => ({
  useSecretsStore: () => secretsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'zh-CN' }),
}))

import JobsView from './JobsView.vue'

class MockWebSocket {
  static instances: MockWebSocket[] = []
  url: string
  onopen: (() => void) | null = null
  onmessage: ((evt: { data: unknown }) => void) | null = null
  onerror: (() => void) | null = null
  onclose: (() => void) | null = null

  constructor(url: string) {
    this.url = url
    MockWebSocket.instances.push(this)
  }

  close(): void {
    this.onclose?.()
  }

  triggerOpen(): void {
    this.onopen?.()
  }

  triggerMessage(value: unknown): void {
    this.onmessage?.({ data: value })
  }
}

describe('JobsView run events', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    MockWebSocket.instances = []
    vi.stubGlobal('WebSocket', MockWebSocket as unknown as typeof WebSocket)
  })

  it('dedupes websocket events by seq and autoscrolls', async () => {
    const scroll = document.createElement('div')
    scroll.id = 'run-events-scroll'
    Object.defineProperty(scroll, 'scrollHeight', { value: 123, configurable: true })
    scroll.scrollTop = 0
    document.body.appendChild(scroll)

    jobsApi.listRunEvents.mockResolvedValue([
      { run_id: 'run1', seq: 1, ts: 1, level: 'info', kind: 'start', message: 'start', fields: null },
    ])

    const wrapper = mount(JobsView)
    await Promise.resolve()

    const vm = wrapper.vm as unknown as {
      openRunEvents: (runId: string) => Promise<void>
      runEvents: unknown[]
      runEventsWsStatus: string
    }

    await vm.openRunEvents('run1')

    expect(vm.runEvents).toHaveLength(1)
    expect(scroll.scrollTop).toBe(123)

    expect(MockWebSocket.instances).toHaveLength(1)
    const sock = MockWebSocket.instances[0]!
    sock.triggerOpen()
    expect(vm.runEventsWsStatus).toBe('connected')

    // Duplicate seq should be ignored.
    sock.triggerMessage(JSON.stringify({ run_id: 'run1', seq: 1, ts: 2, level: 'info', kind: 'dup', message: 'dup', fields: null }))
    await Promise.resolve()
    expect(vm.runEvents).toHaveLength(1)

    // New seq appended.
    sock.triggerMessage(JSON.stringify({ run_id: 'run1', seq: 2, ts: 2, level: 'info', kind: 'next', message: 'next', fields: null }))
    await Promise.resolve()
    await Promise.resolve()
    expect(vm.runEvents).toHaveLength(2)
    expect(scroll.scrollTop).toBe(123)

    document.body.removeChild(scroll)
  })
})
