// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

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
      props: ['value', 'show', 'loading', 'bordered', 'size', 'title'],
      emits: ['update:value', 'update:show', 'update:checked'],
      setup(_, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, [slots.header?.(), slots.default?.(), slots.footer?.()])
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    setup(_, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            onClick: (attrs as { onClick?: ((evt: MouseEvent) => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  const checkbox = vue.defineComponent({
    name: 'NCheckbox',
    props: ['checked'],
    emits: ['update:checked'],
    setup(props, { slots, emit }) {
      return () =>
        vue.h('label', { 'data-stub': 'NCheckbox' }, [
          vue.h('input', {
            type: 'checkbox',
            checked: !!(props as { checked?: boolean }).checked,
            onChange: (e: Event) => emit('update:checked', (e.target as HTMLInputElement).checked),
          }),
          slots.default?.(),
        ])
    },
  })

  const modal = vue.defineComponent({
    name: 'NModal',
    props: ['show'],
    emits: ['update:show'],
    setup(props, { slots }) {
      return () => ((props as { show?: boolean }).show ? vue.h('div', { 'data-stub': 'NModal' }, slots.default?.()) : null)
    },
  })

  return {
    NButton: button,
    NCard: stub('NCard'),
    NCheckbox: checkbox,
    NModal: modal,
    NSpace: stub('NSpace'),
    NSpin: stub('NSpin'),
    NTabPane: stub('NTabPane'),
    NTabs: stub('NTabs'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

const routeApi = {
  params: {} as Record<string, unknown>,
  path: '',
}
const routerApi = {
  push: vi.fn(),
}
vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => ({ items: [] }),
}))

vi.mock('@/components/jobs/JobEditorModal.vue', async () => {
  const vue = await import('vue')
  return {
    default: vue.defineComponent({ name: 'JobEditorModal', setup: () => () => vue.h('div', { 'data-stub': 'JobEditorModal' }) }),
  }
})

vi.mock('@/components/jobs/JobDeployModal.vue', async () => {
  const vue = await import('vue')
  return {
    default: vue.defineComponent({ name: 'JobDeployModal', setup: () => () => vue.h('div', { 'data-stub': 'JobDeployModal' }) }),
  }
})

const jobsApi = {
  getJob: vi.fn(),
  runNow: vi.fn(),
  archiveJob: vi.fn().mockResolvedValue(undefined),
  unarchiveJob: vi.fn().mockResolvedValue(undefined),
  deleteJob: vi.fn().mockResolvedValue(undefined),
}
vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

import JobDetailShellView from './JobDetailShellView.vue'

describe('JobDetailShellView toolbar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    routeApi.params = { nodeId: 'hub', jobId: 'job1' }
    routeApi.path = '/n/hub/jobs/job1'
    jobsApi.getJob.mockResolvedValue({
      id: 'job1',
      name: 'Job 1',
      agent_id: null,
      schedule: null,
      schedule_timezone: 'UTC',
      overlap_policy: 'queue',
      created_at: 1,
      updated_at: 2,
      archived_at: null,
      spec: { v: 1, type: 'filesystem' },
    })
    jobsApi.runNow.mockResolvedValue({ run_id: 'run1', status: 'queued' })
  })

  it('renders common actions in a toolbar', async () => {
    const wrapper = mount(JobDetailShellView, { global: { stubs: { 'router-view': true } } })
    await flushPromises()

    const buttons = wrapper.findAll('button[data-stub=\"NButton\"]').map((b) => b.text())
    expect(buttons).toContain('jobs.actions.runNow')
    expect(buttons).toContain('common.edit')
    expect(buttons).toContain('jobs.actions.deploy')
    expect(buttons).toContain('jobs.actions.archive')
    expect(buttons).toContain('common.delete')
  })

  it('runs job from toolbar', async () => {
    const wrapper = mount(JobDetailShellView, { global: { stubs: { 'router-view': true } } })
    await flushPromises()

    const runNowBtn = wrapper.findAll('button[data-stub=\"NButton\"]').find((b) => b.text() === 'jobs.actions.runNow')
    expect(runNowBtn).toBeTruthy()

    await runNowBtn!.trigger('click')
    await flushPromises()

    expect(jobsApi.runNow).toHaveBeenCalledWith('job1')
    expect(messageApi.success).toHaveBeenCalledWith('messages.runQueued')
  })
})
