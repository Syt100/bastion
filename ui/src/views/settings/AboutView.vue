<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { NCard } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useSystemStore } from '@/stores/system'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { UI_BUILD_INFO } from '@/lib/buildInfo'

const { t } = useI18n()

const ui = useUiStore()
const system = useSystemStore()

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const hubVersion = computed(() => system.version ?? '-')
const hubBuildTime = computed(() => formatUnixSeconds(system.buildTimeUnix))
const uiVersion = computed(() => UI_BUILD_INFO.version || '-')
const uiBuildTime = computed(() => formatUnixSeconds(UI_BUILD_INFO.buildTimeUnix))

onMounted(() => {
  void system.refresh()
})
</script>

<template>
  <div class="space-y-6">
    <n-card class="app-card" :title="t('settings.about.hubTitle')" :bordered="false">
      <div class="app-divide-y">
        <div class="px-3 py-3 flex items-center justify-between gap-3">
          <div class="font-medium">{{ t('settings.about.fields.version') }}</div>
          <div class="font-mono text-xs app-text-muted truncate">{{ hubVersion }}</div>
        </div>
        <div class="px-3 py-3 flex items-center justify-between gap-3">
          <div class="font-medium">{{ t('settings.about.fields.buildTime') }}</div>
          <div class="font-mono text-xs app-text-muted truncate">{{ hubBuildTime }}</div>
        </div>
      </div>
    </n-card>

    <n-card class="app-card" :title="t('settings.about.uiTitle')" :bordered="false">
      <div class="app-divide-y">
        <div class="px-3 py-3 flex items-center justify-between gap-3">
          <div class="font-medium">{{ t('settings.about.fields.version') }}</div>
          <div class="font-mono text-xs app-text-muted truncate">{{ uiVersion }}</div>
        </div>
        <div class="px-3 py-3 flex items-center justify-between gap-3">
          <div class="font-medium">{{ t('settings.about.fields.buildTime') }}</div>
          <div class="font-mono text-xs app-text-muted truncate">{{ uiBuildTime }}</div>
        </div>
      </div>
    </n-card>
  </div>
</template>
