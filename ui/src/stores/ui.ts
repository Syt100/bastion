import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import {
  ensureLocaleMessages,
  persistLocalePreference,
  resolveInitialLocale,
  type SupportedLocale,
  setI18nLocale,
} from '@/i18n'
import { DEFAULT_UI_THEME_ID, isUiThemeId, type UiThemeId } from '@/theme/presets'
import {
  DEFAULT_UI_BACKGROUND_STYLE,
  isUiBackgroundStyle,
  UI_BACKGROUND_STYLE_KEY,
  type UiBackgroundStyle,
} from '@/theme/background'
import { parseScopeValue, scopeFromNodeId, scopeToNodeId, type ScopeValue } from '@/lib/scope'

const STORAGE_KEY = 'bastion.ui.darkMode'
const PREFERRED_NODE_KEY = 'bastion.ui.preferredNodeId'
const PREFERRED_SCOPE_KEY = 'bastion.ui.preferredScope'
const THEME_KEY = 'bastion.ui.themeId'
const JOBS_WORKSPACE_LAYOUT_KEY = 'bastion.ui.jobsWorkspace.layoutMode'
const JOBS_WORKSPACE_LIST_VIEW_KEY = 'bastion.ui.jobsWorkspace.listView'
const JOBS_WORKSPACE_SPLIT_LIST_WIDTH_KEY = 'bastion.ui.jobsWorkspace.splitListWidthPx'
const JOBS_SAVED_VIEWS_KEY = 'bastion.ui.jobsWorkspace.savedViews'

const JOBS_WORKSPACE_SPLIT_LIST_WIDTH_DEFAULT_PX = 360
const JOBS_WORKSPACE_SPLIT_LIST_WIDTH_MIN_PX = 280
const JOBS_WORKSPACE_SPLIT_LIST_WIDTH_MAX_PX = 640

export type JobsWorkspaceLayoutMode = 'split' | 'list' | 'detail'
export type JobsWorkspaceListView = 'list' | 'table'
export type JobsSavedView = {
  id: string
  name: string
  scope: ScopeValue
  q: string
  status: string
  schedule: string
  includeArchived: boolean
  sort: string
  createdAt: number
  updatedAt: number
}

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

