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
    // Primary: light blue (modern, calm, B2B-friendly)
    primaryColor: '#3b82f6',
    primaryColorHover: '#60a5fa',
    primaryColorPressed: '#2563eb',
    primaryColorSuppl: '#3b82f6',
    borderRadius: '10px',
  },
  Card: {
    borderRadius: '14px',
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
