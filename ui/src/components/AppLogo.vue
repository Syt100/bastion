<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  size?: 'sm' | 'md'
}>()

const { t } = useI18n()
</script>

<template>
  <div class="flex items-center gap-2 select-none">
    <div class="h-8 w-8 rounded-lg bg-blue-500/15 flex items-center justify-center">
      <svg
        viewBox="0 0 24 24"
        class="h-5 w-5 text-blue-600 dark:text-blue-400"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M12 2l7 4v6c0 5-3 9-7 10-4-1-7-5-7-10V6l7-4z" />
        <path d="M9 12l2 2 4-5" />
      </svg>
    </div>

    <div class="leading-tight">
      <div
        class="font-semibold tracking-[0.06em]"
        :class="size === 'sm' ? 'text-sm' : 'text-base'"
      >
        {{ t('app.name') }}
      </div>
    </div>
  </div>
</template>
