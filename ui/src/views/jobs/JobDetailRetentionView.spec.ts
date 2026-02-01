// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { computed, ref } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'

import { JOB_DETAIL_CONTEXT, type JobDetailContext } from '@/lib/jobDetailContext'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
}

const jobsApi = {
  getJobRetention: vi.fn(),
  putJobRetention: vi.fn(),
  previewJobRetention: vi.fn(),
  applyJobRetention: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['loading', 'bordered', 'size', 'type', 'title', 'min'],
      emits: ['update:value', 'update:checked'],
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
    NAlert: stub('NAlert'),
    NButton: button,
    NCard: stub('NCard'),
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NIcon: stub('NIcon'),
    NInputNumber: stub('NInputNumber'),
    NSpin: stub('NSpin'),
    NSwitch: stub('NSwitch'),
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

import JobDetailRetentionView from './JobDetailRetentionView.vue'

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

describe('JobDetailRetentionView toolbar placement', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    jobsApi.getJobRetention.mockResolvedValue({
      enabled: false,
      keep_last: null,
      keep_days: null,
      max_delete_per_tick: 50,
      max_delete_per_day: 200,
    })
    jobsApi.putJobRetention.mockResolvedValue(undefined)
  })

  it('renders refresh/save actions inside the retention panel header', async () => {
    const wrapper = mount(JobDetailRetentionView, {
      global: {
        provide: provideJobContext(),
      },
    })

    await flushPromises()

    expect(wrapper.find('[data-testid="job-retention-panel"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="job-retention-refresh"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="job-retention-save"]').exists()).toBe(true)
  })
})

