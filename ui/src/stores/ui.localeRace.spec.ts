import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const i18nMocks = vi.hoisted(() => {
  const pending = new Map<string, { promise: Promise<void>; resolve: () => void }>()

  const ensureLocaleMessages = vi.fn((locale: string) => {
    const cached = pending.get(locale)
    if (cached) return cached.promise

    let resolve!: () => void
    const promise = new Promise<void>((r) => {
      resolve = r
    })
    pending.set(locale, { promise, resolve })
    return promise
  })

  return {
    pending,
    ensureLocaleMessages,
    persistLocalePreference: vi.fn(),
    resolveInitialLocale: vi.fn(() => 'en-US'),
    setI18nLocale: vi.fn(),
  }
})

vi.mock('@/i18n', () => ({
  ensureLocaleMessages: i18nMocks.ensureLocaleMessages,
  persistLocalePreference: i18nMocks.persistLocalePreference,
  resolveInitialLocale: i18nMocks.resolveInitialLocale,
  setI18nLocale: i18nMocks.setI18nLocale,
}))

import { useUiStore } from './ui'

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

async function flushMicrotasks(): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
}

describe('ui locale switching', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
    stubMatchMedia(false)

    i18nMocks.pending.clear()
    i18nMocks.ensureLocaleMessages.mockClear()
    i18nMocks.persistLocalePreference.mockClear()
    i18nMocks.resolveInitialLocale.mockClear()
    i18nMocks.setI18nLocale.mockClear()
  })

  it('keeps last-write-wins semantics for rapid locale switching', async () => {
    const ui = useUiStore()

    ui.setLocale('zh-CN')
    ui.setLocale('en-US')

    i18nMocks.pending.get('en-US')?.resolve()
    await flushMicrotasks()

    i18nMocks.pending.get('zh-CN')?.resolve()
    await flushMicrotasks()

    const calls = i18nMocks.setI18nLocale.mock.calls
    expect(calls[calls.length - 1]?.[0]).toBe('en-US')
    expect(i18nMocks.setI18nLocale.mock.calls.some(([v]) => v === 'zh-CN')).toBe(false)
  })
})
