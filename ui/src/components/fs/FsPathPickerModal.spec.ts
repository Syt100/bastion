// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const messageApi = {
  error: vi.fn(),
  success: vi.fn(),
  warning: vi.fn(),
}

vi.mock('naive-ui', async () => {
  const vue = await import('vue')
  const stub = (name: string) =>
    vue.defineComponent({
      name,
      props: ['value', 'show', 'loading', 'columns', 'data'],
      emits: ['update:value', 'update:show', 'update:checked-row-keys'],
      setup(_, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, [slots.default?.(), slots.trigger?.()])
      },
    })

  return {
    NAlert: stub('NAlert'),
    NBadge: stub('NBadge'),
    NButton: stub('NButton'),
    NDataTable: stub('NDataTable'),
    NDrawer: stub('NDrawer'),
    NDrawerContent: stub('NDrawerContent'),
    NForm: stub('NForm'),
    NFormItem: stub('NFormItem'),
    NIcon: stub('NIcon'),
    NInput: stub('NInput'),
    NInputNumber: stub('NInputNumber'),
    NModal: stub('NModal'),
    NPopover: stub('NPopover'),
    NSelect: stub('NSelect'),
    NSpace: stub('NSpace'),
    NSwitch: stub('NSwitch'),
    NTag: stub('NTag'),
    useMessage: () => messageApi,
  }
})

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

import FsPathPickerModal from './FsPathPickerModal.vue'

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

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } })
}

async function flushAsync(): Promise<void> {
  await new Promise((r) => setTimeout(r, 0))
  await new Promise((r) => setTimeout(r, 0))
}

describe('FsPathPickerModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    stubMatchMedia(true)
  })

  it('remembers the last successfully listed directory per node', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      if (url.includes('path=%2Froot%2Fsub')) {
        return jsonResponse({ path: '/root/sub', entries: [] })
      }
      if (url.includes('path=%2Froot')) {
        return jsonResponse({ path: '/root', entries: [] })
      }
      return jsonResponse({ error: 'unexpected', message: `unexpected url: ${url}` }, 500)
    })
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    const wrapper = mount(FsPathPickerModal)
    ;(wrapper.vm as unknown as { open: (nodeId: 'hub' | string, initial?: string) => void }).open('hub', '/root')
    await flushAsync()

    ;(wrapper.vm as unknown as { currentPath: string; refresh: () => Promise<void> }).currentPath = '/root/sub'
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()

    ;(wrapper.vm as unknown as { open: (nodeId: 'hub' | string) => void }).open('hub')
    await flushAsync()

    const calls = fetchMock.mock.calls.map((c) => String(c[0]))
    expect(calls.at(-1)).toContain('path=%2Froot%2Fsub')
    expect(localStorage.getItem('bastion.fsPicker.lastDir.hub')).toBe('/root/sub')
  })

  it('requires a second confirmation click for not-found directories in single-dir mode', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      if (url.includes('path=%2Fmissing')) {
        return jsonResponse({ error: 'path_not_found', message: 'Path not found', details: { path: '/missing' } }, 404)
      }
      return jsonResponse({ error: 'unexpected', message: `unexpected url: ${url}` }, 500)
    })
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    const wrapper = mount(FsPathPickerModal)
    ;(wrapper.vm as unknown as { open: (nodeId: 'hub' | string, initial?: unknown) => void }).open('hub', {
      mode: 'single_dir',
      path: '/missing',
    })
    await flushAsync()

    await (wrapper.vm as unknown as { pick: () => Promise<void> }).pick()
    expect(wrapper.emitted('picked')).toBeUndefined()

    await (wrapper.vm as unknown as { pick: () => Promise<void> }).pick()
    expect(wrapper.emitted('picked')).toBeTruthy()
    expect(wrapper.emitted('picked')?.[0]).toEqual([['/missing']])
  })
})

