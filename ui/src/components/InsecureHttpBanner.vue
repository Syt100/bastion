<script setup lang="ts">
import { NAlert } from 'naive-ui'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <n-alert type="warning" :bordered="false">
    <div class="font-medium">{{ t('warnings.insecureHttpTitle') }}</div>
    <div class="text-sm app-text-muted mt-1">{{ t('warnings.insecureHttpBody') }}</div>
  </n-alert>
</template>
