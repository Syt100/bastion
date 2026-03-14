// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { reactive, ref } from 'vue'

const routeState = reactive<{
  path: string
  params: Record<string, unknown>
  query: Record<string, unknown>
  matched: Array<{ meta: Record<string, unknown> }>
}>({
  path: '/',
  params: {},
  query: {},
  matched: [{ meta: { primaryNav: 'command-center', scopeMode: 'collection' } }],
})

const routerPush = vi.fn()

const uiStore = {
  preferredNodeId: 'hub',
  preferredScope: 'all',
  locale: 'en-US',
  darkMode: false,
  toggleDarkMode: vi.fn(),
  setLocale: vi.fn(),
  setPreferredNodeId: vi.fn((value: string) => {
    uiStore.preferredNodeId = value
    uiStore.preferredScope = value === 'hub' ? 'hub' : `agent:${value}`
  }),
  setPreferredScope: vi.fn((value: string) => {
    uiStore.preferredScope = value
  }),
}

function setRoute(options: {
  path: string
  params?: Record<string, unknown>
  query?: Record<string, unknown>
  meta?: Record<string, unknown>
}): void {
  routeState.path = options.path
  routeState.params = options.params ?? {}
  routeState.query = options.query ?? {}
  routeState.matched = [{ meta: options.meta ?? {} }]
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
  useAgentsStore: () => ({
    items: [{ id: 'agent1', name: 'Agent One', online: true, revoked: false }],
    refresh: vi.fn().mockResolvedValue(undefined),
  }),
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => uiStore,
}))

vi.mock('@/stores/system', () => ({
  useSystemStore: () => ({ version: null, insecureHttp: false }),
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
      emits: ['update:value', 'update:show', 'select'],
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })

  return {
    NButton: stub('NButton'),
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
    CloudOutline: icon('CloudOutline'),
    ColorPaletteOutline: icon('ColorPaletteOutline'),
    ConstructOutline: icon('ConstructOutline'),
    HomeOutline: icon('HomeOutline'),
    InformationCircleOutline: icon('InformationCircleOutline'),
    MenuOutline: icon('MenuOutline'),
    NotificationsOutline: icon('NotificationsOutline'),
    OptionsOutline: icon('OptionsOutline'),
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

describe('AppShell scope selector and navigation', () => {
  beforeEach(() => {
    uiStore.preferredNodeId = 'hub'
    uiStore.preferredScope = 'all'
    uiStore.setPreferredNodeId.mockClear()
    uiStore.setPreferredScope.mockClear()
    routerPush.mockClear()
    setRoute({
      path: '/',
      meta: { primaryNav: 'command-center', scopeMode: 'collection' },
    })
  })

  it('updates the current collection route scope on collection pages', async () => {
    const wrapper = mount(AppShell, {
      global: { stubs: { 'router-view': { template: '<div />' } } },
    })

    ;(wrapper.vm as unknown as { selectedScope: string }).selectedScope = 'agent:agent1'

    expect(uiStore.setPreferredScope).toHaveBeenCalledWith('agent:agent1')
    expect(routerPush).toHaveBeenCalledWith({
      path: '/',
      query: { scope: 'agent:agent1' },
      hash: undefined,
    })
  })

  it('updates preferred scope without navigating on non-scope-aware pages', async () => {
    setRoute({
      path: '/fleet',
      meta: { primaryNav: 'fleet', scopeMode: 'none' },
    })

    const wrapper = mount(AppShell, {
      global: { stubs: { 'router-view': { template: '<div />' } } },
    })

    ;(wrapper.vm as unknown as { selectedScope: string }).selectedScope = 'hub'

    expect(uiStore.setPreferredScope).toHaveBeenCalledWith('hub')
    expect(routerPush).not.toHaveBeenCalled()
  })

  it('navigates legacy node-scoped workspaces when scope changes there', async () => {
    setRoute({
      path: '/n/hub/jobs',
      params: { nodeId: 'hub' },
      meta: { primaryNav: 'jobs', scopeMode: 'legacy-node' },
    })

    const wrapper = mount(AppShell, {
      global: { stubs: { 'router-view': { template: '<div />' } } },
    })

    ;(wrapper.vm as unknown as { selectedScope: string }).selectedScope = 'agent:agent1'

    expect(uiStore.setPreferredScope).toHaveBeenCalledWith('agent:agent1')
    expect(routerPush).toHaveBeenCalledWith('/n/agent1/jobs')
  })

  it('shows contextual navigation on system pages', () => {
    setRoute({
      path: '/system/runtime',
      meta: { primaryNav: 'system', secondaryNav: 'runtime', scopeMode: 'none' },
    })

    const wrapper = mount(AppShell, {
      global: { stubs: { 'router-view': { template: '<div />' } } },
    })

    expect(wrapper.html()).toContain('nav.context')
  })

  it('uses workbench mode (no outer scroll) for legacy jobs workspace pages on desktop', () => {
    setRoute({
      path: '/n/hub/jobs',
      params: { nodeId: 'hub' },
      meta: { primaryNav: 'jobs', scopeMode: 'legacy-node' },
    })

    const wrapper = mount(AppShell, {
      global: { stubs: { 'router-view': { template: '<div />' } } },
    })

    expect(wrapper.find('main[data-testid="app-shell-main"]').classes()).toContain('overflow-hidden')
  })
})
