// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
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
  let wrapper: ReturnType<typeof mount> | null = null

  beforeEach(() => {
    vi.clearAllMocks()
    stubMatchMedia(true)
  })

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('does not apply search until user clicks Search (or presses Enter)', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      return jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [], _url: url })
    })
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    wrapper = mount(RunEntriesPickerModal)
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

    wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    expect(fetchMock).toHaveBeenCalledTimes(1)

    ;(wrapper.vm as unknown as { hideDotfiles: boolean }).hideDotfiles = true
    ;(wrapper.vm as unknown as { onFiltersChanged: () => void }).onFiltersChanged()
    await flushAsync()

    expect(fetchMock).toHaveBeenCalledTimes(2)
    const secondCall = fetchMock.mock.calls[1] as unknown[] | undefined
    expect(String(secondCall?.[0])).toContain('hide_dotfiles=true')
  })

  it('splits selected entries into files and dirs', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [] }))
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    wrapper = mount(RunEntriesPickerModal)
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

  it('selects all loaded rows via selectAllLoadedRows', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [] }))
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    ;(wrapper.vm as unknown as { entries: Array<{ path: string; kind: string; size: number }> }).entries = [
      { path: 'etc', kind: 'dir', size: 0 },
      { path: 'a.txt', kind: 'file', size: 1 },
    ]

    ;(wrapper.vm as unknown as { selectAllLoadedRows: () => void }).selectAllLoadedRows()
    expect(new Set((wrapper.vm as unknown as { checkedRowKeys: string[] }).checkedRowKeys)).toEqual(new Set(['etc', 'a.txt']))
  })

  it('inverts selection for loaded rows only via invertLoadedRowsSelection', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [] }))
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    ;(wrapper.vm as unknown as { entries: Array<{ path: string; kind: string; size: number }> }).entries = [
      { path: 'a.txt', kind: 'file', size: 1 },
      { path: 'b.txt', kind: 'file', size: 1 },
    ]
    ;(wrapper.vm as unknown as { selected: Map<string, 'file' | 'dir'> }).selected = new Map([['a.txt', 'file'], ['other', 'file']])
    ;(wrapper.vm as unknown as { invertLoadedRowsSelection: () => void }).invertLoadedRowsSelection()

    expect(new Set((wrapper.vm as unknown as { checkedRowKeys: string[] }).checkedRowKeys)).toEqual(new Set(['b.txt', 'other']))
  })

  it('shift-selects a range within loaded rows', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [] }))
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    ;(wrapper.vm as unknown as { entries: Array<{ path: string; kind: string; size: number }> }).entries = [
      { path: 'a', kind: 'file', size: 1 },
      { path: 'b', kind: 'file', size: 1 },
      { path: 'c', kind: 'file', size: 1 },
      { path: 'd', kind: 'file', size: 1 },
    ]

    ;(wrapper.vm as unknown as { updateCheckedRowKeys: (keys: Array<string | number>) => void }).updateCheckedRowKeys(['b'])
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Shift', bubbles: true }))
    ;(wrapper.vm as unknown as { updateCheckedRowKeys: (keys: Array<string | number>) => void }).updateCheckedRowKeys(['b', 'd'])

    expect(new Set((wrapper.vm as unknown as { checkedRowKeys: string[] }).checkedRowKeys)).toEqual(new Set(['b', 'c', 'd']))
  })

  it('navigates to the parent prefix on Backspace when not typing in an input', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      return jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [], _url: url })
    })
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    ;(wrapper.vm as unknown as { prefix: string }).prefix = 'a/b'
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    await flushAsync()

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(String(fetchMock.mock.calls[1]?.[0])).toContain('prefix=a')
  })

  it('does not navigate on Backspace when the event target is an input', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [] }))
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    const input = document.createElement('input')
    document.body.appendChild(input)
    input.focus()
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    await flushAsync()

    expect(fetchMock).toHaveBeenCalledTimes(1)
    input.remove()
  })

  it('closes on Escape', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ prefix: '', cursor: 0, next_cursor: null, entries: [] }))
    vi.stubGlobal('fetch', fetchMock as unknown as typeof fetch)

    wrapper = mount(RunEntriesPickerModal)
    ;(wrapper.vm as unknown as { open: (runId: string) => void }).open('run-1')
    await flushAsync()

    expect((wrapper.vm as unknown as { show: boolean }).show).toBe(true)
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flushAsync()
    expect((wrapper.vm as unknown as { show: boolean }).show).toBe(false)
  })
})
