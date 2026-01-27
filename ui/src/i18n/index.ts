import { createI18n } from 'vue-i18n'

import enUS from './locales/en-US'
import zhCN from './locales/zh-CN'

export const LOCALE_STORAGE_KEY = 'bastion.ui.locale'
export const LOCALE_COOKIE_NAME = 'bastion_locale'

export const supportedLocales = ['en-US', 'zh-CN'] as const
export type SupportedLocale = (typeof supportedLocales)[number]

function isSupportedLocale(locale: string): locale is SupportedLocale {
  return (supportedLocales as readonly string[]).includes(locale)
}

function safeLocalStorageGet(key: string): string | null {
  if (typeof window === 'undefined' || typeof window.localStorage === 'undefined') return null
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}

function safeLocalStorageSet(key: string, value: string): void {
  if (typeof window === 'undefined' || typeof window.localStorage === 'undefined') return
  try {
    localStorage.setItem(key, value)
  } catch {
    // ignore
  }
}

function readCookieValue(name: string): string | null {
  if (typeof document === 'undefined') return null
  const raw = document.cookie
  if (!raw) return null
  for (const part of raw.split(';')) {
    const trimmed = part.trim()
    if (!trimmed) continue
    const idx = trimmed.indexOf('=')
    const key = idx === -1 ? trimmed : trimmed.slice(0, idx)
    if (key !== name) continue
    const value = idx === -1 ? '' : trimmed.slice(idx + 1)
    try {
      return decodeURIComponent(value)
    } catch {
      return value
    }
  }
  return null
}

function writeCookieValue(name: string, value: string): void {
  if (typeof document === 'undefined') return
  const maxAgeSeconds = 60 * 60 * 24 * 365
  const isSecure = typeof location !== 'undefined' && location.protocol === 'https:'
  const encoded = encodeURIComponent(value)

  // Keep cookie readable by JS so UI can resolve locale before hitting the API.
  let cookie = `${name}=${encoded}; Path=/; Max-Age=${maxAgeSeconds}; SameSite=Lax`
  if (isSecure) cookie += '; Secure'
  document.cookie = cookie
}

function normalizeLocale(raw: string | null | undefined): SupportedLocale | null {
  if (!raw) return null
  const v = raw.trim()
  if (!v) return null
  const lower = v.toLowerCase()
  if (lower.startsWith('zh')) return 'zh-CN'
  if (lower.startsWith('en')) return 'en-US'
  if (isSupportedLocale(v)) return v
  return null
}

function localeFromBrowser(): SupportedLocale {
  if (typeof navigator === 'undefined') return 'en-US'
  const langs: string[] = (navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language]).filter(
    (v): v is string => typeof v === 'string' && v.trim().length > 0,
  )
  for (const lang of langs) {
    if (lang.trim().toLowerCase().startsWith('zh')) return 'zh-CN'
  }
  return 'en-US'
}

export function resolveInitialLocale(): SupportedLocale {
  const stored = normalizeLocale(safeLocalStorageGet(LOCALE_STORAGE_KEY))
  if (stored) return stored

  const cookie = normalizeLocale(readCookieValue(LOCALE_COOKIE_NAME))
  if (cookie) return cookie

  return localeFromBrowser()
}

export function persistLocalePreference(locale: SupportedLocale): void {
  safeLocalStorageSet(LOCALE_STORAGE_KEY, locale)
  writeCookieValue(LOCALE_COOKIE_NAME, locale)
}

export const i18n = createI18n({
  legacy: false,
  locale: resolveInitialLocale(),
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})

export function setI18nLocale(locale: SupportedLocale): void {
  i18n.global.locale.value = locale
}
