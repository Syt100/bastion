<script setup lang="ts">
import { NAlert } from 'naive-ui'
import { useI18n } from 'vue-i18n'

defineProps<{ title: string }>()
const { t } = useI18n()
</script>

<template>
  <div class="space-y-2">
    <h1 class="text-xl font-semibold">{{ title }}</h1>
    <n-alert type="info" :bordered="false">
      {{ t('placeholder.comingSoon') }}
    </n-alert>
  </div>
</template>
