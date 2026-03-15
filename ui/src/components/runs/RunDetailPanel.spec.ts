// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'

const runsApi = {
  getWorkspace: vi.fn(),
  listEventConsole: vi.fn(),
  cancelRun: vi.fn(),
}

const operationsApi = {
  listRunOperations: vi.fn(),
}

const messageApi = {
  success: vi.fn(),
  error: vi.fn(),
}

const runEventsStreamApi = {
  status: ref<'disconnected' | 'connecting' | 'live' | 'reconnecting' | 'error'>('disconnected'),
  reconnectAttempts: ref(0),
  reconnectInSeconds: ref<number | null>(null),
  lastSeq: ref(0),
  start: vi.fn(),
  stop: vi.fn(),
  reconnect: vi.fn(),
  setLastSeq: vi.fn(),
}

vi.mock('@/stores/runs', () => ({
  useRunsStore: () => runsApi,
}))

vi.mock('@/stores/operations', () => ({
  useOperationsStore: () => operationsApi,
}))

vi.mock('@/lib/runEventsStream', () => ({
  useRunEventsStream: () => runEventsStreamApi,
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
      props: ['disabled', 'show', 'loading', 'type', 'options', 'component'],
      emits: ['update:show', 'select'],
      setup(_props, { slots, attrs }) {
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
    template: '<div data-stub="NodeContextTag">{{ nodeId }}</div>',
  },
}))

vi.mock('@/components/runs/RunDetailSummaryCard.vue', () => ({
  default: {
    name: 'RunDetailSummaryCard',
    props: ['detail', 'events'],
    template: '<div data-stub="RunDetailSummaryCard">{{ detail?.run?.id }}|{{ events?.length ?? 0 }}</div>',
  },
}))

vi.mock('@/components/runs/RunDetailDetailsTabs.vue', () => ({
  default: {
    name: 'RunDetailDetailsTabs',
    props: ['events', 'window', 'filters', 'consoleLoading', 'wsStatus'],
    template:
      '<div data-stub="RunDetailDetailsTabs">{{ events?.length ?? 0 }}|{{ window?.last_seq ?? "-" }}|{{ filters?.search ?? "" }}|{{ wsStatus }}</div>',
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
  await flushPromises()
  await Promise.resolve()
}

describe('RunDetailPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    runEventsStreamApi.status.value = 'live'
    runEventsStreamApi.lastSeq.value = 0
    vi.spyOn(window, 'setInterval').mockReturnValue(1 as unknown as number)
    vi.spyOn(window, 'clearInterval').mockImplementation(() => undefined)
    vi.spyOn(window, 'confirm').mockReturnValue(true)

    runsApi.getWorkspace.mockResolvedValue({
      run: {
        id: 'run-1',
        job_id: 'job-1',
        job_name: 'Nightly backup',
        scope: 'hub',
        node_id: 'hub',
        status: 'running',
        kind: 'backup',
        started_at: 100,
        ended_at: null,
        cancel_requested_at: null,
        error: null,
      },
      progress: null,
      summary: null,
      diagnostics: {
        state: 'structured',
        failure_title: '—',
        failure_hint: null,
        failure_kind: null,
        failure_stage: null,
        first_error_event_seq: null,
        root_cause_event_seq: null,
      },
      capabilities: {
        can_cancel: true,
        can_restore: false,
        can_verify: false,
      },
      related: {
        operations_total: 0,
        artifacts_total: 0,
      },
    })

    runsApi.listEventConsole.mockResolvedValue({
      filters: {
        q: '',
        levels: [],
        kinds: [],
      },
      window: {
        first_seq: 20,
        last_seq: 21,
        has_older: false,
        has_newer: false,
      },
      locators: {
        first_error_seq: null,
        root_cause_seq: null,
      },
      items: [
        {
          run_id: 'run-1',
          seq: 20,
          ts: 200,
          level: 'info',
          kind: 'started',
          message: 'Run started',
          fields: null,
        },
        {
          run_id: 'run-1',
          seq: 21,
          ts: 210,
          level: 'info',
          kind: 'heartbeat',
          message: 'Upload in progress',
          fields: null,
        },
      ],
    })
    runsApi.cancelRun.mockResolvedValue(undefined)
    operationsApi.listRunOperations.mockResolvedValue([])
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('loads the dedicated run workspace and follows the latest event window', async () => {
    const wrapper = mount(RunDetailPanel, { props: { nodeId: 'hub', runId: 'run-1' } })
    await flush()

    expect(runsApi.getWorkspace).toHaveBeenCalledWith('run-1')
    expect(runsApi.listEventConsole).toHaveBeenCalledWith('run-1', {
      q: undefined,
      levels: undefined,
      kinds: undefined,
      limit: 100,
      beforeSeq: undefined,
      afterSeq: undefined,
      anchor: 'tail',
    })
    expect(runEventsStreamApi.setLastSeq).toHaveBeenCalledWith(21)
    expect(runEventsStreamApi.start).toHaveBeenCalledWith('run-1', 21)
    expect(wrapper.get('[data-stub="RunDetailSummaryCard"]').text()).toContain('run-1|2')
    expect(wrapper.get('[data-stub="RunDetailDetailsTabs"]').text()).toContain('2|21||live')
  })

  it('transitions to canceling state after cancel is requested from the run workspace', async () => {
    runsApi.cancelRun.mockImplementation(async () => {
      runsApi.getWorkspace.mockResolvedValue({
        run: {
          id: 'run-1',
          job_id: 'job-1',
          job_name: 'Nightly backup',
          scope: 'hub',
          node_id: 'hub',
          status: 'running',
          kind: 'backup',
          started_at: 100,
          ended_at: null,
          cancel_requested_at: 120,
          error: null,
        },
        progress: null,
        summary: null,
        diagnostics: {
          state: 'structured',
          failure_title: '—',
          failure_hint: null,
          failure_kind: null,
          failure_stage: null,
          first_error_event_seq: null,
          root_cause_event_seq: null,
        },
        capabilities: {
          can_cancel: true,
          can_restore: false,
          can_verify: false,
        },
        related: {
          operations_total: 0,
          artifacts_total: 0,
        },
      })
    })

    const wrapper = mount(RunDetailPanel, { props: { nodeId: 'hub', runId: 'run-1' } })
    await flush()

    expect(wrapper.get('[data-testid="run-status-tag"]').text()).toContain('running')

    const cancelButton = wrapper.get('[data-testid="run-cancel-button"]')
    expect(cancelButton.attributes('disabled')).toBeUndefined()
    await cancelButton.trigger('click')
    await flush()

    expect(runsApi.cancelRun).toHaveBeenCalledWith('run-1')
    expect(wrapper.get('[data-testid="run-status-tag"]').text()).toContain('runs.statuses.canceling')
    expect(wrapper.get('[data-testid="run-cancel-button"]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-testid="run-cancel-button"]').text()).toContain('runs.actions.canceling')
  })
})
