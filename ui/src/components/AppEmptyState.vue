<script setup lang="ts">
import { NCard, NSpin } from 'naive-ui'

defineProps<{
  title: string
  description?: string
  loading?: boolean
  variant?: 'card' | 'inset' | 'plain'
}>()
</script>

<template>
  <n-card
    v-if="variant !== 'inset' && variant !== 'plain'"
    class="app-card app-motion-soft"
    :bordered="false"
  >
    <div class="py-10 text-center">
      <n-spin v-if="loading" size="small" />
      <div class="mt-3 font-medium">{{ title }}</div>
      <div v-if="description" class="mt-1 app-help-text">{{ description }}</div>
      <div v-if="$slots.actions" class="mt-4 flex items-center justify-center gap-2">
        <slot name="actions" />
      </div>
    </div>
  </n-card>

  <div
    v-else
      :class="[
        'text-center',
        variant === 'inset' ? 'rounded-xl app-panel-inset app-motion-soft px-4 py-10' : 'py-10',
      ]"
  >
    <div>
      <n-spin v-if="loading" size="small" />
      <div class="mt-3 font-medium">{{ title }}</div>
      <div v-if="description" class="mt-1 app-help-text">{{ description }}</div>
      <div v-if="$slots.actions" class="mt-4 flex items-center justify-center gap-2">
        <slot name="actions" />
      </div>
    </div>
  </div>
</template>
