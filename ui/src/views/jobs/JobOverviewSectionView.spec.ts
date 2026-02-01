// @vitest-environment jsdom
import { computed, ref } from 'vue'
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

import { JOB_DETAIL_CONTEXT, type JobDetailContext } from '@/lib/jobDetailContext'

const messageApi = {
  error: vi.fn(),
}

const jobsApi = {
  listRuns: vi.fn(),
}

const routerApi = {
  push: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string, opts?: { respectShow?: boolean }) =>
    vue.defineComponent({
      name,
      props: ['show', 'loading', 'bordered', 'size', 'type', 'title', 'disabled', 'tertiary'],
      emits: ['update:show'],
      setup(props, { slots, attrs }) {
        return () => {
          if (opts?.respectShow && 'show' in props && !(props as { show?: boolean }).show) {
            return vue.h('div', { 'data-stub': name })
          }
          return vue.h(
            'div',
            { 'data-stub': name, ...attrs },
            [slots.header?.(), slots['header-extra']?.(), slots.default?.(), slots.footer?.()].filter(Boolean),
          )
        }
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    props: ['disabled'],
    setup(props, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            'data-stub': 'NButton',
            disabled: !!props.disabled,
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
            ...attrs,
          },
          slots.default?.(),
        )
    },
  })

  return {
    NButton: button,
    NCard: stub('NCard'),
    NSpin: stub('NSpin'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => (params ? `${key}:${JSON.stringify(params)}` : key),
  }),
}))

vi.mock('@/stores/jobs', () => ({
  useJobsStore: () => jobsApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('vue-router', () => ({
  useRouter: () => routerApi,
}))

import JobOverviewSectionView from './JobOverviewSectionView.vue'

function provideJobContext(overrides?: Partial<JobDetailContext>): Record<symbol, JobDetailContext> {
  const base: JobDetailContext = {
    nodeId: computed(() => 'hub'),
    jobId: computed(() => 'job1'),
    job: ref({
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
    }),
    loading: ref(false),
    refresh: vi.fn().mockResolvedValue(undefined),
  }
  return {
    [JOB_DETAIL_CONTEXT as unknown as symbol]: { ...base, ...(overrides ?? {}) },
  }
}

describe('JobOverviewSectionView run summary', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders latest run and opens run drawer from Overview', async () => {
    const now = Math.floor(Date.now() / 1000)
    jobsApi.listRuns.mockResolvedValue([
      { id: 'run1', status: 'success', started_at: now, ended_at: now + 10, error: null, executed_offline: false },
    ])

    const wrapper = mount(JobOverviewSectionView, {
      global: {
        provide: provideJobContext(),
        stubs: {
          AppEmptyState: true,
        },
      },
    })

    await flushPromises()

    expect(wrapper.find('[data-testid="job-overview-run-summary"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('success')

    const open = wrapper.find('[data-testid="job-overview-open-latest-run"]')
    expect(open.exists()).toBe(true)
    await open.trigger('click')

    expect(routerApi.push).toHaveBeenCalledWith('/n/hub/jobs/job1/overview/runs/run1')
  })

  it('shows empty/zero state and disables open action when no runs exist', async () => {
    jobsApi.listRuns.mockResolvedValue([])

    const wrapper = mount(JobOverviewSectionView, {
      global: {
        provide: provideJobContext(),
        stubs: {
          AppEmptyState: true,
        },
      },
    })

    await flushPromises()

    expect(wrapper.text()).toContain('jobs.workspace.overview.runs7dEmpty')

    const open = wrapper.find('[data-testid="job-overview-open-latest-run"]')
    expect(open.attributes('disabled')).toBeDefined()
  })
})

