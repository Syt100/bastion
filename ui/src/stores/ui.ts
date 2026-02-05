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
const JOBS_WORKSPACE_LAYOUT_KEY = 'bastion.ui.jobsWorkspace.layoutMode'
const JOBS_WORKSPACE_LIST_VIEW_KEY = 'bastion.ui.jobsWorkspace.listView'
const JOBS_WORKSPACE_SPLIT_LIST_WIDTH_KEY = 'bastion.ui.jobsWorkspace.splitListWidthPx'

const JOBS_WORKSPACE_SPLIT_LIST_WIDTH_DEFAULT_PX = 360
const JOBS_WORKSPACE_SPLIT_LIST_WIDTH_MIN_PX = 280
const JOBS_WORKSPACE_SPLIT_LIST_WIDTH_MAX_PX = 640

export type JobsWorkspaceLayoutMode = 'split' | 'list' | 'detail'
export type JobsWorkspaceListView = 'list' | 'table'

function isJobsWorkspaceLayoutMode(value: string | null): value is JobsWorkspaceLayoutMode {
  return value === 'split' || value === 'list' || value === 'detail'
}

function isJobsWorkspaceListView(value: string | null): value is JobsWorkspaceListView {
  return value === 'list' || value === 'table'
}

function clampInt(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.round(value)))
}

function parseStoredInt(value: string | null): number | null {
  if (!value) return null
  const n = Number.parseInt(value, 10)
  return Number.isFinite(n) ? n : null
}

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
  const jobsWorkspaceLayoutMode = ref<JobsWorkspaceLayoutMode>(
    (() => {
      const raw = localStorage.getItem(JOBS_WORKSPACE_LAYOUT_KEY)
      return isJobsWorkspaceLayoutMode(raw) ? raw : 'split'
    })(),
  )
  const jobsWorkspaceListView = ref<JobsWorkspaceListView>(
    (() => {
      const raw = localStorage.getItem(JOBS_WORKSPACE_LIST_VIEW_KEY)
      return isJobsWorkspaceListView(raw) ? raw : 'list'
    })(),
  )
  const jobsWorkspaceSplitListWidthPx = ref<number>(
    (() => {
      const raw = parseStoredInt(localStorage.getItem(JOBS_WORKSPACE_SPLIT_LIST_WIDTH_KEY))
      if (raw == null) return JOBS_WORKSPACE_SPLIT_LIST_WIDTH_DEFAULT_PX
      return clampInt(raw, JOBS_WORKSPACE_SPLIT_LIST_WIDTH_MIN_PX, JOBS_WORKSPACE_SPLIT_LIST_WIDTH_MAX_PX)
    })(),
  )
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

  function setJobsWorkspaceLayoutMode(value: JobsWorkspaceLayoutMode): void {
    jobsWorkspaceLayoutMode.value = value
    localStorage.setItem(JOBS_WORKSPACE_LAYOUT_KEY, value)
  }

  function setJobsWorkspaceListView(value: JobsWorkspaceListView): void {
    jobsWorkspaceListView.value = value
    localStorage.setItem(JOBS_WORKSPACE_LIST_VIEW_KEY, value)
  }

  function setJobsWorkspaceSplitListWidthPx(value: number): void {
    const v = clampInt(value, JOBS_WORKSPACE_SPLIT_LIST_WIDTH_MIN_PX, JOBS_WORKSPACE_SPLIT_LIST_WIDTH_MAX_PX)
    jobsWorkspaceSplitListWidthPx.value = v
    localStorage.setItem(JOBS_WORKSPACE_SPLIT_LIST_WIDTH_KEY, String(v))
  }

  return {
    darkMode,
    themeId,
    backgroundStyle,
    locale,
    preferredNodeId,
    jobsWorkspaceLayoutMode,
    jobsWorkspaceListView,
    jobsWorkspaceSplitListWidthPx,
    themeMode,
    setDarkMode,
    setThemeId,
    setBackgroundStyle,
    setLocale,
    setPreferredNodeId,
    setJobsWorkspaceLayoutMode,
    setJobsWorkspaceListView,
    setJobsWorkspaceSplitListWidthPx,
    toggleDarkMode,
  }
})
