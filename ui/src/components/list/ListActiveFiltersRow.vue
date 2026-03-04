<script setup lang="ts">
import PickerActiveChipsRow from '@/components/pickers/PickerActiveChipsRow.vue'
import type { FilterChip } from '@/lib/filterChips'

withDefaults(
  defineProps<{
    chips: FilterChip[]
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
