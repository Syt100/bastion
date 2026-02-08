// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

import { createNaiveStub } from '@/test-utils/naiveUiStubs'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const stub = (name: string) => createNaiveStub(name)

  return {
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

const authApi = {
  login: vi.fn(),
}
vi.mock('@/stores/auth', () => ({
  useAuthStore: () => authApi,
}))

const apiFetchMock = vi.fn()
vi.mock('@/lib/api', () => ({
  apiFetch: (...args: unknown[]) => apiFetchMock(...args),
}))

import LoginView from './LoginView.vue'

describe('LoginView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('redirects to /setup when setup is required', async () => {
    apiFetchMock.mockResolvedValue({ needs_setup: true })

    mount(LoginView)

    // onMounted async
    await Promise.resolve()
    await Promise.resolve()

    expect(routerApi.replace).toHaveBeenCalledWith('/setup')
  })

  it('calls auth.login and navigates to home on submit', async () => {
    apiFetchMock.mockResolvedValue({ needs_setup: false })
    authApi.login.mockResolvedValue(undefined)

    const wrapper = mount(LoginView)
    const vm = wrapper.vm as unknown as {
      username: string
      password: string
      onSubmit: () => Promise<void>
    }

    vm.username = 'admin'
    vm.password = 'p1'

    await vm.onSubmit()

    expect(authApi.login).toHaveBeenCalledWith('admin', 'p1')
    expect(routerApi.push).toHaveBeenCalledWith('/')
  })
})
