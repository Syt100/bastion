<script setup lang="ts">
import { NCard, NEmpty } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'

const { t } = useI18n()
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('dashboard.title')" :subtitle="t('dashboard.subtitle')" />

    <n-card class="app-card" :title="t('dashboard.runs7d')">
      <div class="h-64 flex items-center justify-center">
        <n-empty :description="t('dashboard.trendUnavailable')" />
      </div>
    </n-card>
  </div>
</template>
