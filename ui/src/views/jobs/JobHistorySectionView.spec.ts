// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { computed, ref } from 'vue'
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

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['loading', 'bordered', 'size', 'type', 'title'],
      setup(_, { slots, attrs }) {
        return () =>
          vue.h(
            'div',
            { 'data-stub': name, ...attrs },
            [slots.header?.(), slots['header-extra']?.(), slots.default?.(), slots.footer?.()].filter(Boolean),
          )
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    props: ['disabled'],
    setup(props, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          { 'data-stub': 'NButton', disabled: !!props.disabled, onClick: (attrs as { onClick?: (() => void) | undefined }).onClick, ...attrs },
          slots.default?.(),
        )
    },
  })

  return {
    NButton: button,
    NCard: stub('NCard'),
    NDataTable: stub('NDataTable'),
    NIcon: stub('NIcon'),
    NSpace: stub('NSpace'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
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

import JobHistorySectionView from './JobHistorySectionView.vue'

function provideJobContext(): Record<symbol, JobDetailContext> {
  return {
    [JOB_DETAIL_CONTEXT as unknown as symbol]: {
      nodeId: computed(() => 'hub'),
      jobId: computed(() => 'job1'),
      job: ref(null),
      loading: ref(false),
      refresh: vi.fn().mockResolvedValue(undefined),
    },
  }
}

describe('JobHistorySectionView layout', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
    jobsApi.listRuns.mockResolvedValue([])
  })

  it('renders actions in the list panel header and removes the old summary grid', async () => {
    const wrapper = mount(JobHistorySectionView, {
      global: {
        provide: provideJobContext(),
        stubs: {
          AppEmptyState: true,
          RunEventsModal: true,
          RestoreWizardModal: true,
          VerifyWizardModal: true,
          OperationModal: true,
        },
      },
    })

    await flushPromises()

    expect(wrapper.find('[data-testid="job-history-panel"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="job-history-refresh"]').exists()).toBe(true)

    // Regression guard: the removed summary grid referenced the latest run label.
    expect(wrapper.text()).not.toContain('runs.latestRun')
  })
})

