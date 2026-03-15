// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { createNaiveButtonStub, createNaiveStub } from '@/test-utils/naiveUiStubs'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  info: vi.fn(),
}

const integrationsApi = {
  getDistribution: vi.fn(),
}

const agentsApi = {
  syncConfigNow: vi.fn(),
}

const routerApi = {
  push: vi.fn(),
}

vi.mock('@/stores/integrations', () => ({
  useIntegrationsStore: () => integrationsApi,
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('vue-router', () => ({
  useRouter: () => routerApi,
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/components/AppEmptyState.vue', () => ({
  default: {
    props: ['title', 'loading'],
    template: '<div data-stub="AppEmptyState">{{ title }}</div>',
  },
}))

vi.mock('naive-ui', async () => ({
  NButton: createNaiveButtonStub(),
  NCard: createNaiveStub('NCard'),
  NTag: createNaiveStub('NTag'),
  useMessage: () => messageApi,
}))

import DistributionView from './DistributionView.vue'

describe('DistributionView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    integrationsApi.getDistribution.mockResolvedValue({
      summary: { coverage_total: 2, drifted_total: 1, failed_total: 1, offline_total: 1 },
      items: [
        {
          agent_id: 'edge-a',
          agent_name: 'Edge A',
          connection_status: 'offline',
          distribution_state: 'failed',
          pending_tasks_total: 2,
          last_attempt_at: 42,
          last_error_kind: 'send_failed',
          last_error: 'Sync failed',
        },
      ],
    })
    agentsApi.syncConfigNow.mockResolvedValue({ outcome: 'sent' })
  })

  it('renders per-agent distribution detail and allows direct follow-up', async () => {
    const wrapper = mount(DistributionView)
    await flushPromises()

    expect(integrationsApi.getDistribution).toHaveBeenCalled()
    expect(wrapper.text()).toContain('integrations.distribution.scopeTitle')
    expect(wrapper.text()).toContain('Edge A')
    expect(wrapper.text()).toContain('integrations.distribution.state.failed')

    const syncButton = wrapper.findAll('button').find((button) => button.text() === 'agents.actions.syncNow')
    expect(syncButton).toBeTruthy()
    await syncButton!.trigger('click')

    expect(agentsApi.syncConfigNow).toHaveBeenCalledWith('edge-a')
  })
})
