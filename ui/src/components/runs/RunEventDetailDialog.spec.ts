// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const wrapper = (name: string) =>
    vue.defineComponent({
      name,
      props: ['show', 'title'],
      emits: ['update:show'],
      setup(_, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, slots.default?.())
      },
    })

  const button = vue.defineComponent({
    name: 'NButton',
    setup(_, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            type: 'button',
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  const tag = vue.defineComponent({
    name: 'NTag',
    setup(_, { slots }) {
      return () => vue.h('span', { 'data-stub': 'NTag' }, slots.default?.())
    },
  })

  return {
    NButton: button,
    NDrawer: wrapper('NDrawer'),
    NDrawerContent: wrapper('NDrawerContent'),
    NModal: wrapper('NModal'),
    NSpace: wrapper('NSpace'),
    NTag: tag,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}))

vi.mock('@/stores/ui', () => ({
  useUiStore: () => ({ locale: 'en-US' }),
}))

vi.mock('@/lib/datetime', () => ({
  useUnixSecondsFormatter: () => ({
    formatUnixSeconds: (ts: number | null) => `ts-${ts ?? '-'}`,
  }),
}))

vi.mock('./RunEventDetailContent.vue', async () => {
  const vue = await import('vue')
  return {
    default: vue.defineComponent({
      name: 'RunEventDetailContent',
      setup() {
        return () => vue.h('div', { 'data-stub': 'RunEventDetailContent' }, 'content')
      },
    }),
  }
})

import RunEventDetailDialog from './RunEventDetailDialog.vue'

const sampleEvent = {
  run_id: 'run-1',
  seq: 12,
  ts: 1700000000,
  level: 'error',
  kind: 'failed',
  message: 'failed',
  fields: {
    trace_id: 'trace-abc',
    error_envelope: {
      transport: {
        provider_request_id: 'req-1',
      },
    },
  },
}

describe('RunEventDetailDialog', () => {
  it('renders default header schema items', () => {
    const wrapper = mount(RunEventDetailDialog, {
      props: {
        show: true,
        event: sampleEvent,
        isDesktop: true,
        title: 'title',
        closeLabel: 'close',
      },
    })

    expect(wrapper.text()).toContain('ts-1700000000')
    expect(wrapper.text()).toContain('error')
    expect(wrapper.text()).toContain('failed')
    expect(wrapper.text()).not.toContain('#12')
  })

  it('supports custom header schema with request/trace metadata', () => {
    const wrapper = mount(RunEventDetailDialog, {
      props: {
        show: true,
        event: sampleEvent,
        isDesktop: true,
        title: 'title',
        closeLabel: 'close',
        headerMetaFields: ['seq', 'requestId', 'traceId', 'kind'],
      },
    })

    const text = wrapper.text()
    expect(text).toContain('#12')
    expect(text).toContain('runEvents.details.labels.providerRequestId: req-1')
    expect(text).toContain('runEvents.details.labels.traceId: trace-abc')
    expect(text).not.toContain('ts-1700000000')
  })
})
