// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const jobsApi = {
  items: [],
  createJob: vi.fn(),
  updateJob: vi.fn(),
  getJob: vi.fn(),
}

const agentsApi = {
  items: [],
}

const secretsApi = {
  webdav: [],
  wecomBots: [],
  smtp: [],
}

const notificationsApi = {
  destinations: [],
  refreshDestinations: vi.fn(),
}

const systemApi = {
  hubTimezone: 'UTC',
  refresh: vi.fn(),
}

const hubRuntimeConfigApi = {
  get: vi.fn(),
}

const messageApi = {
  success: vi.fn(),
  warning: vi.fn(),
  error: vi.fn(),
}

const { editorFormToRequestMock, jobDetailToEditorFormMock, validateJobEditorUpToStepMock } = vi.hoisted(() => ({
  editorFormToRequestMock: vi.fn(() => ({ payload: true })),
  jobDetailToEditorFormMock: vi.fn(() => ({ id: 'job-1', name: 'Loaded' })),
  validateJobEditorUpToStepMock: vi.fn(() => []),
}))

vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

vi.mock('@/stores/agents', () => ({
  useAgentsStore: () => agentsApi,
}))

vi.mock('@/stores/secrets', () => ({
  useSecretsStore: () => secretsApi,
}))

vi.mock('@/stores/notifications', () => ({
  useNotificationsStore: () => notificationsApi,
}))

vi.mock('@/stores/system', () => ({
  useSystemStore: () => systemApi,
}))

vi.mock('@/stores/hubRuntimeConfig', () => ({
  useHubRuntimeConfigStore: () => hubRuntimeConfigApi,
}))

vi.mock('@/lib/media', () => ({
  useMediaQuery: () => ({ value: true }),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('./editor/mapping', () => ({
  editorFormToRequest: editorFormToRequestMock,
  jobDetailToEditorForm: jobDetailToEditorFormMock,
}))

vi.mock('./editor/validation', () => ({
  stepForJobEditorField: () => 1,
  validateJobEditorUpToStep: validateJobEditorUpToStepMock,
}))

vi.mock('@/components/AppModalShell.vue', async () => {
  const vue = await import('vue')
  return {
    default: vue.defineComponent({
      name: 'AppModalShell',
      props: ['show', 'style', 'contentStyle', 'scrollBody'],
      setup(props, { slots }) {
        return () => (props.show ? vue.h('div', { 'data-stub': 'AppModalShell' }, [slots.default?.(), slots.footer?.()]) : null)
      },
    }),
  }
})

vi.mock('@/components/fs/FsPathPickerModal.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'FsPathPickerModal', setup: () => () => vue.h('div') }) }
})

vi.mock('./editor/steps/JobEditorStepBasics.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'JobEditorStepBasics', setup: () => () => vue.h('div') }) }
})

vi.mock('./editor/steps/JobEditorStepSource.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'JobEditorStepSource', setup: () => () => vue.h('div') }) }
})

vi.mock('./editor/steps/JobEditorStepTarget.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'JobEditorStepTarget', setup: () => () => vue.h('div') }) }
})

vi.mock('./editor/steps/JobEditorStepSecurity.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'JobEditorStepSecurity', setup: () => () => vue.h('div') }) }
})

vi.mock('./editor/steps/JobEditorStepNotifications.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'JobEditorStepNotifications', setup: () => () => vue.h('div') }) }
})

vi.mock('./editor/steps/JobEditorStepReview.vue', async () => {
  const vue = await import('vue')
  return { default: vue.defineComponent({ name: 'JobEditorStepReview', setup: () => () => vue.h('div') }) }
})

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      setup(_props, { slots, attrs }) {
        return () => vue.h('div', { 'data-stub': name, ...attrs }, slots.default?.())
      },
    })
  return {
    NButton: stub('NButton'),
    NForm: stub('NForm'),
    NStep: stub('NStep'),
    NSteps: stub('NSteps'),
    useMessage: () => messageApi,
  }
})

import JobEditorModal from './JobEditorModal.vue'

async function flush(): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
}

describe('JobEditorModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    systemApi.refresh.mockResolvedValue(undefined)
    hubRuntimeConfigApi.get.mockResolvedValue({ saved: { default_backup_retention: null } })
    notificationsApi.refreshDestinations.mockResolvedValue(undefined)
    jobsApi.createJob.mockResolvedValue(undefined)
  })

  it('creates job on save in create mode', async () => {
    const wrapper = mount(JobEditorModal)
    const vm = wrapper.vm as unknown as {
      openCreate: (ctx?: { nodeId?: string }) => void
      save: () => Promise<void>
    }

    vm.openCreate({ nodeId: 'hub' })
    await flush()
    await vm.save()

    expect(validateJobEditorUpToStepMock).toHaveBeenCalled()
    expect(editorFormToRequestMock).toHaveBeenCalled()
    expect(jobsApi.createJob).toHaveBeenCalledWith({ payload: true })
    expect(wrapper.emitted('saved')).toBeTruthy()
  })

  it('shows error and closes when openEdit fails to load job', async () => {
    jobsApi.getJob.mockRejectedValue(new Error('failed'))

    const wrapper = mount(JobEditorModal)
    const vm = wrapper.vm as unknown as {
      openEdit: (jobId: string, ctx?: { nodeId?: string }) => Promise<void>
    }

    await vm.openEdit('job-404', { nodeId: 'hub' })

    expect(messageApi.error).toHaveBeenCalled()
    expect(wrapper.find('[data-stub="AppModalShell"]').exists()).toBe(false)
  })

  it('applies desktop height bounds at modal container layer', async () => {
    const wrapper = mount(JobEditorModal)
    const vm = wrapper.vm as unknown as {
      openCreate: (ctx?: { nodeId?: string }) => void
    }

    vm.openCreate({ nodeId: 'hub' })
    await flush()

    const modal = wrapper.getComponent({ name: 'AppModalShell' })
    const style = modal.props('style') as Record<string, string>
    expect(style.height).toBe('min(85vh, calc(100vh - 64px))')
    expect(style.maxHeight).toBe('calc(100vh - 64px)')
    expect(modal.props('contentStyle')).toBeUndefined()
    expect(modal.props('scrollBody')).toBe(false)
  })
})
