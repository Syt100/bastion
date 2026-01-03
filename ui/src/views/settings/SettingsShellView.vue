<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'

const { t } = useI18n()
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('settings.title')" :subtitle="t('settings.subtitle')" />
    <router-view />
  </div>
</template>
