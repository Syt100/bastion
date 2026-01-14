// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      setup(_, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })

  return {
    NCard: stub('NCard'),
    NIcon: stub('NIcon'),
  }
})

const routerApi = {
  push: vi.fn(),
  replace: vi.fn(),
}
vi.mock('vue-router', () => ({
  useRouter: () => routerApi,
}))

vi.mock('vue-i18n', async (importOriginal) => {
  const actual = await importOriginal<typeof import('vue-i18n')>()
  return {
    ...actual,
    useI18n: () => ({ t: (key: string) => key }),
  }
})

import NotificationsIndexView from './NotificationsIndexView.vue'
import { DEFAULT_NOTIFICATIONS_TAB_KEY, getNotificationsNavItems } from '@/navigation/notifications'

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

describe('NotificationsIndexView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(false)
  })

  it('navigates to notification subpages', async () => {
    const wrapper = mount(NotificationsIndexView)
    const buttons = wrapper.findAll('button')
    const items = getNotificationsNavItems()
    expect(buttons.length).toBe(items.length)

    for (let i = 0; i < items.length; i += 1) {
      await buttons[i]!.trigger('click')
      expect(routerApi.push).toHaveBeenCalledWith(items[i]!.to)
    }
  })

  it('redirects to destinations on desktop', async () => {
    stubMatchMedia(true)
    mount(NotificationsIndexView)
    await Promise.resolve()
    const fallback = getNotificationsNavItems().find((i) => i.key === DEFAULT_NOTIFICATIONS_TAB_KEY)!
    expect(routerApi.replace).toHaveBeenCalledWith(fallback.to)
  })
})
