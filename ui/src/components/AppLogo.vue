<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { ShieldCheckmark } from '@vicons/ionicons5'

defineProps<{
  size?: 'sm' | 'md'
}>()

const { t } = useI18n()
</script>

<template>
  <div class="flex items-center gap-2 select-none">
    <div class="h-8 w-8 rounded-lg app-border-subtle bg-[var(--app-primary-soft)] flex items-center justify-center shrink-0">
      <ShieldCheckmark class="h-5 w-5 text-[var(--app-primary)]" aria-hidden="true" />
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
