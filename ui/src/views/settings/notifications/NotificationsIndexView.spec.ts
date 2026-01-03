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

describe('NotificationsIndexView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('navigates to notification subpages', async () => {
    const wrapper = mount(NotificationsIndexView)
    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBe(4)

    await buttons[0]!.trigger('click')
    expect(routerApi.push).toHaveBeenCalledWith('/settings/notifications/channels')

    await buttons[1]!.trigger('click')
    expect(routerApi.push).toHaveBeenCalledWith('/settings/notifications/destinations')

    await buttons[2]!.trigger('click')
    expect(routerApi.push).toHaveBeenCalledWith('/settings/notifications/templates')

    await buttons[3]!.trigger('click')
    expect(routerApi.push).toHaveBeenCalledWith('/settings/notifications/queue')
  })
})

