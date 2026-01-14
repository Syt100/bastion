// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

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
    NAlert: stub('NAlert'),
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NDropdown: stub('NDropdown'),
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NInput: stub('NInput'),
    zhCN: {},
    enUS: {},
    dateZhCN: {},
    dateEnUS: {},
    useMessage: () => messageApi,
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

vi.mock('@/stores/system', () => ({
  useSystemStore: () => ({ insecureHttp: false }),
}))

const uiApi = {
  locale: 'zh-CN',
  darkMode: false,
  toggleDarkMode: vi.fn(),
  setLocale: vi.fn(),
}
vi.mock('@/stores/ui', () => ({
  useUiStore: () => uiApi,
}))

const apiFetchMock = vi.fn()
vi.mock('@/lib/api', () => ({
  apiFetch: (...args: unknown[]) => apiFetchMock(...args),
}))

import SetupView from './SetupView.vue'

describe('SetupView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows error when passwords do not match and does not call api', async () => {
    const wrapper = mount(SetupView)

    const vm = wrapper.vm as unknown as {
      password: string
      password2: string
      onSubmit: () => Promise<void>
    }

    vm.password = 'p1'
    vm.password2 = 'p2'

    await vm.onSubmit()

    expect(wrapper.text()).toContain('errors.passwordsDoNotMatch')
    expect(apiFetchMock).not.toHaveBeenCalled()
  })

  it('calls initialize API and redirects to login on success', async () => {
    apiFetchMock.mockResolvedValue(undefined)

    const wrapper = mount(SetupView)
    const vm = wrapper.vm as unknown as {
      username: string
      password: string
      password2: string
      onSubmit: () => Promise<void>
    }

    vm.username = 'admin'
    vm.password = 'p1'
    vm.password2 = 'p1'

    await vm.onSubmit()

    expect(apiFetchMock).toHaveBeenCalledWith(
      '/api/setup/initialize',
      expect.objectContaining({ method: 'POST', expectedStatus: 204 }),
    )
    expect(messageApi.success).toHaveBeenCalledWith('messages.initializedPleaseSignIn')
    expect(routerApi.push).toHaveBeenCalledWith('/login')
  })
})
