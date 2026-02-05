// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const messageApi = {
  success: vi.fn(),
  error: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: [
        'show',
        'value',
        'loading',
        'bordered',
        'size',
        'type',
        'title',
        'options',
        'preset',
        'style',
        'placement',
        'width',
        'height',
        'consistentMenuWidth',
        'filterable',
        'clearable',
        'placeholder',
        'language',
        'code',
        'animated',
        'disabled',
        'trigger',
      ],
      emits: ['update:show', 'update:value', 'select'],
      setup(_, { slots, attrs }) {
        return () =>
          vue.h(
            'div',
            { 'data-stub': name, ...attrs },
            [slots.tab?.(), slots.header?.(), slots.default?.(), slots.footer?.()].filter(Boolean),
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

  const tag = vue.defineComponent({
    name: 'NTag',
    setup(_, { slots, attrs }) {
      return () => vue.h('span', { 'data-stub': 'NTag', ...attrs }, slots.default?.())
    },
  })

  return {
    NButton: button,
    NCard: stub('NCard'),
    NCode: stub('NCode'),
    NDataTable: stub('NDataTable'),
    NDrawer: stub('NDrawer'),
    NDrawerContent: stub('NDrawerContent'),
    NInput: stub('NInput'),
    NModal: stub('NModal'),
    NSelect: stub('NSelect'),
    NSpace: stub('NSpace'),
    NTabs: stub('NTabs'),
    NTabPane: stub('NTabPane'),
    NTag: tag,
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const count = params?.count
      return typeof count === 'number' ? `${key}:${count}` : key
    },
  }),
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
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

import RunDetailDetailsTabs from './RunDetailDetailsTabs.vue'

describe('RunDetailDetailsTabs consistency section', () => {
  it('renders breakdown + samples and can jump to filtered consistency events', async () => {
    stubMatchMedia(true)

    const wrapper = mount(RunDetailDetailsTabs, {
      props: {
        runId: 'run1',
        wsStatus: 'live',
        ops: [],
        summary: {
          filesystem: {
            consistency: {
              v: 2,
              changed_total: 1,
              replaced_total: 0,
              deleted_total: 0,
              read_error_total: 0,
              sample_truncated: false,
              sample: [
                {
                  path: 'a.txt',
                  reason: 'mtime_changed',
                  before: { size_bytes: 1, mtime_unix_nanos: 1 },
                  after_handle: { size_bytes: 1, mtime_unix_nanos: 2 },
                  after_path: { size_bytes: 1, mtime_unix_nanos: 2 },
                },
              ],
            },
          },
        },
        events: [
          {
            run_id: 'run1',
            seq: 1,
            ts: 1,
            level: 'warn',
            kind: 'source_consistency',
            message: 'source consistency warnings',
            fields: { changed_total: 1, replaced_total: 0, deleted_total: 0, read_error_total: 0 },
          },
          {
            run_id: 'run1',
            seq: 2,
            ts: 2,
            level: 'info',
            kind: 'upload',
            message: 'upload',
            fields: null,
          },
        ],
      },
    })

    await flushPromises()

    expect(wrapper.find('[data-testid="run-detail-consistency"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('a.txt')
    expect(wrapper.text()).toContain('mtime_changed')
    expect(wrapper.text()).toContain('runs.consistency.evidence')

    // Before filtering, both events are visible.
    expect(wrapper.text()).toContain('upload')

    const btn = wrapper
      .findAll('button')
      .find((b) => b.text().trim() === 'runs.consistency.viewEvents')
    expect(btn).toBeTruthy()

    await btn!.trigger('click')
    await flushPromises()

    // After clicking, the event kind filter should hide unrelated events.
    expect(wrapper.text()).toContain('source consistency warnings')
    expect(wrapper.text()).not.toContain('upload')
  })
})
