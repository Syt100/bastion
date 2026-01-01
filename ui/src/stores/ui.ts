import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { LOCALE_STORAGE_KEY, type SupportedLocale, setI18nLocale } from '@/i18n'

const STORAGE_KEY = 'bastion.ui.darkMode'

export const useUiStore = defineStore('ui', () => {
  function detectSystemDarkMode(): boolean {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false
    return window.matchMedia('(prefers-color-scheme: dark)').matches
  }

  const storedDarkMode = localStorage.getItem(STORAGE_KEY)
  const darkMode = ref<boolean>(storedDarkMode === null ? detectSystemDarkMode() : storedDarkMode === 'true')
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
