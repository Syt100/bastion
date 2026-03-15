// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const messageApi = {
  success: vi.fn(),
  error: vi.fn(),
}

vi.mock('@/lib/clipboard', () => ({
  copyText: vi.fn().mockResolvedValue(true),
}))

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
        'placeholder',
        'language',
        'code',
        'animated',
        'disabled',
        'filterable',
        'clearable',
        'pagination',
        'data',
        'columns',
      ],
      emits: ['update:show', 'update:value', 'select'],
      setup(_, { slots, attrs }) {
        return () =>
          vue.h(
            'div',
            { 'data-stub': name, ...attrs },
            [slots.tab?.(), slots.default?.(), slots.footer?.()].filter(Boolean),
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

  const input = vue.defineComponent({
    name: 'NInput',
    props: ['value', 'placeholder', 'size', 'clearable'],
    emits: ['update:value'],
    setup(props, { emit }) {
      return () =>
        vue.h('input', {
          'data-stub': 'NInput',
          value: props.value ?? '',
          placeholder: props.placeholder,
          onInput: (event: Event) => {
            const target = event.target as HTMLInputElement
            emit('update:value', target.value)
          },
        })
    },
  })

  const select = vue.defineComponent({
    name: 'NSelect',
    props: ['value', 'options', 'placeholder', 'disabled', 'size', 'filterable', 'clearable'],
    emits: ['update:value'],
    setup(props, { emit }) {
      return () =>
        vue.h(
          'select',
          {
            'data-stub': 'NSelect',
            value: props.value ?? '',
            disabled: !!props.disabled,
            onChange: (event: Event) => {
              const target = event.target as HTMLSelectElement
              const next = target.value || null
              emit('update:value', next)
            },
          },
          [
            vue.h('option', { value: '' }, ''),
            ...(((props.options as Array<{ label: string; value: string }> | undefined) ?? []).map((option) =>
              vue.h('option', { value: option.value }, option.label),
            )),
          ],
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
    NInput: input,
    NSelect: select,
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
      if (typeof count === 'number') return `${key}:${count}`
      if (typeof params?.first === 'number' || typeof params?.last === 'number') {
        return `${key}:${params?.first ?? '-'}-${params?.last ?? '-'}`
      }
      return key
    },
  }),
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('@/components/runs/RunEventDetailDialog.vue', () => ({
  default: {
    name: 'RunEventDetailDialog',
    props: ['show', 'event', 'isDesktop', 'title', 'closeLabel'],
    template:
      '<div data-testid="run-event-detail-dialog" :data-show="String(show)" :data-desktop="String(isDesktop)" :data-close-label="closeLabel">{{ event?.message }}</div>',
  },
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

function buttonByText(wrapper: ReturnType<typeof mount>, text: string) {
  return wrapper.findAll('button').find((button) => button.text().trim() === text)
}

describe('RunDetailDetailsTabs', () => {
  it('emits filter and pagination events for the server-driven event console', async () => {
    stubMatchMedia(true)

    const wrapper = mount(RunDetailDetailsTabs, {
      props: {
        runId: 'run-1',
        events: [
          {
            run_id: 'run-1',
            seq: 8,
            ts: 1710000000,
            level: 'error',
            kind: 'upload_failed',
            message: 'WebDAV upload failed',
            fields: null,
          },
        ],
        consoleLoading: false,
        window: {
          first_seq: 8,
          last_seq: 8,
          has_older: true,
          has_newer: true,
        },
        locators: {
          first_error_seq: 8,
          root_cause_seq: 8,
        },
        filters: {
          search: '',
          level: null,
          kind: null,
        },
        ops: [],
        wsStatus: 'error',
        summary: null,
      },
    })

    await wrapper.get('input[data-stub="NInput"]').setValue('webdav')
    const selects = wrapper.findAll('select[data-stub="NSelect"]')
    expect(selects).toHaveLength(2)
    await selects[0]!.setValue('error')
    await selects[1]!.setValue('upload_failed')

    await buttonByText(wrapper, 'runEvents.actions.firstError')!.trigger('click')
    await buttonByText(wrapper, 'runs.detail.loadOlderEvents')!.trigger('click')
    await buttonByText(wrapper, 'runs.detail.loadNewerEvents')!.trigger('click')
    await buttonByText(wrapper, 'runEvents.actions.reconnect')!.trigger('click')

    expect(wrapper.text()).toContain('runs.detail.eventWindow:8-8')
    expect(wrapper.emitted('update:search')).toEqual([['webdav']])
    expect(wrapper.emitted('update:level')).toEqual([['error']])
    expect(wrapper.emitted('update:kind')).toEqual([['upload_failed']])
    expect(wrapper.emitted('jump-first-error')).toHaveLength(1)
    expect(wrapper.emitted('load-older')).toHaveLength(1)
    expect(wrapper.emitted('load-newer')).toHaveLength(1)
    expect(wrapper.emitted('reconnect')).toHaveLength(1)
  })

  it('renders structured summary context and opens the event detail dialog with desktop props', async () => {
    stubMatchMedia(true)

    const wrapper = mount(RunDetailDetailsTabs, {
      props: {
        runId: 'run-1',
        events: [
          {
            run_id: 'run-1',
            seq: 42,
            ts: 1710000000,
            level: 'warn',
            kind: 'source_consistency',
            message: 'Source files changed while packaging',
            fields: { changed_total: 1 },
          },
        ],
        consoleLoading: false,
        window: {
          first_seq: 42,
          last_seq: 42,
          has_older: false,
          has_newer: false,
        },
        locators: {
          first_error_seq: null,
          root_cause_seq: null,
        },
        filters: {
          search: '',
          level: null,
          kind: null,
        },
        ops: [],
        wsStatus: 'live',
        summary: {
          target: {
            type: 'local_dir',
            run_dir: '/srv/backups/nightly',
          },
          entries_count: 12,
          parts: 3,
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
                  path: 'db.sqlite3',
                  reason: 'mtime_changed',
                  before: { mtime_unix_nanos: 1 },
                  after_handle: { mtime_unix_nanos: 2 },
                },
              ],
            },
          },
        },
      },
    })

    expect(wrapper.get('[data-testid="run-detail-consistency"]').text()).toContain('runs.badges.sourceChanged:1')
    expect(wrapper.text()).toContain('/srv/backups/nightly')
    expect(wrapper.text()).toContain('runs.detail.entries:12')
    expect(wrapper.text()).toContain('runs.detail.parts:3')

    await buttonByText(wrapper, 'runEvents.actions.details')!.trigger('click')
    await flushPromises()

    const dialog = wrapper.get('[data-testid="run-event-detail-dialog"]')
    expect(dialog.attributes('data-show')).toBe('true')
    expect(dialog.attributes('data-desktop')).toBe('true')
    expect(dialog.attributes('data-close-label')).toBe('common.close')
    expect(dialog.text()).toContain('Source files changed while packaging')
  })
})
