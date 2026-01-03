<script setup lang="ts">
import { NCard, NSpin } from 'naive-ui'

defineProps<{
  title: string
  description?: string
  loading?: boolean
}>()
</script>

<template>
  <n-card class="app-card" :bordered="false">
    <div class="py-10 text-center">
      <n-spin v-if="loading" size="small" />
      <div class="mt-3 font-medium">{{ title }}</div>
      <div v-if="description" class="mt-1 app-help-text">{{ description }}</div>
      <div v-if="$slots.actions" class="mt-4 flex items-center justify-center gap-2">
        <slot name="actions" />
      </div>
    </div>
  </n-card>
</template>

