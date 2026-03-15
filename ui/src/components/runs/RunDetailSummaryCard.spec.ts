// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

import type { RunWorkspaceDetail } from '@/stores/runs'

const i18nApi = {
  t: (key: string, params?: Record<string, unknown>) => {
    const count = params?.count
    if (typeof count === 'number') return `${key}:${count}`
    if (typeof params?.seq === 'number') return `${key}:${params.seq}`
    return key
  },
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['bordered', 'size', 'type', 'title'],
      setup(props, { slots, attrs }) {
        return () =>
          vue.h('div', { 'data-stub': name, ...attrs }, [
            props.title ? vue.h('div', { 'data-title': name }, String(props.title)) : null,
            slots.default?.(),
          ])
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

function makeDetail(overrides: Partial<RunWorkspaceDetail> = {}): RunWorkspaceDetail {
  return {
    run: {
      id: 'run-1',
      job_id: 'job-1',
      job_name: 'Nightly backup',
      scope: 'hub',
      node_id: 'hub',
      node_name: 'Hub',
      status: 'failed',
      kind: 'backup',
      started_at: 100,
      ended_at: 220,
      error: 'Upload failed after remote timeout',
    },
    progress: null,
    summary: {
      target: {
        type: 'webdav',
        run_url: 'https://dav.example.com/nightly/run-1',
      },
      filesystem: {
        warnings_total: 2,
        errors_total: 1,
      },
    },
    diagnostics: {
      state: 'structured',
      failure_kind: 'transport',
      failure_stage: 'upload',
      failure_title: 'WebDAV upload failed',
      failure_hint: 'Retry after confirming network stability.',
      first_error_event_seq: 42,
      root_cause_event_seq: 42,
    },
    capabilities: {
      can_cancel: false,
      can_restore: true,
      can_verify: true,
    },
    related: {
      operations_total: 3,
      artifacts_total: 2,
    },
    ...overrides,
  }
}

describe('RunDetailSummaryCard', () => {
  it('renders structured diagnostics and related run context for failed runs', () => {
    const wrapper = mount(RunDetailSummaryCard, {
      props: {
        detail: makeDetail(),
        events: [],
      },
      global: {
        stubs: {
          RunProgressPanel: { template: '<div data-testid="progress" />' },
        },
      },
    })

    expect(wrapper.text()).toContain('WebDAV upload failed')
    expect(wrapper.text()).toContain('Upload failed after remote timeout')
    expect(wrapper.text()).toContain('Retry after confirming network stability.')
    expect(wrapper.text()).toContain('runs.detail.failureKind: transport')
    expect(wrapper.text()).toContain('runs.detail.failureStage: upload')
    expect(wrapper.text()).toContain('runs.detail.firstErrorSeq:42')
    expect(wrapper.text()).toContain('runs.detail.errors:1')
    expect(wrapper.text()).toContain('runs.detail.warnings:2')
    expect(wrapper.text()).toContain('jobs.targets.webdav')
    expect(wrapper.text()).toContain('https://dav.example.com/nightly/run-1')
    expect(wrapper.text()).toContain('3')
    expect(wrapper.text()).toContain('2')
  })

  it('omits the failure alert for successful runs while keeping overview content visible', () => {
    const wrapper = mount(RunDetailSummaryCard, {
      props: {
        detail: makeDetail({
          run: {
            ...makeDetail().run,
            status: 'success',
            error: null,
          },
          diagnostics: {
            ...makeDetail().diagnostics,
            failure_kind: null,
            failure_stage: null,
            failure_hint: null,
            first_error_event_seq: null,
          },
        }),
        events: [],
      },
      global: {
        stubs: {
          RunProgressPanel: { template: '<div data-testid="progress" />' },
        },
      },
    })

    expect(wrapper.find('[data-stub="NAlert"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="run-detail-overview"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="progress"]').exists()).toBe(true)
  })
})
