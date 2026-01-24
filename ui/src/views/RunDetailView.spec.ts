// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string, opts?: { respectShow?: boolean }) =>
    vue.defineComponent({
      name,
      props: [
        'value',
        'show',
        'loading',
        'columns',
        'data',
        'options',
        'preset',
        'title',
        'style',
        'placement',
        'height',
        'size',
        'bordered',
        'type',
        'disabled',
        'trigger',
      ],
      emits: ['update:value', 'update:show', 'select'],
      setup(props, { slots }) {
        return () => {
          if (opts?.respectShow && 'show' in props && !props.show) {
            return vue.h('div', { 'data-stub': name })
          }
          return vue.h('div', { 'data-stub': name }, slots.default?.())
        }
      },
    })

  return {
    NAlert: stub('NAlert'),
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NCode: stub('NCode'),
    NDataTable: stub('NDataTable'),
    NDrawer: stub('NDrawer', { respectShow: true }),
    NDrawerContent: stub('NDrawerContent'),
    NDropdown: stub('NDropdown'),
    NIcon: stub('NIcon'),
    NModal: stub('NModal', { respectShow: true }),
    NSpin: stub('NSpin'),
    NSpace: stub('NSpace'),
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
  getRun: vi.fn(),
  listRunEvents: vi.fn(),
}
vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

const operationsApi = {
  listRunOperations: vi.fn(),
}
vi.mock('@/stores/operations', () => ({
  useOperationsStore: () => operationsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'zh-CN' }),
}))

vi.mock('@/lib/clipboard', () => ({
  copyText: vi.fn().mockResolvedValue(true),
}))

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

class FakeWebSocket {
  url: string
  onopen: (() => void) | null = null
  onmessage: ((evt: MessageEvent) => void) | null = null
  onerror: (() => void) | null = null
  onclose: (() => void) | null = null

  constructor(url: string) {
    this.url = url
  }

  close(): void {
    this.onclose?.()
  }
}

async function flush(): Promise<void> {
  // Let all pending microtasks + Vue updates settle.
  await Promise.resolve()
  await Promise.resolve()
}

import RunDetailView from './RunDetailView.vue'

describe('RunDetailView visual polish', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
    vi.stubGlobal('WebSocket', FakeWebSocket as unknown as typeof WebSocket)
    vi.spyOn(window, 'setInterval').mockReturnValue(1 as unknown as number)

    routeApi.params = { nodeId: 'hub', runId: 'r1' }
    jobsApi.getRun.mockResolvedValue({
      id: 'r1',
      job_id: 'j1',
      status: 'success',
      started_at: 100,
      ended_at: 160,
      progress: null,
      summary: { target: { type: 'local_dir', run_dir: '/tmp/run' }, entries_count: 3, parts: 0 },
      error: null,
    })
    jobsApi.listRunEvents.mockResolvedValue([
      { run_id: 'r1', seq: 1, ts: 100, level: 'info', kind: 'start', message: 'hello', fields: { k: 'v' } },
    ])
    operationsApi.listRunOperations.mockResolvedValue([])
  })

  it('renders a compact operations empty state and uses desktop modal for event details', async () => {
    const wrapper = mount(RunDetailView, {
      global: {
        stubs: {
          RunProgressPanel: { template: '<div data-testid="run-progress-panel" />' },
          RestoreWizardModal: { template: '<div data-testid="restore-wizard-modal" />' },
          VerifyWizardModal: { template: '<div data-testid="verify-wizard-modal" />' },
          OperationModal: { template: '<div data-testid="operation-modal" />' },
        },
      },
    })
    await flush()

    expect(wrapper.find('[data-testid="run-detail"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="run-detail-overview"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('success')
    expect(wrapper.text()).toContain('r1')

    expect(wrapper.text()).toContain('runs.detail.noOperations')
    expect(wrapper.findComponent({ name: 'NDataTable' }).exists()).toBe(false)

    expect(wrapper.findComponent({ name: 'NModal' }).exists()).toBe(true)
    expect(wrapper.findComponent({ name: 'NDrawer' }).exists()).toBe(false)
  })

  it('opens event details in a desktop modal when clicking an event row', async () => {
    const wrapper = mount(RunDetailView, {
      global: {
        stubs: {
          RunProgressPanel: { template: '<div />' },
          RestoreWizardModal: { template: '<div />' },
          VerifyWizardModal: { template: '<div />' },
          OperationModal: { template: '<div />' },
        },
      },
    })
    await flush()

    expect(wrapper.text()).toContain('hello')
    await wrapper.find('[data-testid="run-detail-events-list"] > div').trigger('click')
    await flush()

    // Modal content is only rendered when show=true in our stub.
    expect(wrapper.findComponent({ name: 'NModal' }).text()).toContain('hello')
    expect(wrapper.findComponent({ name: 'NModal' }).text()).toContain('common.close')
  })

  it('uses a bottom drawer for event details on mobile', async () => {
    stubMatchMedia(false)

    const wrapper = mount(RunDetailView, {
      global: {
        stubs: {
          RunProgressPanel: { template: '<div />' },
          RestoreWizardModal: { template: '<div />' },
          VerifyWizardModal: { template: '<div />' },
          OperationModal: { template: '<div />' },
        },
      },
    })
    await flush()

    expect(wrapper.findComponent({ name: 'NDrawer' }).exists()).toBe(true)
    expect(wrapper.findComponent({ name: 'NModal' }).exists()).toBe(false)

    await wrapper.find('[data-testid="run-detail-events-list"] > div').trigger('click')
    await flush()

    expect(wrapper.findComponent({ name: 'NDrawer' }).text()).toContain('hello')
  })
})
