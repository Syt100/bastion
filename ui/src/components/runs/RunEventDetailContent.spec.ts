// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const messageApi = {
  success: vi.fn(),
  error: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')

  const button = vue.defineComponent({
    name: 'NButton',
    setup(_, { slots, attrs }) {
      return () =>
        vue.h(
          'button',
          {
            type: 'button',
            'data-stub': 'NButton',
            onClick: (attrs as { onClick?: (() => void) | undefined }).onClick,
          },
          slots.default?.(),
        )
    },
  })

  const code = vue.defineComponent({
    name: 'NCode',
    props: ['code'],
    setup(props, { attrs }) {
      return () => vue.h('pre', { 'data-stub': 'NCode', ...attrs }, String((props as { code?: unknown }).code ?? ''))
    },
  })

  return {
    NButton: button,
    NCode: code,
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (!params) return key
      return key.replace(/\{(\w+)\}/g, (_, token: string) => String(params[token] ?? ''))
    },
  }),
}))

const copyTextMock = vi.fn(async (_value: string) => true)
vi.mock('@/lib/clipboard', () => ({
  copyText: (value: string) => copyTextMock(value),
}))

import RunEventDetailContent from './RunEventDetailContent.vue'

const sampleEvent = {
  run_id: 'run1',
  seq: 1,
  ts: 1,
  level: 'error',
  kind: 'failed',
  message: 'failed',
  fields: {
    error_chain: ['first line', 'second line', 'third line'],
    error_envelope: {
      schema_version: '1.0',
      code: 'target.webdav.timeout',
      kind: 'timeout',
      retriable: { value: true, reason: 'timeout', retry_after_sec: 10 },
      message: { key: 'diagnostics.message.target.webdav.put_failed', params: { attempt: 3, max_attempts: 3 } },
      hint: { key: 'diagnostics.hint.run_failed.timeout', params: {} },
      transport: { protocol: 'http', provider_request_id: 'req-copy-1' },
      context: {
        source: 'webdav_put',
        attempt: 3,
        max_attempts: 3,
        target_url: 'https://example.com/upload',
      },
    },
  },
}

describe('RunEventDetailContent', () => {
  it('renders copy actions for code/request id/target url and triggers copy', async () => {
    const wrapper = mount(RunEventDetailContent, {
      props: {
        event: sampleEvent,
      },
    })

    const codeCopy = wrapper.find('[data-testid="run-event-copy-code"]')
    const reqCopy = wrapper.find('[data-testid="run-event-copy-providerRequestId"]')
    const urlCopy = wrapper.find('[data-testid="run-event-copy-target_url"]')
    expect(codeCopy.exists()).toBe(true)
    expect(reqCopy.exists()).toBe(true)
    expect(urlCopy.exists()).toBe(true)

    await codeCopy.trigger('click')
    expect(copyTextMock).toHaveBeenCalledWith('target.webdav.timeout')
    expect(messageApi.success).toHaveBeenCalled()
  })

  it('shows compact error-chain preview and keeps raw JSON collapsed by default', () => {
    const wrapper = mount(RunEventDetailContent, {
      props: {
        event: sampleEvent,
      },
    })

    const chainEntries = wrapper.findAll('[data-testid="run-event-error-chain-entry"]')
    expect(chainEntries).toHaveLength(2)
    expect(chainEntries[0]?.text()).toContain('first line')
    expect(chainEntries[1]?.text()).toContain('second line')
    expect(chainEntries[1]?.text()).not.toContain('third line')

    const rawJson = wrapper.find('[data-testid="run-event-raw-json"]')
    expect(rawJson.exists()).toBe(true)
    expect(rawJson.attributes('style')).toContain('display: none')
    expect(wrapper.find('[data-testid="run-event-raw-toggle"]').text()).toContain('runEvents.details.actions.showRaw')
  })

  it('expands error-chain and raw JSON sections when user toggles actions', async () => {
    const wrapper = mount(RunEventDetailContent, {
      props: {
        event: sampleEvent,
      },
    })

    const expandChainButton = wrapper.find('[data-testid="run-event-error-chain-toggle"]')
    expect(expandChainButton.exists()).toBe(true)
    await expandChainButton.trigger('click')

    const chainEntries = wrapper.findAll('[data-testid="run-event-error-chain-entry"]')
    expect(chainEntries).toHaveLength(3)
    expect(chainEntries[2]?.text()).toContain('third line')

    await wrapper.find('[data-testid="run-event-raw-toggle"]').trigger('click')

    const rawJson = wrapper.find('[data-testid="run-event-raw-json"]')
    expect(rawJson.attributes('style') ?? '').not.toContain('display: none')
    expect(wrapper.text()).toContain('error_chain')
  })
})
