<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NDropdown } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import type { SupportedLocale } from '@/i18n'
import AppLogo from '@/components/AppLogo.vue'

const ui = useUiStore()
const { t } = useI18n()

const languageOptions = computed(() => [
  { label: '简体中文', key: 'zh-CN' },
  { label: 'English', key: 'en-US' },
])

function onSelectLanguage(key: string | number): void {
  ui.setLocale(key as SupportedLocale)
}
</script>

<template>
  <div class="min-h-screen relative overflow-hidden">
    <div
      class="absolute inset-0 bg-gradient-to-br from-slate-50 via-white to-blue-50 dark:from-[#0b1220] dark:via-[#0b1220] dark:to-[#0a1a33]"
    />
    <div class="absolute inset-0 opacity-30 dark:opacity-20 bg-[radial-gradient(circle_at_20%_10%,rgba(59,130,246,0.22),transparent_45%),radial-gradient(circle_at_80%_30%,rgba(99,102,241,0.16),transparent_55%)]" />

    <div class="relative min-h-screen flex flex-col">
      <div class="px-6 pt-6">
        <div class="max-w-md mx-auto flex items-center justify-between">
          <AppLogo size="sm" />
          <div class="flex items-center gap-2">
            <n-dropdown :options="languageOptions" trigger="click" @select="onSelectLanguage">
              <n-button quaternary size="small">{{ t('common.language') }}</n-button>
            </n-dropdown>
            <n-button quaternary size="small" @click="ui.toggleDarkMode()">
              {{ ui.darkMode ? t('common.light') : t('common.dark') }}
            </n-button>
          </div>
        </div>
      </div>

      <div class="flex-1 flex items-center justify-center px-6 pb-10">
        <div class="w-full max-w-md">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>
