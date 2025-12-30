import { createI18n } from 'vue-i18n'

import enUS from './locales/en-US'
import zhCN from './locales/zh-CN'

export const LOCALE_STORAGE_KEY = 'bastion.ui.locale'

export const supportedLocales = ['zh-CN', 'en-US'] as const
export type SupportedLocale = (typeof supportedLocales)[number]

function isSupportedLocale(locale: string): locale is SupportedLocale {
  return (supportedLocales as readonly string[]).includes(locale)
}

function initialLocale(): SupportedLocale {
  const stored = localStorage.getItem(LOCALE_STORAGE_KEY)
  if (stored && isSupportedLocale(stored)) return stored
  return 'zh-CN'
}

export const i18n = createI18n({
  legacy: false,
  locale: initialLocale(),
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})

export function setI18nLocale(locale: SupportedLocale): void {
  i18n.global.locale.value = locale
}

