import { beforeEach, describe, expect, it } from 'vitest'

import { LOCALE_COOKIE_NAME, LOCALE_STORAGE_KEY, resolveInitialLocale } from '@/i18n'

function setLocaleCookie(value: string): void {
  document.cookie = `${LOCALE_COOKIE_NAME}=${encodeURIComponent(value)}; Path=/`
}

function clearLocaleCookie(): void {
  document.cookie = `${LOCALE_COOKIE_NAME}=; Max-Age=0; Path=/`
}

function stubNavigatorLanguages(languages: string[] | undefined, language: string | undefined): void {
  Object.defineProperty(window.navigator, 'languages', { value: languages, configurable: true })
  Object.defineProperty(window.navigator, 'language', { value: language, configurable: true })
}

describe('i18n locale resolution', () => {
  beforeEach(() => {
    localStorage.clear()
    clearLocaleCookie()
    stubNavigatorLanguages(['en-US'], 'en-US')
  })

  it('prefers localStorage over cookie and browser language', () => {
    localStorage.setItem(LOCALE_STORAGE_KEY, 'en-US')
    setLocaleCookie('zh-CN')
    stubNavigatorLanguages(['zh-CN'], 'zh-CN')

    expect(resolveInitialLocale()).toBe('en-US')
  })

  it('uses cookie when localStorage is missing', () => {
    setLocaleCookie('zh-CN')
    stubNavigatorLanguages(['en-US'], 'en-US')

    expect(resolveInitialLocale()).toBe('zh-CN')
  })

  it('falls back to browser language when no stored preference exists', () => {
    stubNavigatorLanguages(['zh-TW', 'en-US'], 'en-US')

    expect(resolveInitialLocale()).toBe('zh-CN')
  })

  it('defaults to en-US when no language info is available', () => {
    stubNavigatorLanguages([], '')

    expect(resolveInitialLocale()).toBe('en-US')
  })
})

