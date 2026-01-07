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
      props: ['value', 'show', 'loading', 'columns', 'data', 'checkedRowKeys'],
      emits: ['update:value', 'update:show', 'update:checked-row-keys'],
      setup(_, { slots }) {
        return () => vue.h('div', { 'data-stub': name }, [slots.default?.(), slots.trigger?.()])
      },
    })

  return {
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

import RunEntriesPickerModal from './RunEntriesPickerModal.vue'

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

describe('RunEntriesPickerModal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
  })

  it('does not apply search until user clicks Search (or presses Enter)', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      return jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [], _url: url })
    })
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    const wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(String(fetchMock.mock.calls[0]?.[0])).not.toContain('q=')

    ;(wrapper.vm as unknown as { searchDraft: string }).searchDraft = 'ssh'
    await flushAsync()
    expect(fetchMock).toHaveBeenCalledTimes(1)

    ;(wrapper.vm as unknown as { applySearch: () => void }).applySearch()
    await flushAsync()
    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(String(fetchMock.mock.calls[1]?.[0])).toContain('q=ssh')
  })

  it('applies filter changes immediately via a refresh', async () => {
    const fetchMock = vi.fn(async () =>
      jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [] }),
    )
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    const wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    expect(fetchMock).toHaveBeenCalledTimes(1)

    ;(wrapper.vm as unknown as { hideDotfiles: boolean }).hideDotfiles = true
    ;(wrapper.vm as unknown as { onFiltersChanged: () => void }).onFiltersChanged()
    await flushAsync()

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(String(fetchMock.mock.calls[1]?.[0])).toContain('hide_dotfiles=true')
  })

  it('splits selected entries into files and dirs', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [] }))
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    const wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    ;(wrapper.vm as unknown as { entries: Array<{ path: string; kind: string; size: number }> }).entries = [
      { path: 'etc', kind: 'dir', size: 0 },
      { path: 'a.txt', kind: 'file', size: 1 },
    ]
    ;(wrapper.vm as unknown as { updateCheckedRowKeys: (keys: Array<string | number>) => void }).updateCheckedRowKeys([
      'etc',
      'a.txt',
    ])

    ;(wrapper.vm as unknown as { pick: () => void }).pick()
    expect(wrapper.emitted('picked')?.[0]).toEqual([{ dirs: ['etc'], files: ['a.txt'] }])
  })
})
