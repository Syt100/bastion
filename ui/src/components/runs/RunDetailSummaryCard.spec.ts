// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const i18nApi = {
  t: (key: string, params?: Record<string, unknown>) => {
    const count = params?.count
    return typeof count === 'number' ? `${key}:${count}` : key
  },
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['bordered', 'size', 'type', 'title'],
      setup(_, { slots, attrs }) {
        return () => vue.h('div', { 'data-stub': name, ...attrs }, slots.default?.())
      },
    })

  return {
    NAlert: stub('NAlert'),
    NCard: stub('NCard'),
    NTag: stub('NTag'),
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => i18nApi,
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

import RunDetailSummaryCard from './RunDetailSummaryCard.vue'

describe('RunDetailSummaryCard', () => {
  it('renders a source consistency warning tag when changes are detected', () => {
    const wrapper = mount(RunDetailSummaryCard, {
      props: {
        run: {
          id: 'run1',
          job_id: 'job1',
          status: 'success',
          started_at: 1000,
          ended_at: 1001,
          progress: null,
          error: null,
          summary: {
            filesystem: {
              consistency: {
                v: 1,
                changed_total: 2,
                replaced_total: 0,
                deleted_total: 0,
                read_error_total: 1,
                sample_truncated: false,
                sample: [],
              },
            },
          },
        },
        events: [],
      },
      global: {
        stubs: {
          RunProgressPanel: { template: '<div data-testid="progress" />' },
        },
      },
    })

    expect(wrapper.text()).toContain('runs.badges.sourceChanged:3')
  })

  it('does not render a source consistency warning tag when report totals are zero', () => {
    const wrapper = mount(RunDetailSummaryCard, {
      props: {
        run: {
          id: 'run1',
          job_id: 'job1',
          status: 'success',
          started_at: 1000,
          ended_at: 1001,
          progress: null,
          error: null,
          summary: {
            filesystem: {
              consistency: {
                v: 1,
                changed_total: 0,
                replaced_total: 0,
                deleted_total: 0,
                read_error_total: 0,
                sample_truncated: false,
                sample: [],
              },
            },
          },
        },
        events: [],
      },
      global: {
        stubs: {
          RunProgressPanel: { template: '<div data-testid="progress" />' },
        },
      },
    })

    expect(wrapper.text()).not.toContain('runs.badges.sourceChanged')
  })
})

