<script setup lang="ts">
import ListActiveFiltersRow from '@/components/list/ListActiveFiltersRow.vue'
import type { FilterChip } from '@/lib/filterChips'

withDefaults(
  defineProps<{
    resultsLabel?: string
    chips: FilterChip[]
    clearLabel: string
    wrap?: boolean
  }>(),
  {
    resultsLabel: undefined,
    wrap: true,
  },
)

const emit = defineEmits<{
  (e: 'clear'): void
}>()
</script>

<template>
  <div class="app-list-summary">
    <div v-if="resultsLabel" class="app-list-summary__count">{{ resultsLabel }}</div>
    <ListActiveFiltersRow
      :chips="chips"
      :clear-label="clearLabel"
      :wrap="wrap"
      @clear="emit('clear')"
    />
  </div>
</template>
