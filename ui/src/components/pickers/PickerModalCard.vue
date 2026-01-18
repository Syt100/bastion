<script setup lang="ts">
import { NModal } from 'naive-ui'
import type { CSSProperties } from 'vue'

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
</script>

<template>
  <n-modal
    :show="show"
    preset="card"
    :style="style"
    :content-style="contentStyle || { display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden', minHeight: 0 }"
    :title="title"
    @update:show="(v) => emit('update:show', v)"
  >
    <slot />
    <template #footer>
      <slot name="footer" />
    </template>
  </n-modal>
</template>
