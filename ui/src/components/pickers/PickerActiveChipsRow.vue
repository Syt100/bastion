<script setup lang="ts">
import { NButton, NTag } from 'naive-ui'

export type PickerActiveChip = {
  key: string
  label: string
  onClose: () => void
}

defineProps<{
  chips: PickerActiveChip[]
  clearLabel: string
  /**
   * When false, chips are rendered on a single line with horizontal scroll.
   * Useful in narrow panes (e.g. split views) where wrapping would consume vertical space.
   */
  wrap?: boolean
}>()

const emit = defineEmits<{
  (e: 'clear'): void
}>()
</script>

<template>
  <div
    v-if="chips.length > 0"
    :class="
      wrap === false
        ? 'flex gap-2 items-center overflow-x-auto whitespace-nowrap'
        : 'flex flex-wrap gap-2 items-center'
    "
  >
    <n-tag v-for="chip in chips" :key="chip.key" size="small" class="shrink-0" closable @close="chip.onClose">
      {{ chip.label }}
    </n-tag>
    <n-button size="tiny" tertiary class="shrink-0" @click="emit('clear')">{{ clearLabel }}</n-button>
  </div>
</template>
