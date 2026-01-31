// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { reactive, ref } from 'vue'

let routeState = reactive<{ path: string; params: Record<string, unknown> }>({ path: '/', params: {} })
const routerPush = vi.fn()

const uiStore = {
  preferredNodeId: 'hub',
  locale: 'en-US',
  darkMode: false,
  toggleDarkMode: vi.fn(),
  setLocale: vi.fn(),
  setPreferredNodeId: vi.fn((value: string) => {
    uiStore.preferredNodeId = value
  }),
}

vi.mock('vue-router', () => ({
  useRoute: () => routeState,
  useRouter: () => ({ push: routerPush }),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => ({ logout: vi.fn().mockResolvedValue(undefined), status: 'authenticated', isAuthenticated: true }),
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => ({ items: [], refresh: vi.fn().mockResolvedValue(undefined) }),
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => uiStore,
}))

vi.mock('@/stores/system', () => ({
  useSystemStore: () => ({ version: null, insecureHttp: false }),
}))

vi.mock('@/navigation/settings', () => ({
  getSettingsMenuRouteKeys: () => ['/settings'],
  getSettingsSidebarItems: () => [],
}))

vi.mock('@/i18n/language', () => ({
  getLocaleDropdownOptions: () => [{ label: 'English', key: 'en-US' }],
}))

vi.mock('@/lib/media', () => ({
  useMediaQuery: () => ref(true),
}))

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'options', 'show'],
      emits: ['update:value', 'update:expanded-keys', 'update:show', 'select'],
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })

  return {
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NDrawer: stub('NDrawer'),
    NDrawerContent: stub('NDrawerContent'),
    NDropdown: stub('NDropdown'),
    NIcon: stub('NIcon'),
    NLayout: stub('NLayout'),
    NLayoutHeader: stub('NLayoutHeader'),
    NLayoutSider: stub('NLayoutSider'),
    NMenu: stub('NMenu'),
    NSelect: stub('NSelect'),
    NTag: stub('NTag'),
    useMessage: () => ({ error: vi.fn(), success: vi.fn() }),
  }
})

vi.mock('@vicons/ionicons5', async () => {
  const vue = await import('vue')
  const icon = (name: string) => vue.defineComponent({ name, setup: () => () => vue.h('i') })
  return {
    ArchiveOutline: icon('ArchiveOutline'),
    EllipsisHorizontal: icon('EllipsisHorizontal'),
    HomeOutline: icon('HomeOutline'),
    MenuOutline: icon('MenuOutline'),
    PeopleOutline: icon('PeopleOutline'),
    SettingsOutline: icon('SettingsOutline'),
  }
})

vi.mock('@/components/InsecureHttpBanner.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'InsecureHttpBanner', setup: () => () => vue.h('div') }) }
})

vi.mock('@/components/AppLogo.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'AppLogo', setup: () => () => vue.h('div') }) }
})

import AppShell from './AppShell.vue'

describe('AppShell node selector', () => {
  beforeEach(() => {
    uiStore.preferredNodeId = 'hub'
    uiStore.setPreferredNodeId.mockClear()
    routerPush.mockClear()
    routeState.path = '/'
    routeState.params = {}
  })

  it('updates preferred node without navigating on global pages', async () => {
    routeState.path = '/agents'
    routeState.params = {}

    const wrapper = mount(AppShell, {
      global: {
        stubs: { 'router-view': { template: '<div />' } },
      },
    })
    ;(wrapper.vm as unknown as { selectedNodeId: string }).selectedNodeId = 'agent1'

    expect(uiStore.setPreferredNodeId).toHaveBeenCalledWith('agent1')
    expect(routerPush).not.toHaveBeenCalled()
  })

  it('navigates when selecting a node on node-scoped pages', async () => {
    routeState.path = '/n/hub/jobs'
    routeState.params = { nodeId: 'hub' }

    const wrapper = mount(AppShell, {
      global: {
        stubs: { 'router-view': { template: '<div />' } },
      },
    })
    ;(wrapper.vm as unknown as { selectedNodeId: string }).selectedNodeId = 'agent1'

    expect(uiStore.setPreferredNodeId).toHaveBeenCalledWith('agent1')
    expect(routerPush).toHaveBeenCalledWith('/n/agent1/jobs')
  })
})
