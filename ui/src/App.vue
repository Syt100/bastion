<script setup lang="ts">
import { computed, onMounted, watchEffect } from 'vue'
import {
  darkTheme,
  dateEnUS,
  dateZhCN,
  enUS,
  NConfigProvider,
  NGlobalStyle,
  NMessageProvider,
  zhCN,
  type GlobalThemeOverrides,
} from 'naive-ui'

import { useUiStore } from '@/stores/ui'
import { useSystemStore } from '@/stores/system'

const ui = useUiStore()
const system = useSystemStore()

watchEffect(() => {
  if (typeof document === 'undefined') return
  document.documentElement.classList.toggle('dark', ui.darkMode)
  document.documentElement.lang = ui.locale
})

const naiveLocale = computed(() => (ui.locale === 'zh-CN' ? zhCN : enUS))
const naiveDateLocale = computed(() => (ui.locale === 'zh-CN' ? dateZhCN : dateEnUS))

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
