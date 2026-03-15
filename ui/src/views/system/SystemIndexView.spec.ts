// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { createNaiveStub } from '@/test-utils/naiveUiStubs'

const systemApi = {
  refresh: vi.fn(),
  version: '1.2.3',
}

const runtimeApi = {
  get: vi.fn(),
}

const routerApi = {
  push: vi.fn(),
}

vi.mock('@/stores/system', () => ({
  useSystemStore: () => systemApi,
}))

vi.mock('@/stores/hubRuntimeConfig', () => ({
  useHubRuntimeConfigStore: () => runtimeApi,
}))

vi.mock('@/navigation/settings', () => ({
  getSettingsOverviewItemsForDomain: () => [
    { key: 'runtime-config', titleKey: 'settings.menu.runtimeConfig', descriptionKey: 'settings.overview.runtimeConfigDesc', to: '/system/runtime', icon: {} },
    { key: 'maintenance', titleKey: 'settings.menu.maintenance', descriptionKey: 'settings.overview.maintenanceDesc', to: '/system/maintenance', icon: {} },
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

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('naive-ui', async () => ({
  NCard: createNaiveStub('NCard'),
  NTag: createNaiveStub('NTag'),
  useMessage: () => ({ error: vi.fn() }),
}))

import SystemIndexView from './SystemIndexView.vue'

describe('SystemIndexView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    systemApi.refresh.mockResolvedValue(undefined)
    runtimeApi.get.mockResolvedValue({
      effective: { public_base_url: 'https://backup.example.com' },
    })
  })

  it('renders public base url summary and system entry points', async () => {
    const wrapper = mount(SystemIndexView)
    await flushPromises()

    expect(systemApi.refresh).toHaveBeenCalled()
    expect(runtimeApi.get).toHaveBeenCalled()
    expect(wrapper.text()).toContain('https://backup.example.com')
    expect(wrapper.text()).toContain('settings.menu.runtimeConfig')
    expect(wrapper.text()).toContain('settings.menu.maintenance')
  })
})
