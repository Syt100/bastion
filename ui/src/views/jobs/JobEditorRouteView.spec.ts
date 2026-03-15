// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { reactive } from 'vue'

import { createInitialJobEditorForm } from '@/components/jobs/editor/form'
import type { JobEditorForm } from '@/components/jobs/editor/types'

const messageApi = {
  success: vi.fn(),
  warning: vi.fn(),
  error: vi.fn(),
}

const routeApi = reactive({
  params: {} as Record<string, unknown>,
  query: {} as Record<string, unknown>,
  path: '/jobs/new',
  fullPath: '/jobs/new',
  hash: '',
})

function buildFullPath(location: unknown): string {
  if (typeof location === 'string') return location
  if (!location || typeof location !== 'object') return '/'
  const path = typeof (location as { path?: unknown }).path === 'string' ? (location as { path: string }).path : '/'
  const query = (location as { query?: Record<string, unknown> }).query ?? {}
  const params = new URLSearchParams()
  for (const [key, raw] of Object.entries(query)) {
    if (raw == null) continue
    if (Array.isArray(raw)) {
      raw.forEach((value) => {
        if (value != null) params.append(key, String(value))
      })
      continue
    }
    params.set(key, String(raw))
  }
  const suffix = params.toString()
  return suffix ? `${path}?${suffix}` : path
}

const routerApi = {
  push: vi.fn().mockResolvedValue(undefined),
  replace: vi.fn().mockResolvedValue(undefined),
  resolve: vi.fn((location: unknown) => ({ fullPath: buildFullPath(location) })),
}

const jobsApi = {
  getJob: vi.fn(),
  createJob: vi.fn(),
  updateJob: vi.fn(),
}

const agentsApi = reactive({
  items: [{ id: 'agent-1', name: 'DB Node', online: true, revoked: false }],
  refresh: vi.fn().mockResolvedValue(undefined),
})

const secretsApi = {
  webdav: [{ name: 'webdav-main' }],
}

const notificationsApi = reactive({
  destinations: [] as Array<{ name: string; channel: string; enabled: boolean }>,
  refreshDestinations: vi.fn().mockResolvedValue(undefined),
})

const systemApi = {
  hubTimezone: 'UTC',
  refresh: vi.fn().mockResolvedValue(undefined),
}

const hubRuntimeConfigApi = {
  get: vi.fn().mockResolvedValue({ saved: { default_backup_retention: null } }),
}

vi.mock('vue-router', () => ({
  useRoute: () => routeApi,
  useRouter: () => routerApi,
}))

