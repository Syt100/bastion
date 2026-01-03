<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import { NCard } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'

const { t } = useI18n()

const BackupTrendChart = defineAsyncComponent(() => import('@/components/BackupTrendChart.vue'))
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('dashboard.title')" :subtitle="t('dashboard.subtitle')" />

    <n-card class="app-card" :title="t('dashboard.runs7d')">
      <div class="h-64">
        <Suspense>
          <template #default>
            <BackupTrendChart />
          </template>
          <template #fallback>
            <div class="h-full w-full rounded-lg bg-black/5 dark:bg-white/10 animate-pulse" />
          </template>
        </Suspense>
      </div>
    </n-card>
  </div>
</template>
