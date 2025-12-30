<script setup lang="ts">
import { computed, watchEffect } from 'vue'
import { darkTheme, dateEnUS, dateZhCN, enUS, NConfigProvider, NGlobalStyle, NMessageProvider, zhCN } from 'naive-ui'

import { useUiStore } from '@/stores/ui'

const ui = useUiStore()

watchEffect(() => {
  document.documentElement.classList.toggle('dark', ui.darkMode)
})

const naiveLocale = computed(() => (ui.locale === 'zh-CN' ? zhCN : enUS))
const naiveDateLocale = computed(() => (ui.locale === 'zh-CN' ? dateZhCN : dateEnUS))
</script>

<template>
  <n-config-provider
    :theme="ui.darkMode ? darkTheme : null"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
  >
    <n-global-style />
    <n-message-provider>
      <router-view />
    </n-message-provider>
  </n-config-provider>
</template>
