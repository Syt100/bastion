import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { persistLocalePreference, resolveInitialLocale, type SupportedLocale, setI18nLocale } from '@/i18n'
import { DEFAULT_UI_THEME_ID, isUiThemeId, type UiThemeId } from '@/theme/presets'
import {
  DEFAULT_UI_BACKGROUND_STYLE,
  isUiBackgroundStyle,
  UI_BACKGROUND_STYLE_KEY,
  type UiBackgroundStyle,
} from '@/theme/background'

const STORAGE_KEY = 'bastion.ui.darkMode'
const PREFERRED_NODE_KEY = 'bastion.ui.preferredNodeId'
const THEME_KEY = 'bastion.ui.themeId'

export const useUiStore = defineStore('ui', () => {
  function detectSystemDarkMode(): boolean {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false
    return window.matchMedia('(prefers-color-scheme: dark)').matches
  }

  const storedDarkMode = localStorage.getItem(STORAGE_KEY)
  const darkMode = ref<boolean>(storedDarkMode === null ? detectSystemDarkMode() : storedDarkMode === 'true')
  const themeId = ref<UiThemeId>(
    (() => {
      const raw = localStorage.getItem(THEME_KEY)
      return isUiThemeId(raw) ? raw : DEFAULT_UI_THEME_ID
    })(),
  )
  const backgroundStyle = ref<UiBackgroundStyle>(
    (() => {
      const raw = localStorage.getItem(UI_BACKGROUND_STYLE_KEY)
      return isUiBackgroundStyle(raw) ? raw : DEFAULT_UI_BACKGROUND_STYLE
    })(),
  )
  const locale = ref<SupportedLocale>(resolveInitialLocale())
  const preferredNodeId = ref<string>(localStorage.getItem(PREFERRED_NODE_KEY) || 'hub')
  const themeMode = computed(() => (darkMode.value ? 'dark' : 'light'))

  // Ensure i18n and docs entrypoint use the same initial locale preference.
  persistLocalePreference(locale.value)
  setI18nLocale(locale.value)

  function setDarkMode(value: boolean): void {
    darkMode.value = value
    localStorage.setItem(STORAGE_KEY, String(value))
  }

  function setLocale(value: SupportedLocale): void {
    locale.value = value
    persistLocalePreference(value)
    setI18nLocale(value)
  }

  function toggleDarkMode(): void {
    setDarkMode(!darkMode.value)
  }

  function setThemeId(value: UiThemeId): void {
    themeId.value = value
    localStorage.setItem(THEME_KEY, value)
  }

  function setBackgroundStyle(value: UiBackgroundStyle): void {
    backgroundStyle.value = value
    localStorage.setItem(UI_BACKGROUND_STYLE_KEY, value)
  }

  function setPreferredNodeId(value: string): void {
    const v = value.trim() || 'hub'
    preferredNodeId.value = v
    localStorage.setItem(PREFERRED_NODE_KEY, v)
  }

  return {
    darkMode,
    themeId,
    backgroundStyle,
    locale,
    preferredNodeId,
    themeMode,
    setDarkMode,
    setThemeId,
    setBackgroundStyle,
    setLocale,
    setPreferredNodeId,
    toggleDarkMode,
  }
})
