<script setup lang="ts">
import type { CSSProperties } from 'vue'
import AppModalShell from '@/components/AppModalShell.vue'

withDefaults(
  defineProps<{
    show: boolean
    title: string
    style?: CSSProperties
    contentStyle?: CSSProperties
  }>(),
  {
    style: undefined,
    contentStyle: undefined,
  },
)

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
}>()

const DEFAULT_CONTENT_STYLE: CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  height: '100%',
  overflow: 'hidden',
  minHeight: 0,
}
</script>

<template>
  <AppModalShell
    :show="show"
    :title="title"
    :style="style"
    :content-style="contentStyle || DEFAULT_CONTENT_STYLE"
    :scroll-body="false"
    body-class="gap-0"
    @update:show="(v) => emit('update:show', v)"
  >
    <slot />
    <template #footer>
      <slot name="footer" />
    </template>
  </AppModalShell>
</template>
