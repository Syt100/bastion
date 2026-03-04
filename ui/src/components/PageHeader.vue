<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <div class="flex items-start justify-between gap-4 flex-wrap">
    <div class="flex items-start gap-3 min-w-[12rem]">
      <div v-if="$slots.prefix" class="pt-0.5">
        <slot name="prefix" />
      </div>
      <div class="space-y-1.5 min-w-0">
        <div class="flex items-center gap-2 flex-wrap">
          <h1 class="app-page-title">{{ title }}</h1>
          <slot name="titleSuffix" />
        </div>
        <div v-if="$slots.subtitle" class="min-w-0">
          <slot name="subtitle" />
        </div>
        <p v-else-if="subtitle" class="app-page-subtitle">{{ subtitle }}</p>
      </div>
    </div>
    <!-- Keep actions aligned to the right even when they wrap below the title. -->
    <div class="flex items-center gap-2 flex-wrap justify-end ml-auto max-w-full app-motion-soft">
      <slot />
    </div>
  </div>
</template>
