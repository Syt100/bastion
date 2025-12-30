import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { LOCALE_STORAGE_KEY, type SupportedLocale, setI18nLocale } from '@/i18n'

const STORAGE_KEY = 'bastion.ui.darkMode'

export const useUiStore = defineStore('ui', () => {
  const darkMode = ref<boolean>(localStorage.getItem(STORAGE_KEY) === 'true')
  const locale = ref<SupportedLocale>(
    (localStorage.getItem(LOCALE_STORAGE_KEY) as SupportedLocale | null) ?? 'zh-CN',
  )
  const themeMode = computed(() => (darkMode.value ? 'dark' : 'light'))

  function setDarkMode(value: boolean): void {
    darkMode.value = value
    localStorage.setItem(STORAGE_KEY, String(value))
  }

  function setLocale(value: SupportedLocale): void {
    locale.value = value
    localStorage.setItem(LOCALE_STORAGE_KEY, value)
    setI18nLocale(value)
  }

  function toggleDarkMode(): void {
    setDarkMode(!darkMode.value)
  }

  return { darkMode, locale, themeMode, setDarkMode, setLocale, toggleDarkMode }
})
