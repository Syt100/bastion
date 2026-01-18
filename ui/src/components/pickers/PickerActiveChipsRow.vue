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
}>()

const emit = defineEmits<{
  (e: 'clear'): void
}>()
</script>

<template>
  <div v-if="chips.length > 0" class="flex flex-wrap gap-2 items-center">
    <n-tag v-for="chip in chips" :key="chip.key" size="small" closable @close="chip.onClose">
      {{ chip.label }}
    </n-tag>
    <n-button size="tiny" tertiary @click="emit('clear')">{{ clearLabel }}</n-button>
  </div>
</template>

