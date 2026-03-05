// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const agentsApi = {
  items: [],
}

const secretsApi = {
  webdav: [],
  refreshWebdav: vi.fn(),
}

const operationsApi = {
  startRestore: vi.fn(),
}

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/secrets', () => ({
  useSecretsStore: () => secretsApi,
}))

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
      setup(props, { slots }) {
        return () => (props.show ? vue.h('div', { 'data-stub': 'AppModalShell' }, [slots.default?.(), slots.footer?.()]) : null)
      },
    }),
  }
})

vi.mock('@/components/jobs/RunEntriesPickerModal.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'RunEntriesPickerModal', setup: () => () => vue.h('div') }) }
})

vi.mock('@/components/fs/FsPathPickerModal.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'FsPathPickerModal', setup: () => () => vue.h('div') }) }
})

vi.mock('@/components/webdav/WebdavPathPickerModal.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'WebdavPathPickerModal', setup: () => () => vue.h('div') }) }
})

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value'],
      emits: ['update:value'],
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })
  return {
    NButton: stub('NButton'),
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NInput: stub('NInput'),
    NSelect: stub('NSelect'),
    useMessage: () => messageApi,
  }
})

import RestoreWizardModal from './RestoreWizardModal.vue'

describe('RestoreWizardModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    secretsApi.refreshWebdav.mockResolvedValue(undefined)
  })

  it('validates local destination before start', async () => {
    const wrapper = mount(RestoreWizardModal)
    const vm = wrapper.vm as unknown as {
      open: (runId: string, opts?: { defaultNodeId?: string | null }) => void
      start: () => Promise<void>
    }

    vm.open('run-1', { defaultNodeId: 'node-a' })
    await vm.start()

    expect(messageApi.error).toHaveBeenCalledWith('errors.restoreDestinationRequired')
    expect(operationsApi.startRestore).not.toHaveBeenCalled()
  })

  it('starts restore with local_fs destination and emits started', async () => {
    operationsApi.startRestore.mockResolvedValue('op-restore-1')

    const wrapper = mount(RestoreWizardModal)
    const vm = wrapper.vm as unknown as {
      open: (runId: string, opts?: { defaultNodeId?: string | null }) => void
      start: () => Promise<void>
      localFsDirectory: string
    }

    vm.open('run-2', { defaultNodeId: 'node-b' })
    vm.localFsDirectory = '/restore/dir'
    await vm.start()

    expect(operationsApi.startRestore).toHaveBeenCalled()
    expect(wrapper.emitted('started')?.[0]).toEqual(['op-restore-1'])
  })
})
