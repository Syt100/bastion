// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value'],
      emits: ['update:value'],
      setup(_, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })

  return {
    NCard: stub('NCard'),
    NTabs: stub('NTabs'),
    NTabPane: stub('NTabPane'),
  }
})

const routeApi = {
  path: '/integrations/notifications/channels',
}
const routerApi = {
  push: vi.fn(),
}
vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

import NotificationsShellView from './NotificationsShellView.vue'

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

describe('NotificationsShellView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    routeApi.path = '/integrations/notifications/channels'
    stubMatchMedia(true)
  })

  it('renders desktop tabs in embedded style on notification subpages', () => {
    const wrapper = mount(NotificationsShellView, {
      global: {
        stubs: {
          'router-view': true,
        },
      },
    })

    expect(wrapper.find('.app-tabs-embedded').exists()).toBe(true)
  })

  it('hides desktop tabs on notifications index route', () => {
    routeApi.path = '/integrations/notifications'
    const wrapper = mount(NotificationsShellView, {
      global: {
        stubs: {
          'router-view': true,
        },
      },
    })

    expect(wrapper.find('.app-tabs-embedded').exists()).toBe(false)
  })

  it('navigates when switching tabs', async () => {
    const wrapper = mount(NotificationsShellView, {
      global: {
        stubs: {
          'router-view': true,
        },
      },
    })

    const tabs = wrapper.findComponent({ name: 'NTabs' })
    tabs.vm.$emit('update:value', 'queue')
    tabs.vm.$emit('update:value', 'invalid')

    expect(routerApi.push).toHaveBeenCalledTimes(1)
    expect(routerApi.push).toHaveBeenCalledWith('/integrations/notifications/queue')
  })
})
