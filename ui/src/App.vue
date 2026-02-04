<script setup lang="ts">
import { computed, onMounted, watchEffect } from 'vue'
import { useRoute } from 'vue-router'
import {
  darkTheme,
  NConfigProvider,
  NGlobalStyle,
  NMessageProvider,
  type GlobalThemeOverrides,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import { useSystemStore } from '@/stores/system'
import { NAIVE_UI_DATE_LOCALES, NAIVE_UI_LOCALES } from '@/i18n/language'
import { UI_BACKGROUND_NEUTRAL_COLORS } from '@/theme/background'

const ui = useUiStore()
const system = useSystemStore()
const route = useRoute()
const { t } = useI18n()

const appName = computed(() => t('app.name'))
const pageTitle = computed(() => {
  const titleKey = [...route.matched]
    .reverse()
    .find((r) => typeof r.meta.titleKey === 'string')?.meta.titleKey as string | undefined

  if (!titleKey) return appName.value
  const localized = t(titleKey)
  if (!localized || localized === titleKey) return appName.value
  if (localized === appName.value) return appName.value
  return `${localized} · ${appName.value}`
})

watchEffect(() => {
  if (typeof document === 'undefined') return
  document.documentElement.dataset.theme = ui.themeId
  document.documentElement.dataset.bg = ui.backgroundStyle
  document.documentElement.classList.toggle('dark', ui.darkMode)
  // Apply to <body> as well so CSS variables used by
  // `body { background-color: var(--app-bg-solid); background-image: var(--app-bg) }`
  // always resolve correctly (some themes/libraries may style <html> separately).
  document.body?.classList.toggle('dark', ui.darkMode)
  document.documentElement.lang = ui.locale
  document.title = pageTitle.value

  // Use solid background in dark mode; use primary accent in light mode.
  // Plain background uses the solid base in both modes to avoid tinted browser chrome.
  const styles = getComputedStyle(document.documentElement)
  const mode = ui.darkMode ? 'dark' : 'light'
  const bgSolid = styles.getPropertyValue('--app-bg-solid').trim()
  const primary = styles.getPropertyValue('--app-primary').trim()
  const fallback = ui.backgroundStyle === 'plain' ? UI_BACKGROUND_NEUTRAL_COLORS[mode] : ui.darkMode ? '#040b0b' : '#0d9488'
  const themeColor = (ui.backgroundStyle === 'plain' ? bgSolid : ui.darkMode ? bgSolid : primary) || fallback
  const themeMeta = document.querySelector('meta[name="theme-color"]')
  if (themeMeta) {
    themeMeta.setAttribute('content', themeColor)
  }
})

const naiveLocale = computed(() => NAIVE_UI_LOCALES[ui.locale])
const naiveDateLocale = computed(() => NAIVE_UI_DATE_LOCALES[ui.locale])

const FALLBACK_TOKENS_LIGHT = {
  primary: '#0d9488',
  primaryHover: '#14b8a6',
  primaryPressed: '#0f766e',
  primarySoft: 'rgba(20, 184, 166, 0.14)',

  info: '#0ea5e9',
  success: '#22c55e',
  warning: '#f59e0b',
  danger: '#ef4444',

  text: '#0f172a',
  textMuted: 'rgba(15, 23, 42, 0.72)',
  border: 'rgba(15, 23, 42, 0.09)',
  hover: 'rgba(15, 23, 42, 0.04)',
  pressed: 'rgba(15, 23, 42, 0.06)',

  surface: '#ffffff',
  surface2: '#f1f5f9',

  shadowSm: '0 1px 2px rgba(15, 23, 42, 0.05), 0 1px 3px rgba(15, 23, 42, 0.08)',
  shadowMd: '0 10px 24px rgba(15, 23, 42, 0.10)',
  shadowLg: '0 18px 50px rgba(15, 23, 42, 0.14)',
} as const

const FALLBACK_TOKENS_DARK = {
  primary: '#2dd4bf',
  primaryHover: '#5eead4',
  primaryPressed: '#14b8a6',
  primarySoft: 'rgba(45, 212, 191, 0.20)',

  info: '#0ea5e9',
  success: '#22c55e',
  warning: '#f59e0b',
  danger: '#ef4444',

  text: '#f8fafc',
  textMuted: 'rgba(248, 250, 252, 0.74)',
  border: 'rgba(248, 250, 252, 0.14)',
  hover: 'rgba(255, 255, 255, 0.06)',
  pressed: 'rgba(255, 255, 255, 0.08)',

  surface: '#0b1515',
  surface2: 'rgba(16, 32, 32, 0.72)',

  shadowSm: '0 1px 2px rgba(0, 0, 0, 0.28), 0 2px 8px rgba(0, 0, 0, 0.22)',
  shadowMd: '0 12px 28px rgba(0, 0, 0, 0.34)',
  shadowLg: '0 20px 60px rgba(0, 0, 0, 0.42)',
} as const

function cssVar(name: string, fallback: string): string {
  // Naive UI theme overrides are parsed by seemly and must be real color strings (not `var(...)`).
  if (typeof document === 'undefined') return fallback
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  if (!v) return fallback
  if (v.includes('var(')) return fallback
  return v
}

const resolvedTokens = computed(() => {
  // Ensure theme overrides recompute when the theme preset changes.
  void ui.themeId
  const fb = ui.darkMode ? FALLBACK_TOKENS_DARK : FALLBACK_TOKENS_LIGHT
  return {
    primary: cssVar('--app-primary', fb.primary),
    primaryHover: cssVar('--app-primary-hover', fb.primaryHover),
    primaryPressed: cssVar('--app-primary-pressed', fb.primaryPressed),
    primarySoft: cssVar('--app-primary-soft', fb.primarySoft),

    info: cssVar('--app-info', fb.info),
    success: cssVar('--app-success', fb.success),
    warning: cssVar('--app-warning', fb.warning),
    danger: cssVar('--app-danger', fb.danger),

    text: cssVar('--app-text', fb.text),
    textMuted: cssVar('--app-text-muted', fb.textMuted),
    border: cssVar('--app-border', fb.border),
    hover: cssVar('--app-hover', fb.hover),
    pressed: cssVar('--app-pressed', fb.pressed),

    surface: cssVar('--app-surface', fb.surface),
    surface2: cssVar('--app-surface-2', fb.surface2),

    shadowSm: cssVar('--app-shadow-sm', fb.shadowSm),
    shadowMd: cssVar('--app-shadow-md', fb.shadowMd),
    shadowLg: cssVar('--app-shadow-lg', fb.shadowLg),
  }
})

const themeOverrides = computed<GlobalThemeOverrides>(() => ({
  common: {
    ...(() => {
      const c = resolvedTokens.value
      return {
        primaryColor: c.primary,
        primaryColorHover: c.primaryHover,
        primaryColorPressed: c.primaryPressed,
        primaryColorSuppl: c.primary,

        infoColor: c.info,
        infoColorHover: c.info,
        infoColorPressed: c.info,
        infoColorSuppl: c.info,

        successColor: c.success,
        successColorHover: c.success,
        successColorPressed: c.success,
        successColorSuppl: c.success,

        warningColor: c.warning,
        warningColorHover: c.warning,
        warningColorPressed: c.warning,
        warningColorSuppl: c.warning,

        errorColor: c.danger,
        errorColorHover: c.danger,
        errorColorPressed: c.danger,
        errorColorSuppl: c.danger,

        textColorBase: c.text,
        textColor1: c.text,
        textColor2: c.textMuted,
        textColor3: c.textMuted,
        placeholderColor: c.textMuted,

        dividerColor: c.border,
        borderColor: c.border,
        hoverColor: c.hover,
        pressedColor: c.pressed,

        // Let the body show the gradient background from
        // `body { background-color: var(--app-bg-solid); background-image: var(--app-bg) }`.
        bodyColor: 'transparent',

        // Make surface colors consistent across overlays.
        cardColor: c.surface,
        popoverColor: c.surface,
        modalColor: c.surface,
        inputColor: c.surface,

        // Tables should generally blend into the surrounding card.
        tableColor: 'transparent',
        tableHeaderColor: c.surface2,
        tableColorHover: c.hover,

        boxShadow1: c.shadowSm,
        boxShadow2: c.shadowMd,
        boxShadow3: c.shadowLg,
      }
    })(),

    borderRadius: '12px',
    borderRadiusSmall: '8px',
  },
  Card: {
    borderRadius: '16px',
    borderColor: resolvedTokens.value.border,
  },
  Menu: {
    color: 'transparent',
    groupTextColor: resolvedTokens.value.textMuted,
    dividerColor: resolvedTokens.value.border,

    itemColorHover: resolvedTokens.value.hover,
    itemColorActive: resolvedTokens.value.primarySoft,
    itemColorActiveHover: resolvedTokens.value.primarySoft,

    itemTextColor: resolvedTokens.value.text,
    itemTextColorHover: resolvedTokens.value.text,
    itemTextColorActive: resolvedTokens.value.primary,
    itemTextColorActiveHover: resolvedTokens.value.primary,

    itemIconColor: resolvedTokens.value.textMuted,
    itemIconColorHover: resolvedTokens.value.text,
    itemIconColorActive: resolvedTokens.value.primary,
    itemIconColorActiveHover: resolvedTokens.value.primary,
  },
}))

onMounted(async () => {
  try {
    await system.refresh()
  } catch {
    // ignore
  }
})
</script>

<template>
  <n-config-provider
    :theme="ui.darkMode ? darkTheme : null"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
    :theme-overrides="themeOverrides"
  >
    <n-global-style />
    <n-message-provider>
      <router-view />
    </n-message-provider>
  </n-config-provider>
</template>
