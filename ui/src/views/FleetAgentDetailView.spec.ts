// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'

import { createNaiveButtonStub, createNaiveInputStub, createNaiveStub } from '@/test-utils/naiveUiStubs'

const fleetApi = {
  get: vi.fn(),
}

const agentsApi = {
  syncConfigNow: vi.fn(),
  rotateAgentKey: vi.fn(),
  revokeAgent: vi.fn(),
}

const uiStore = { locale: 'en-US' }

const routerApi = {
  push: vi.fn(),
}

const routeApi = {
  params: { agentId: 'edge-a' },
}

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  info: vi.fn(),
}

vi.mock('@/stores/fleet', () => ({
  useFleetStore: () => fleetApi,
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => uiStore,
}))

vi.mock('@/lib/clipboard', () => ({
  copyText: vi.fn().mockResolvedValue(true),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string, params?: Record<string, unknown>) => (params ? `${key}:${JSON.stringify(params)}` : key) }),
}))

vi.mock('vue-router', () => ({
  useRouter: () => routerApi,
  useRoute: () => routeApi,
}))

vi.mock('@/components/AppModalShell.vue', () => ({
  default: {
    template: '<div><slot /><slot name="footer" /></div>',
  },
}))

vi.mock('@/components/AppEmptyState.vue', () => ({
  default: {
    props: ['title', 'description', 'loading'],
    template: '<div data-stub="AppEmptyState"><div>{{ title }}</div><div>{{ description }}</div><slot name="actions" /></div>',
  },
}))

vi.mock('naive-ui', async () => {
  const stub = (name: string) => createNaiveStub(name)
  const nCard = defineComponent({
    name: 'NCard',
    props: ['title'],
    setup(props, { slots, attrs }) {
      return () =>
        h('div', { 'data-stub': 'NCard', ...attrs }, [props.title ? h('div', props.title as string) : null, slots.default?.()])
    },
  })
  return {
    NButton: createNaiveButtonStub(),
    NCard: nCard,
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NInput: createNaiveInputStub(),
    NSpace: stub('NSpace'),
    NSpin: stub('NSpin'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

import FleetAgentDetailView from './FleetAgentDetailView.vue'

describe('FleetAgentDetailView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    fleetApi.get.mockResolvedValue({
      agent: {
        id: 'edge-a',
        name: 'DB Node A',
        status: 'online',
        created_at: 10,
        last_seen_at: 20,
        labels: ['db'],
      },
      sync: {
        desired_snapshot_id: 'cfg-1',
        desired_snapshot_at: 10,
        applied_snapshot_id: 'cfg-1',
        applied_snapshot_at: 12,
        state: 'synced',
        last_error_kind: null,
        last_error: null,
        last_attempt_at: 12,
      },
      recent_activity: [
        {
          run_id: 'run-1',
          job_id: 'job-1',
          job_name: 'Nightly Backup',
          status: 'success',
          started_at: 100,
          ended_at: 120,
        },
      ],
      related_jobs: [
        {
          id: 'job-1',
          name: 'Nightly Backup',
          schedule: '0 * * * *',
          updated_at: 200,
        },
      ],
      capabilities: {
        can_rotate_key: true,
        can_revoke: true,
        can_sync_now: true,
        can_manage_storage: true,
      },
    })
    agentsApi.syncConfigNow.mockResolvedValue({ outcome: 'sent' })
    agentsApi.rotateAgentKey.mockResolvedValue({ agent_id: 'edge-a', agent_key: 'new-key' })
    agentsApi.revokeAgent.mockResolvedValue(undefined)
  })

  it('renders detail sections from fleet response', async () => {
    const wrapper = mount(FleetAgentDetailView)
    await flushPromises()

    expect(fleetApi.get).toHaveBeenCalledWith('edge-a')
    expect(wrapper.text()).toContain('DB Node A')
    expect(wrapper.text()).toContain('fleet.detail.syncTitle')
    expect(wrapper.text()).toContain('fleet.detail.relatedJobsTitle')
    expect(wrapper.text()).toContain('Nightly Backup')
  })

  it('routes to jobs workspace from the header action', async () => {
    const wrapper = mount(FleetAgentDetailView)
    await flushPromises()

    const jobsButton = wrapper.findAll('button').find((b) => b.text() === 'agents.actions.jobs')
    expect(jobsButton).toBeTruthy()
    await jobsButton!.trigger('click')

    expect(routerApi.push).toHaveBeenCalledWith({
      path: '/jobs',
      query: { scope: 'agent:edge-a' },
    })
  })
})
