// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

const i18nDict: Record<string, string> = {}

const tMock = (key: string, params?: Record<string, unknown>): string => {
  const template = i18nDict[key]
  if (!template) return key
  if (!params) return template
  return template.replace(/\{(\w+)\}/g, (_, token: string) => {
    const value = params[token]
    return value == null ? '' : String(value)
  })
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'show', 'loading', 'disabled'],
      emits: ['update:value', 'update:show'],
      setup(props, { slots }) {
        return () =>
          vue.h(
            'div',
            {
              'data-stub': name,
              'data-disabled': typeof (props as { disabled?: boolean }).disabled === 'boolean'
                ? String(Boolean((props as { disabled?: boolean }).disabled))
                : undefined,
            },
            slots.default?.(),
          )
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
  useI18n: () => ({ t: tMock }),
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
    for (const key of Object.keys(i18nDict)) delete i18nDict[key]
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

      const latestButton = () => wrapper.find('[data-testid="run-events-latest"] [data-stub="NButton"]')

      // Move away from bottom -> follow auto-unfollows.
      nowMs = 1000
      el.scrollTop = 700
      await list.trigger('scroll')
      await Promise.resolve()
      await Promise.resolve()
      expect(latestButton().attributes('data-disabled')).toBe('false')

      // Return to bottom -> follow auto-resumes.
      el.scrollTop = 900
      await list.trigger('scroll')
      await Promise.resolve()
      await Promise.resolve()
      expect(latestButton().attributes('data-disabled')).toBe('true')
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
    expect(wrapper.find('[data-testid="run-events-latest"] [data-stub="NButton"]').attributes('data-disabled')).toBe(
      'false',
    )

    // Reaching bottom should not re-enable follow.
    el.scrollTop = 900
    await list.trigger('scroll')
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.find('[data-testid="run-events-latest"] [data-stub="NButton"]').attributes('data-disabled')).toBe(
      'false',
    )

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

  it('renders failure diagnostic chips from structured fields', async () => {
    jobsApi.listRunEvents.mockResolvedValue([
      {
        run_id: 'run1',
        seq: 1,
        ts: 1,
        level: 'error',
        kind: 'failed',
        message: 'failed: webdav put failed',
        fields: {
          error_kind: 'payload_too_large',
          http_status: 413,
          hint: 'reduce part size and check gateway upload limits',
          part_size_bytes: 16 * 1024 * 1024,
        },
      },
    ])

    const wrapper = mount(RunEventsModal)
    const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
    await vm.open('run1')

    const text = wrapper.text()
    expect(text).toContain('payload_too_large')
    expect(text).toContain('HTTP 413')
    expect(text).toContain('16MB')

    wrapper.unmount()
  })

  it('renders localized hint label in event details', async () => {
    jobsApi.listRunEvents.mockResolvedValue([
      {
        run_id: 'run1',
        seq: 1,
        ts: 1,
        level: 'error',
        kind: 'failed',
        message: 'failed: storage capacity exhausted',
        fields: {
          error_kind: 'storage_full',
          hint: 'free space or adjust retention before retrying',
        },
      },
    ])

    const wrapper = mount(RunEventsModal)
    const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
    await vm.open('run1')

    await wrapper.find('[data-testid="run-event-row"]').trigger('click')
    await Promise.resolve()
    await Promise.resolve()

    expect(wrapper.text()).toContain('runEvents.details.hintLabel')
    expect(wrapper.text()).toContain('free space or adjust retention before retrying')

    wrapper.unmount()
  })

  it('renders envelope message/hint localization with legacy fallback', async () => {
    i18nDict['diagnostics.message.notification.network'] = '通知发送网络异常'
    i18nDict['diagnostics.hint.notification.network'] = '请检查通知通道网络连通性'

    jobsApi.listRunEvents.mockResolvedValue([
      {
        run_id: 'run1',
        seq: 1,
        ts: 1,
        level: 'warn',
        kind: 'notify_failed',
        message: 'notify_failed',
        fields: {
          hint: 'legacy hint fallback',
          error_envelope: {
            schema_version: '1.0',
            code: 'notification.send.network',
            kind: 'network',
            retriable: { value: true, reason: 'network' },
            hint: { key: 'diagnostics.hint.notification.network', params: {} },
            message: { key: 'diagnostics.message.notification.network', params: {} },
            transport: { protocol: 'http' },
          },
        },
      },
    ])

    const wrapper = mount(RunEventsModal)
    const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
    await vm.open('run1')

    expect(wrapper.text()).toContain('通知发送网络异常')

    await wrapper.find('[data-testid="run-event-row"]').trigger('click')
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.text()).toContain('请检查通知通道网络连通性')

    wrapper.unmount()
  })

  it('falls back to generic hint when envelope key is missing and no legacy hint exists', async () => {
    i18nDict['runEvents.details.genericHint'] = '请查看诊断上下文和日志'

    jobsApi.listRunEvents.mockResolvedValue([
      {
        run_id: 'run1',
        seq: 1,
        ts: 1,
        level: 'error',
        kind: 'failed',
        message: 'failed',
        fields: {
          error_envelope: {
            schema_version: '1.0',
            code: 'run.failed.unknown',
            kind: 'unknown',
            retriable: { value: false },
            hint: { key: 'diagnostics.hint.missing', params: {} },
            message: { key: 'diagnostics.message.missing', params: {} },
            transport: { protocol: 'unknown' },
          },
        },
      },
    ])

    const wrapper = mount(RunEventsModal)
    const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
    await vm.open('run1')

    await wrapper.find('[data-testid="run-event-row"]').trigger('click')
    await Promise.resolve()
    await Promise.resolve()
    expect(wrapper.text()).toContain('请查看诊断上下文和日志')

    wrapper.unmount()
  })

  it('shows protocol, async operation and partial failure details from envelope context', async () => {
    jobsApi.listRunEvents.mockResolvedValue([
      {
        run_id: 'run1',
        seq: 1,
        ts: 1,
        level: 'error',
        kind: 'failed',
        message: 'failed',
        fields: {
          error_envelope: {
            schema_version: '1.0',
            code: 'target.sftp.permission_denied',
            kind: 'auth',
            retriable: { value: false },
            hint: { key: 'diagnostics.hint.artifact_delete.auth', params: {} },
            message: { key: 'diagnostics.message.artifact_delete.auth', params: {} },
            transport: {
              protocol: 'sftp',
              provider_code: 'SSH_FX_PERMISSION_DENIED',
            },
            context: {
              operation: {
                operation_id: 'op-123',
                status: 'failed',
                poll_after_sec: 20,
              },
              partial_failures: [
                {
                  path: '/docs/a.txt',
                  code: 'target.permission_denied',
                  kind: 'auth',
                  transport: { protocol: 'sftp' },
                },
              ],
            },
          },
        },
      },
    ])

    const wrapper = mount(RunEventsModal)
    const vm = wrapper.vm as unknown as { open: (runId: string) => Promise<void> }
    await vm.open('run1')

    await wrapper.find('[data-testid="run-event-row"]').trigger('click')
    await Promise.resolve()
    await Promise.resolve()

    const text = wrapper.text()
    expect(text).toContain('SSH_FX_PERMISSION_DENIED')
    expect(text).toContain('op-123')
    expect(text).toContain('/docs/a.txt')

    wrapper.unmount()
  })
})