function parseJobsSavedViews(value: string | null): JobsSavedView[] {
  if (!value) return []
  try {
    const parsed = JSON.parse(value) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed
      .filter((item): item is Record<string, unknown> => !!item && typeof item === 'object' && !Array.isArray(item))
      .map((item) => {
        const scope = parseScopeValue(item.scope)
        const id = typeof item.id === 'string' ? item.id.trim() : ''
        const name = typeof item.name === 'string' ? item.name.trim() : ''
        if (!id || !name || !scope) return null
        return {
          id,
          name,
          scope,
          q: typeof item.q === 'string' ? item.q : '',
          status: typeof item.status === 'string' ? item.status : 'all',
          schedule: typeof item.schedule === 'string' ? item.schedule : 'all',
          includeArchived: item.includeArchived === true,
          sort: typeof item.sort === 'string' ? item.sort : 'updated_desc',
          createdAt: typeof item.createdAt === 'number' ? item.createdAt : Date.now(),
          updatedAt: typeof item.updatedAt === 'number' ? item.updatedAt : Date.now(),
        }
      })
      .filter((item): item is JobsSavedView => !!item)
  } catch {
    return []
  }
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
  const storedPreferredNodeId = localStorage.getItem(PREFERRED_NODE_KEY)
  const preferredNodeId = ref<string>(storedPreferredNodeId || 'hub')
  const preferredScope = ref<ScopeValue>(
    (() => {
      const storedScope = parseScopeValue(localStorage.getItem(PREFERRED_SCOPE_KEY))
      if (storedScope) return storedScope
      if (storedPreferredNodeId) return scopeFromNodeId(storedPreferredNodeId)
      return 'all'
    })(),
  )
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
  const jobsSavedViews = ref<JobsSavedView[]>(parseJobsSavedViews(localStorage.getItem(JOBS_SAVED_VIEWS_KEY)))
  const themeMode = computed(() => (darkMode.value ? 'dark' : 'light'))
  let localeSwitchSeq = 0

  // Ensure i18n and docs entrypoint use the same initial locale preference.
  persistLocalePreference(locale.value)
  void ensureLocaleMessages(locale.value).catch(() => {
    // Keep UI responsive; missing locale bundle falls back to translation keys.
  })
  setI18nLocale(locale.value)

  function setDarkMode(value: boolean): void {
    darkMode.value = value
    localStorage.setItem(STORAGE_KEY, String(value))
  }

  function setLocale(value: SupportedLocale): void {
    locale.value = value
    persistLocalePreference(value)
    const seq = ++localeSwitchSeq
    void ensureLocaleMessages(value)
      .then(() => {
        if (seq !== localeSwitchSeq) return
        setI18nLocale(value)
      })
      .catch(() => {
        // Keep current locale if loading fails.
      })
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

  function persistPreferredNodeId(value: string): void {
    preferredNodeId.value = value
    localStorage.setItem(PREFERRED_NODE_KEY, value)
  }

  function persistPreferredScope(value: ScopeValue): void {
    preferredScope.value = value
    localStorage.setItem(PREFERRED_SCOPE_KEY, value)
  }

  function setPreferredNodeId(value: string): void {
    const v = value.trim() || 'hub'
    persistPreferredNodeId(v)
    persistPreferredScope(scopeFromNodeId(v))
  }

  function setPreferredScope(value: ScopeValue): void {
    persistPreferredScope(value)
    const nextNodeId = scopeToNodeId(value)
    if (nextNodeId) {
      persistPreferredNodeId(nextNodeId)
    }
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

  function persistJobsSavedViews(): void {
    localStorage.setItem(JOBS_SAVED_VIEWS_KEY, JSON.stringify(jobsSavedViews.value))
  }

  function upsertJobsSavedView(
    value: Omit<JobsSavedView, 'createdAt' | 'updatedAt'> & Partial<Pick<JobsSavedView, 'createdAt' | 'updatedAt'>>,
  ): void {
    const now = Date.now()
    const existingIndex = jobsSavedViews.value.findIndex((item) => item.id === value.id)
    const next: JobsSavedView = {
      id: value.id,
      name: value.name.trim(),
      scope: value.scope,
      q: value.q,
      status: value.status,
      schedule: value.schedule,
      includeArchived: value.includeArchived,
      sort: value.sort,
      createdAt: value.createdAt ?? now,
      updatedAt: value.updatedAt ?? now,
    }
    if (existingIndex >= 0) {
      const existing = jobsSavedViews.value[existingIndex]
      jobsSavedViews.value.splice(existingIndex, 1, {
        ...next,
        createdAt: existing?.createdAt ?? next.createdAt,
        updatedAt: now,
      })
    } else {
      jobsSavedViews.value = [...jobsSavedViews.value, next]
    }
    persistJobsSavedViews()
  }

  function deleteJobsSavedView(id: string): void {
    jobsSavedViews.value = jobsSavedViews.value.filter((item) => item.id !== id)
    persistJobsSavedViews()
  }

  return {
    darkMode,
    themeId,
    backgroundStyle,
    locale,
    preferredNodeId,
    preferredScope,
    jobsWorkspaceLayoutMode,
    jobsWorkspaceListView,
    jobsWorkspaceSplitListWidthPx,
    jobsSavedViews,
    themeMode,
    setDarkMode,
    setThemeId,
    setBackgroundStyle,
    setLocale,
    setPreferredNodeId,
    setPreferredScope,
    setJobsWorkspaceLayoutMode,
    setJobsWorkspaceListView,
    setJobsWorkspaceSplitListWidthPx,
    upsertJobsSavedView,
    deleteJobsSavedView,
    toggleDarkMode,
  }
})
