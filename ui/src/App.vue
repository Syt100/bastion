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
  document.documentElement.classList.toggle('dark', ui.darkMode)
  document.documentElement.lang = ui.locale
  document.title = pageTitle.value

  const themeColor = ui.darkMode ? '#0b1220' : '#3b82f6'
  const themeMeta = document.querySelector('meta[name="theme-color"]')
  if (themeMeta) {
    themeMeta.setAttribute('content', themeColor)
  }
})

const naiveLocale = computed(() => NAIVE_UI_LOCALES[ui.locale])
const naiveDateLocale = computed(() => NAIVE_UI_DATE_LOCALES[ui.locale])

const themeOverrides = computed<GlobalThemeOverrides>(() => ({
  common: {
    // Tokens are defined in `ui/src/styles/main.css` and switch with `.dark`.
    primaryColor: 'var(--app-primary)',
    primaryColorHover: 'var(--app-primary-hover)',
    primaryColorPressed: 'var(--app-primary-pressed)',
    primaryColorSuppl: 'var(--app-primary)',

    infoColor: 'var(--app-info)',
    infoColorHover: 'var(--app-info)',
    infoColorPressed: 'var(--app-info)',
    infoColorSuppl: 'var(--app-info)',

    successColor: 'var(--app-success)',
    successColorHover: 'var(--app-success)',
    successColorPressed: 'var(--app-success)',
    successColorSuppl: 'var(--app-success)',

    warningColor: 'var(--app-warning)',
    warningColorHover: 'var(--app-warning)',
    warningColorPressed: 'var(--app-warning)',
    warningColorSuppl: 'var(--app-warning)',

    errorColor: 'var(--app-danger)',
    errorColorHover: 'var(--app-danger)',
    errorColorPressed: 'var(--app-danger)',
    errorColorSuppl: 'var(--app-danger)',

    textColorBase: 'var(--app-text)',
    textColor1: 'var(--app-text)',
    textColor2: 'var(--app-text-muted)',
    textColor3: 'var(--app-text-muted)',
    placeholderColor: 'var(--app-text-muted)',

    dividerColor: 'var(--app-border)',
    borderColor: 'var(--app-border)',
    hoverColor: 'var(--app-hover)',
    pressedColor: 'var(--app-pressed)',

    // Let the body show the gradient background from `body { background: var(--app-bg) }`.
    bodyColor: 'transparent',

    // Make surface colors consistent across overlays.
    cardColor: 'var(--app-surface)',
    popoverColor: 'var(--app-surface)',
    modalColor: 'var(--app-surface)',
    inputColor: 'var(--app-surface)',

    // Tables should generally blend into the surrounding card.
    tableColor: 'transparent',
    tableHeaderColor: 'var(--app-surface-2)',
    tableColorHover: 'var(--app-hover)',

    boxShadow1: 'var(--app-shadow-sm)',
    boxShadow2: 'var(--app-shadow-md)',
    boxShadow3: 'var(--app-shadow-lg)',

    borderRadius: '12px',
    borderRadiusSmall: '10px',
  },
  Card: {
    borderRadius: '16px',
    borderColor: 'var(--app-border)',
  },
  Menu: {
    color: 'transparent',
    groupTextColor: 'var(--app-text-muted)',
    dividerColor: 'var(--app-border)',

    itemColorHover: 'var(--app-hover)',
    itemColorActive: 'var(--app-primary-soft)',
    itemColorActiveHover: 'var(--app-primary-soft)',

    itemTextColor: 'var(--app-text)',
    itemTextColorHover: 'var(--app-text)',
    itemTextColorActive: 'var(--app-primary)',
    itemTextColorActiveHover: 'var(--app-primary)',

    itemIconColor: 'var(--app-text-muted)',
    itemIconColorHover: 'var(--app-text)',
    itemIconColorActive: 'var(--app-primary)',
    itemIconColorActiveHover: 'var(--app-primary)',
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
