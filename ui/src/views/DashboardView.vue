<script setup lang="ts">
import { NCard } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import BackupTrendChart from '@/components/BackupTrendChart.vue'
import PageHeader from '@/components/PageHeader.vue'

const { t } = useI18n()
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('dashboard.title')" :subtitle="t('dashboard.subtitle')" />

    <n-card class="shadow-sm border border-black/5 dark:border-white/10" :title="t('dashboard.runs7d')">
      <div class="h-64">
        <BackupTrendChart />
      </div>
    </n-card>
  </div>
</template>
