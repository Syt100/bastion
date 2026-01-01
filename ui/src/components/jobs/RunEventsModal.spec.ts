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
    NButton: stub('NButton'),
    NCode: stub('NCode'),
    NModal: stub('NModal'),
    NSpin: stub('NSpin'),
    NSpace: stub('NSpace'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

const jobsApi = {
  listRunEvents: vi.fn(),
}
vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'zh-CN' }),
}))

import RunEventsModal from './RunEventsModal.vue'

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

describe('RunEventsModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    MockWebSocket.instances = []
    vi.stubGlobal('WebSocket', MockWebSocket as unknown as typeof WebSocket)
  })

  it('dedupes websocket events by seq and autoscrolls', async () => {
    jobsApi.listRunEvents.mockResolvedValue([
      { run_id: 'run1', seq: 1, ts: 1, level: 'info', kind: 'start', message: 'start', fields: null },
    ])

    const wrapper = mount(RunEventsModal)

    const scroll = wrapper.get('#run-events-scroll').element as HTMLElement
    Object.defineProperty(scroll, 'scrollHeight', { value: 123, configurable: true })
    scroll.scrollTop = 0

    const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
    await vm.open('run1')

    expect(scroll.scrollTop).toBe(123)

    expect(MockWebSocket.instances).toHaveLength(1)
    const sock = MockWebSocket.instances[0]!
    sock.triggerOpen()
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.text()).toContain('runEvents.ws.connected')

    // Duplicate seq should be ignored.
    sock.triggerMessage(
      JSON.stringify({ run_id: 'run1', seq: 1, ts: 2, level: 'info', kind: 'dup', message: 'dup', fields: null }),
    )
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.findAll('#run-events-scroll .font-mono')).toHaveLength(1)

    // New seq appended.
    sock.triggerMessage(
      JSON.stringify({ run_id: 'run1', seq: 2, ts: 2, level: 'info', kind: 'next', message: 'next', fields: null }),
    )
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.findAll('#run-events-scroll .font-mono')).toHaveLength(2)
    expect(scroll.scrollTop).toBe(123)
  })
})
