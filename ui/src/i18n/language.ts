import { dateEnUS, dateZhCN, enUS, zhCN } from 'naive-ui'

import { supportedLocales, type SupportedLocale } from '@/i18n'

export type LocaleDropdownOption = {
  label: string
  key: SupportedLocale
}

export const LOCALE_LABELS: Record<SupportedLocale, string> = {
  'zh-CN': '简体中文',
  'en-US': 'English',
}

export function getLocaleDropdownOptions(): LocaleDropdownOption[] {
  return supportedLocales.map((locale) => ({
    key: locale,
    label: LOCALE_LABELS[locale],
  }))
}

type NaiveLocale = typeof zhCN
type NaiveDateLocale = typeof dateZhCN

export const NAIVE_UI_LOCALES: Record<SupportedLocale, NaiveLocale> = {
  'zh-CN': zhCN,
  'en-US': enUS,
}

export const NAIVE_UI_DATE_LOCALES: Record<SupportedLocale, NaiveDateLocale> = {
  'zh-CN': dateZhCN,
  'en-US': dateEnUS,
}

