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

function openDocs(): void {
  const docsRoot = ui.locale === 'zh-CN' ? '/docs/zh/' : '/docs/'
  window.open(docsRoot, '_blank', 'noopener')
}
</script>

<template>
  <div class="min-h-screen relative overflow-hidden">
    <div class="absolute inset-0" style="background-color: var(--app-bg-solid); background-image: var(--app-bg)" />

    <div class="relative min-h-screen flex flex-col">
      <div class="px-6 pt-6">
        <div class="max-w-5xl mx-auto flex items-center justify-between">
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
        <div class="w-full max-w-5xl grid gap-8 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,28rem)] items-center">
          <div class="space-y-6">
            <div class="space-y-3 text-center lg:text-left">
              <div class="text-sm uppercase tracking-[0.18em] app-text-muted">{{ t('app.name') }}</div>
              <h1 class="app-page-title text-[clamp(2rem,1.5rem+1.2vw,3rem)]">{{ t('auth.heroTitle') }}</h1>
              <p class="app-page-subtitle text-base max-w-2xl mx-auto lg:mx-0">{{ t('auth.heroBody') }}</p>
            </div>

            <div class="hidden md:grid gap-3 md:grid-cols-3">
              <div class="rounded-2xl app-panel-inset px-4 py-4">
                <div class="font-medium">{{ t('auth.heroCards.jobsTitle') }}</div>
                <div class="mt-1 text-sm app-text-muted">{{ t('auth.heroCards.jobsBody') }}</div>
              </div>
              <div class="rounded-2xl app-panel-inset px-4 py-4">
                <div class="font-medium">{{ t('auth.heroCards.agentsTitle') }}</div>
                <div class="mt-1 text-sm app-text-muted">{{ t('auth.heroCards.agentsBody') }}</div>
              </div>
              <div class="rounded-2xl app-panel-inset px-4 py-4">
                <div class="font-medium">{{ t('auth.heroCards.restoresTitle') }}</div>
                <div class="mt-1 text-sm app-text-muted">{{ t('auth.heroCards.restoresBody') }}</div>
              </div>
            </div>

            <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-center lg:justify-start">
              <n-button size="small" @click="openDocs">{{ t('auth.docsCta') }}</n-button>
              <div class="text-sm app-text-muted">{{ t('auth.heroHelp') }}</div>
            </div>
          </div>

          <div class="w-full max-w-md lg:ml-auto">
            <slot />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
