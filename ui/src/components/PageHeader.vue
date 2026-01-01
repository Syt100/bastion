<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <div class="flex items-start justify-between gap-4 flex-wrap">
    <div class="space-y-1 min-w-[12rem]">
      <h1 class="text-2xl font-semibold tracking-tight">{{ title }}</h1>
      <p v-if="subtitle" class="text-sm opacity-70">{{ subtitle }}</p>
    </div>
    <div class="flex items-center gap-2">
      <slot />
    </div>
  </div>
</template>

