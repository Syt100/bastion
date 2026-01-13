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
    NDrawer: stub('NDrawer'),
    NDrawerContent: stub('NDrawerContent'),
    NModal: stub('NModal'),
    NSpin: stub('NSpin'),
    NSpace: stub('NSpace'),
    NSwitch: stub('NSwitch'),
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

  it('auto-resumes follow when returning to bottom after auto-unfollow', async () => {
    let nowMs = 0
    const nowSpy = vi.spyOn(Date, 'now').mockImplementation(() => nowMs)

    jobsApi.listRunEvents.mockResolvedValue([
      { run_id: 'run1', seq: 1, ts: 1, level: 'info', kind: 'start', message: 'start', fields: null },
    ])

    const wrapper = mount(RunEventsModal)
    try {
      const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
      await vm.open('run1')

      const list = wrapper.find('[data-testid="run-events-list"]')
      const el = list.element as HTMLElement
      Object.defineProperty(el, 'scrollHeight', { value: 1000, configurable: true })
      Object.defineProperty(el, 'clientHeight', { value: 100, configurable: true })

      // Move away from bottom -> follow auto-unfollows.
      nowMs = 1000
      el.scrollTop = 700
      await list.trigger('scroll')
      await Promise.resolve()
      await Promise.resolve()
      expect(wrapper.find('[data-testid="run-events-latest"]').classes()).not.toContain('invisible')

      // Return to bottom -> follow auto-resumes.
      el.scrollTop = 900
      await list.trigger('scroll')
      await Promise.resolve()
      await Promise.resolve()
      expect(wrapper.find('[data-testid="run-events-latest"]').classes()).toContain('invisible')
    } finally {
      wrapper.unmount()
      nowSpy.mockRestore()
    }
  })

  it('does not auto-resume follow when it was manually disabled', async () => {
    jobsApi.listRunEvents.mockResolvedValue([
      { run_id: 'run1', seq: 1, ts: 1, level: 'info', kind: 'start', message: 'start', fields: null },
    ])

    const wrapper = mount(RunEventsModal)
    const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
    await vm.open('run1')

    const list = wrapper.find('[data-testid="run-events-list"]')
    const el = list.element as HTMLElement
    Object.defineProperty(el, 'scrollHeight', { value: 1000, configurable: true })
    Object.defineProperty(el, 'clientHeight', { value: 100, configurable: true })

    const followSwitch = wrapper.findComponent({ name: 'NSwitch' })
    followSwitch.vm.$emit('update:value', false)
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.find('[data-testid="run-events-latest"]').classes()).not.toContain('invisible')

    // Reaching bottom should not re-enable follow.
    el.scrollTop = 900
    await list.trigger('scroll')
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.find('[data-testid="run-events-latest"]').classes()).not.toContain('invisible')

    wrapper.unmount()
  })

  it('uses after_seq and dedupes websocket events by seq', async () => {
    jobsApi.listRunEvents.mockResolvedValue([
      { run_id: 'run1', seq: 1, ts: 1, level: 'info', kind: 'start', message: 'start', fields: null },
    ])

    const wrapper = mount(RunEventsModal)

    const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
    await vm.open('run1')

    expect(MockWebSocket.instances).toHaveLength(1)
    const sock = MockWebSocket.instances[0]!
    expect(sock.url).toContain('after_seq=1')
    sock.triggerOpen()
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.text()).toContain('runEvents.ws.live')

    // Duplicate seq should be ignored.
    sock.triggerMessage(
      JSON.stringify({ run_id: 'run1', seq: 1, ts: 2, level: 'info', kind: 'dup', message: 'dup', fields: null }),
    )
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.findAll('[data-testid="run-event-row"]')).toHaveLength(1)

    // New seq appended.
    sock.triggerMessage(
      JSON.stringify({ run_id: 'run1', seq: 2, ts: 2, level: 'info', kind: 'next', message: 'next', fields: null }),
    )
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.findAll('[data-testid="run-event-row"]')).toHaveLength(2)

    wrapper.unmount()
  })
})
