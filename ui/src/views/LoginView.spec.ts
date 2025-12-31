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
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NInput: stub('NInput'),
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

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/stores/system', () => ({
  useSystemStore: () => ({ insecureHttp: false }),
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
