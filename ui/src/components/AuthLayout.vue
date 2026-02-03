<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NDropdown } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import type { SupportedLocale } from '@/i18n'
import AppLogo from '@/components/AppLogo.vue'
import { getLocaleDropdownOptions } from '@/i18n/language'

const ui = useUiStore()
const { t } = useI18n()

const languageOptions = computed(() => getLocaleDropdownOptions())

function onSelectLanguage(key: string | number): void {
  ui.setLocale(key as SupportedLocale)
}
</script>

<template>
  <div class="min-h-screen relative overflow-hidden">
    <div class="absolute inset-0" style="background-color: var(--app-bg-solid); background-image: var(--app-bg)" />

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