vi.mock('vue-i18n', async () => {
  const vue = await import('vue')
  return {
    useI18n: () => ({
      t: (key: string, params?: Record<string, unknown>) => (params ? `${key}:${JSON.stringify(params)}` : key),
    }),
    createI18n: () => ({
      global: {
        locale: vue.ref('en-US'),
        t: (key: string, params?: Record<string, unknown>) => (params ? `${key}:${JSON.stringify(params)}` : key),
      },
    }),
  }
})

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['current', 'type', 'show', 'bordered', 'loading', 'disabled', 'title'],
      emits: ['update:current', 'update:show'],
      setup(_props, { slots, attrs }) {
        return () => vue.h('div', { 'data-stub': name, ...attrs }, [slots.default?.(), slots.footer?.()].filter(Boolean))
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    props: ['disabled', 'loading'],
    setup(props, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            disabled: !!props.disabled,
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  return {
    NAlert: stub('NAlert'),
    NButton: button,
    NCard: stub('NCard'),
    NForm: stub('NForm'),
    NStep: stub('NStep'),
    NSteps: stub('NSteps'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

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

function makeDraftEnvelope(mode: 'create' | 'edit', values: JobEditorForm, options?: { jobId?: string; baseJobUpdatedAt?: number | null; step?: number }) {
  return {
    version: 1,
    mode,
    jobId: options?.jobId ?? null,
    baseJobUpdatedAt: options?.baseJobUpdatedAt ?? null,
    step: options?.step ?? 1,
    values,
    updatedAt: 1710000000000,
  }
}

function resetRoute(options?: { path?: string; fullPath?: string; query?: Record<string, unknown>; params?: Record<string, unknown> }): void {
  routeApi.path = options?.path ?? '/jobs/new'
  routeApi.fullPath = options?.fullPath ?? '/jobs/new'
  routeApi.query = options?.query ?? {}
  routeApi.params = options?.params ?? {}
  routeApi.hash = ''
}

function mountEditor() {
  return mount(JobEditorRouteView, {
    global: {
      stubs: {
        PageHeader: { template: '<div data-stub="PageHeader"><slot /></div>' },
        FsPathPickerModal: { template: '<div data-stub="FsPathPickerModal" />' },
        JobEditorStepBasicsOnly: { template: '<div data-stub="JobEditorStepBasicsOnly" />' },
        JobEditorStepSource: { template: '<div data-stub="JobEditorStepSource" />' },
        JobEditorStepTarget: { template: '<div data-stub="JobEditorStepTarget" />' },
        JobEditorStepScheduleRetention: { template: '<div data-stub="JobEditorStepScheduleRetention" />' },
        JobEditorStepSecurity: { template: '<div data-stub="JobEditorStepSecurity" />' },
        JobEditorStepNotifications: { template: '<div data-stub="JobEditorStepNotifications" />' },
        JobEditorStepReview: { template: '<div data-stub="JobEditorStepReview" />' },
      },
    },
  })
}

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

import JobEditorRouteView from './JobEditorRouteView.vue'

describe('JobEditorRouteView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
    resetRoute({ path: '/jobs/new', fullPath: '/jobs/new?scope=hub', query: { scope: 'hub' } })
    localStorage.clear()
    jobsApi.getJob.mockReset()
    jobsApi.createJob.mockReset()
    jobsApi.updateJob.mockReset()
    jobsApi.createJob.mockResolvedValue({
      id: 'job-new',
      name: 'Nightly DB',
    })
    routerApi.push.mockReset()
    routerApi.push.mockResolvedValue(undefined)
    routerApi.replace.mockReset()
    routerApi.replace.mockResolvedValue(undefined)
    routerApi.resolve.mockImplementation((location: unknown) => ({ fullPath: buildFullPath(location) }))
    agentsApi.refresh.mockReset()
    agentsApi.refresh.mockResolvedValue(undefined)
    notificationsApi.refreshDestinations.mockReset()
    notificationsApi.refreshDestinations.mockResolvedValue(undefined)
    systemApi.refresh.mockReset()
    systemApi.refresh.mockResolvedValue(undefined)
    hubRuntimeConfigApi.get.mockReset()
    hubRuntimeConfigApi.get.mockResolvedValue({ saved: { default_backup_retention: null } })
  })

  it('offers to resume a saved create draft and restores the saved form state', async () => {
    const draftValues = createInitialJobEditorForm()
    draftValues.name = 'Nightly DB'
    draftValues.jobType = 'filesystem'
    draftValues.fsPaths = ['/var/lib/postgresql']
    draftValues.targetType = 'local_dir'
    draftValues.localBaseDir = '/srv/backups'

    localStorage.setItem(
      'bastion.jobs.editor.createDraft',
      JSON.stringify(makeDraftEnvelope('create', draftValues, { step: 4 })),
    )

    const wrapper = mountEditor()
    await flushPromises()

    expect(wrapper.text()).toContain('jobs.editor.draftResumeNotice')

    const resume = wrapper.findAll('button').find((node) => node.text() === 'jobs.editor.resumeDraft')
    expect(resume).toBeTruthy()
    await resume!.trigger('click')

    const vm = wrapper.vm as unknown as { step: number; form: JobEditorForm }
    expect(vm.step).toBe(4)
    expect(vm.form.name).toBe('Nightly DB')
    expect(vm.form.localBaseDir).toBe('/srv/backups')
  })

  it('keeps live edit state until a stale draft is explicitly resumed', async () => {
    resetRoute({
      path: '/jobs/job-1/edit',
      fullPath: '/jobs/job-1/edit?scope=hub',
      query: { scope: 'hub' },
      params: { jobId: 'job-1' },
    })

    jobsApi.getJob.mockResolvedValue({
      id: 'job-1',
      name: 'Live job',
      agent_id: null,
      schedule: '0 0 * * *',
      schedule_timezone: 'UTC',
      overlap_policy: 'queue',
      created_at: 1,
      updated_at: 20,
      archived_at: null,
      spec: {
        v: 1,
        type: 'filesystem',
        source: { paths: ['/live'] },
        target: { type: 'local_dir', base_dir: '/live-backups' },
      },
    })

    const draftValues = createInitialJobEditorForm()
    draftValues.name = 'Draft job'
    draftValues.jobType = 'filesystem'
    draftValues.fsPaths = ['/draft']
    draftValues.targetType = 'local_dir'
    draftValues.localBaseDir = '/draft-backups'
    localStorage.setItem(
      'bastion.jobs.editor.editDraft.job-1',
      JSON.stringify(makeDraftEnvelope('edit', draftValues, { jobId: 'job-1', baseJobUpdatedAt: 10, step: 3 })),
    )

    const wrapper = mountEditor()
    await flushPromises()

    expect(wrapper.text()).toContain('jobs.editor.draftStaleNotice')
    expect(wrapper.text()).toContain('jobs.editor.riskLabels.stale_draft')

    const vm = wrapper.vm as unknown as { step: number; form: JobEditorForm }
    expect(vm.form.name).toBe('Live job')

    const resume = wrapper.findAll('button').find((node) => node.text() === 'jobs.editor.resumeDraft')
    expect(resume).toBeTruthy()
    await resume!.trigger('click')

    expect(vm.step).toBe(3)
    expect(vm.form.name).toBe('Draft job')
  })

  it('uses compact mobile step navigation and keeps summaries collapsed by default', async () => {
    stubMatchMedia(false)

    const wrapper = mountEditor()
    await flushPromises()

    expect(wrapper.find('[data-testid="job-editor-mobile-progress"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="job-editor-summary-body"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="job-editor-risks-body"]').exists()).toBe(false)

    await wrapper.find('[data-testid="job-editor-mobile-step-toggle"]').trigger('click')
    expect(wrapper.find('[data-testid="job-editor-mobile-step-list"]').exists()).toBe(true)

    await wrapper.find('[data-testid="job-editor-toggle-summary"]').trigger('click')
    expect(wrapper.find('[data-testid="job-editor-summary-body"]').exists()).toBe(true)

    await wrapper.find('[data-testid="job-editor-toggle-risks"]').trigger('click')
    expect(wrapper.find('[data-testid="job-editor-risks-body"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="job-editor-mobile-action-warning"]').exists()).toBe(true)
  })

  it('saves from review, clears the draft, and returns to the Jobs collection context', async () => {
    resetRoute({
      path: '/jobs/new',
      fullPath: '/jobs/new?scope=hub&status=failed',
      query: { scope: 'hub', status: 'failed' },
    })

    const wrapper = mountEditor()
    await flushPromises()

    const vm = wrapper.vm as unknown as { step: number; form: JobEditorForm; save: () => Promise<void> }
    vm.form.name = 'Nightly DB'
    vm.form.jobType = 'filesystem'
    vm.form.fsPaths = ['/data']
    vm.form.targetType = 'local_dir'
    vm.form.localBaseDir = '/srv/backups'
    vm.step = 7
    await wrapper.vm.$nextTick()

    expect(localStorage.getItem('bastion.jobs.editor.createDraft')).not.toBeNull()

    await vm.save()

    expect(jobsApi.createJob).toHaveBeenCalledTimes(1)
    expect(localStorage.getItem('bastion.jobs.editor.createDraft')).toBeNull()
    expect(messageApi.success).toHaveBeenCalledWith('messages.jobCreated')
    expect(routerApi.push).toHaveBeenCalledWith('/jobs?scope=hub&status=failed')
  })
})
