// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const routerPush = vi.fn()

const agentsApi = {
  items: [{ id: 'agent-1', name: 'Agent 1', revoked: false }],
  refresh: vi.fn(),
  listLabelIndex: vi.fn(),
}

const jobsApi = {
  items: [{ id: 'job-1', name: 'Daily Backup' }],
}

const bulkOpsApi = {
  previewJobDeploy: vi.fn(),
  create: vi.fn(),
}

const messageApi = {
  success: vi.fn(),
  warning: vi.fn(),
  error: vi.fn(),
}

vi.mock('vue-router', () => ({
  useRouter: () => ({ push: routerPush }),
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

vi.mock('@/stores/bulkOperations', () => ({
  useBulkOperationsStore: () => bulkOpsApi,
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

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'options'],
      emits: ['update:value'],
      setup(_props, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })
  return {
    NButton: stub('NButton'),
    NCard: stub('NCard'),
    NDataTable: stub('NDataTable'),
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NInput: stub('NInput'),
    NRadioButton: stub('NRadioButton'),
    NRadioGroup: stub('NRadioGroup'),
    NSelect: stub('NSelect'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

import JobDeployModal from './JobDeployModal.vue'

describe('JobDeployModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    agentsApi.refresh.mockResolvedValue(undefined)
    agentsApi.listLabelIndex.mockResolvedValue([{ label: 'prod', count: 1 }])
  })

  it('previews and deploys to bulk operation page', async () => {
    bulkOpsApi.previewJobDeploy.mockResolvedValue({
      items: [{ agent_id: 'agent-1', agent_name: 'Agent 1', planned_name: 'Daily Backup (agent-1)', valid: true, error: null }],
    })
    bulkOpsApi.create.mockResolvedValue('op-1')

    const wrapper = mount(JobDeployModal)
    const vm = wrapper.vm as unknown as {
      open: (jobId: string) => Promise<void>
      selectedLabels: string[]
      previewDeploy: () => Promise<void>
      deploy: () => Promise<void>
    }

    await vm.open('job-1')
    expect(agentsApi.listLabelIndex).toHaveBeenCalled()

    vm.selectedLabels = ['prod']
    await vm.previewDeploy()
    expect(bulkOpsApi.previewJobDeploy).toHaveBeenCalled()

    await vm.deploy()
    expect(bulkOpsApi.create).toHaveBeenCalled()
    expect(routerPush).toHaveBeenCalledWith({ path: '/system/bulk-operations', query: { open: 'op-1' } })
  })

  it('blocks deploy when preview is missing', async () => {
    const wrapper = mount(JobDeployModal)
    const vm = wrapper.vm as unknown as {
      open: (jobId: string) => Promise<void>
      selectedLabels: string[]
      deploy: () => Promise<void>
    }

    await vm.open('job-1')
    vm.selectedLabels = ['prod']
    await vm.deploy()

    expect(bulkOpsApi.create).not.toHaveBeenCalled()
    expect(routerPush).not.toHaveBeenCalled()
  })
})
