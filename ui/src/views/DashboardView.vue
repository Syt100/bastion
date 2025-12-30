<script setup lang="ts">
import { NCard } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import BackupTrendChart from '@/components/BackupTrendChart.vue'

const { t } = useI18n()
</script>

<template>
  <div class="space-y-4">
    <div>
      <h1 class="text-xl font-semibold">{{ t('dashboard.title') }}</h1>
      <p class="text-sm opacity-70">{{ t('dashboard.subtitle') }}</p>
    </div>

    <n-card :title="t('dashboard.runs7d')">
      <div class="h-64">
        <BackupTrendChart />
      </div>
    </n-card>
  </div>
</template>
