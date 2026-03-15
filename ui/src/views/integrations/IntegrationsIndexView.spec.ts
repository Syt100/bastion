// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { createNaiveStub } from '@/test-utils/naiveUiStubs'

const integrationsApi = {
  getSummary: vi.fn(),
}

const routerApi = {
  push: vi.fn(),
}

vi.mock('@/stores/integrations', () => ({
  useIntegrationsStore: () => integrationsApi,
}))

vi.mock('@/navigation/settings', () => ({
  getSettingsOverviewItemsForDomain: () => [
    { key: 'storage', titleKey: 'settings.menu.storage', descriptionKey: 'settings.overview.storageDesc', to: '/integrations/storage', icon: {} },
    { key: 'notifications', titleKey: 'settings.menu.notifications', descriptionKey: 'settings.overview.notificationsDesc', to: '/integrations/notifications', icon: {} },
    { key: 'distribution', titleKey: 'settings.menu.distribution', descriptionKey: 'settings.overview.distributionDesc', to: '/integrations/distribution', icon: {} },
  ],
}))

vi.mock('vue-router', () => ({
  useRouter: () => routerApi,
}))

vi.mock('@/components/AppIcon.vue', () => ({
  default: {
    template: '<span data-stub="AppIcon" />',
  },
}))

vi.mock('@/components/AppEmptyState.vue', () => ({
  default: {
    props: ['title', 'description', 'loading'],
    template: '<div data-stub="AppEmptyState"><div>{{ title }}</div><div>{{ description }}</div><slot name="actions" /></div>',
  },
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('naive-ui', async () => ({
  NCard: createNaiveStub('NCard'),
  NSpin: createNaiveStub('NSpin'),
  NTag: createNaiveStub('NTag'),
  useMessage: () => ({ error: vi.fn() }),
}))

import IntegrationsIndexView from './IntegrationsIndexView.vue'

describe('IntegrationsIndexView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    integrationsApi.getSummary.mockResolvedValue({
      storage: { state: 'ready', summary: { items_total: 3, in_use_total: 2, invalid_total: 1 } },
      notifications: { state: 'degraded', summary: { destinations_total: 2, recent_failures_total: 1, queue_backlog_total: 4 } },
      distribution: { state: 'ready', summary: { coverage_total: 5, drifted_total: 0, failed_total: 0 } },
    })
  })

  it('renders grouped integration entries with status tags', async () => {
    const wrapper = mount(IntegrationsIndexView)
    await flushPromises()

    expect(integrationsApi.getSummary).toHaveBeenCalled()
    expect(wrapper.text()).toContain('settings.menu.storage')
    expect(wrapper.text()).toContain('integrations.states.ready')
    expect(wrapper.text()).toContain('settings.menu.notifications')
    expect(wrapper.text()).toContain('integrations.states.degraded')
  })
})
