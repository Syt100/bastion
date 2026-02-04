<script setup lang="ts">
import { computed } from 'vue'
import { NCard, NRadioButton, NRadioGroup, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { UI_THEME_PRESETS } from '@/theme/presets'
import { useUiStore } from '@/stores/ui'

const { t } = useI18n()
const ui = useUiStore()

const themes = computed(() => UI_THEME_PRESETS)

function selectTheme(id: (typeof UI_THEME_PRESETS)[number]['id']): void {
  ui.setThemeId(id)
}

function selectBackgroundStyle(value: string | number): void {
  if (value === 'aurora' || value === 'solid' || value === 'plain') {
    ui.setBackgroundStyle(value)
  }
}
</script>

<template>
  <n-card class="app-card" :bordered="false">
    <div class="flex items-start justify-between gap-4">
      <div class="min-w-0">
        <div class="text-base font-semibold">{{ t('settings.appearance.title') }}</div>
        <div class="text-sm app-text-muted mt-1">{{ t('settings.appearance.subtitle') }}</div>
      </div>
    </div>

    <div class="mt-4 space-y-2">
      <div class="text-sm font-medium">{{ t('settings.appearance.background.title') }}</div>
      <div class="text-sm app-text-muted">{{ t('settings.appearance.background.subtitle') }}</div>
      <n-radio-group size="small" :value="ui.backgroundStyle" @update:value="selectBackgroundStyle">
        <n-radio-button value="aurora">{{ t('settings.appearance.background.styles.aurora') }}</n-radio-button>
        <n-radio-button value="solid">{{ t('settings.appearance.background.styles.solid') }}</n-radio-button>
        <n-radio-button value="plain">{{ t('settings.appearance.background.styles.plain') }}</n-radio-button>
      </n-radio-group>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 mt-4">
      <button
        v-for="theme in themes"
        :key="theme.id"
        type="button"
        class="text-left rounded-xl p-3 app-border-subtle transition"
        :class="
          theme.id === ui.themeId
            ? 'ring-2 ring-[var(--app-primary)] bg-[var(--app-primary-soft)]'
            : 'hover:bg-[var(--app-hover)]'
        "
        @click="selectTheme(theme.id)"
      >
        <div class="flex items-center justify-between gap-3">
          <div class="font-medium truncate">{{ t(theme.titleKey) }}</div>
          <n-tag v-if="theme.isDefault" size="small" round :bordered="false">
            {{ t('settings.appearance.default') }}
          </n-tag>
        </div>

        <!-- Preview: use the same theme tokens via data-theme, so the swatch stays accurate. -->
        <div
          class="mt-2 rounded-lg app-border-subtle overflow-hidden"
          :class="ui.darkMode ? 'dark' : ''"
          :data-theme="theme.id"
          :data-bg="ui.backgroundStyle"
        >
          <div class="h-12" style="background-color: var(--app-bg-solid); background-image: var(--app-bg)"></div>
          <div class="flex items-center justify-between px-2 py-2" style="background-color: var(--app-surface)">
            <div class="flex items-center gap-2 min-w-0">
              <span
                class="inline-block w-3 h-3 rounded-full"
                style="background-color: var(--app-primary)"
              ></span>
              <span
                class="inline-block w-3 h-3 rounded-full"
                style="background-color: var(--app-primary-2)"
              ></span>
              <span class="text-xs truncate" style="color: var(--app-text-muted)">{{ ui.darkMode ? t('common.dark') : t('common.light') }}</span>
            </div>
            <span class="text-xs font-medium" style="color: var(--app-text)">{{ t('common.preview') }}</span>
          </div>
        </div>
      </button>
    </div>
  </n-card>
</template>
