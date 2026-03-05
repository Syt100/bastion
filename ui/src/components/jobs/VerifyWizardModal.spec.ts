// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const operationsApi = {
  startVerify: vi.fn(),
}

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

vi.mock('@/stores/operations', () => ({
  useOperationsStore: () => operationsApi,
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/components/AppModalShell.vue', async () => {
  const vue = await import('vue')
  return {
    default: vue.defineComponent({
      name: 'AppModalShell',
      props: ['show'],
      emits: ['update:show'],
      setup(props, { slots }) {
        return () => (props.show ? vue.h('div', { 'data-stub': 'AppModalShell' }, [slots.default?.(), slots.footer?.()]) : null)
      },
    }),
  }
})

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })
  return {
    NAlert: stub('NAlert'),
    NButton: stub('NButton'),
    useMessage: () => messageApi,
  }
})

import VerifyWizardModal from './VerifyWizardModal.vue'

describe('VerifyWizardModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('starts verify operation and emits started', async () => {
    operationsApi.startVerify.mockResolvedValue('op-verify-1')

    const wrapper = mount(VerifyWizardModal)
    const vm = wrapper.vm as unknown as { open: (runId: string) => void; start: () => Promise<void> }

    vm.open('run-1')
    await vm.start()

    expect(operationsApi.startVerify).toHaveBeenCalledWith('run-1')
    expect(wrapper.emitted('started')?.[0]).toEqual(['op-verify-1'])
  })

  it('shows error feedback when verify start fails', async () => {
    operationsApi.startVerify.mockRejectedValue(new Error('boom'))

    const wrapper = mount(VerifyWizardModal)
    const vm = wrapper.vm as unknown as { open: (runId: string) => void; start: () => Promise<void> }

    vm.open('run-2')
    await vm.start()

    expect(messageApi.error).toHaveBeenCalled()
    expect(wrapper.emitted('started')).toBeUndefined()
  })
})
