<script setup lang="ts">
import PickerActiveChipsRow, { type PickerActiveChip } from '@/components/pickers/PickerActiveChipsRow.vue'

withDefaults(
  defineProps<{
    chips: PickerActiveChip[]
    clearLabel: string
    wrap?: boolean
  }>(),
  {
    wrap: true,
  },
)

const emit = defineEmits<{
  (e: 'clear'): void
}>()
</script>

<template>
  <PickerActiveChipsRow :chips="chips" :clear-label="clearLabel" :wrap="wrap" @clear="emit('clear')" />
</template>
