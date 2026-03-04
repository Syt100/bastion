<script setup lang="ts">
import { computed } from 'vue'
import AppEmptyState from '@/components/AppEmptyState.vue'

type EmptyVariant = 'card' | 'inset' | 'plain'

type ListState = 'ready' | 'loading' | 'base-empty' | 'filtered-empty'

const props = withDefaults(
  defineProps<{
    loading: boolean
    itemCount: number
    baseEmpty: boolean
    loadingTitle: string
    loadingDescription?: string
    baseEmptyTitle: string
    baseEmptyDescription?: string
    filteredEmptyTitle: string
    filteredEmptyDescription?: string
    variant?: EmptyVariant
  }>(),
  {
    loadingDescription: undefined,
    baseEmptyDescription: undefined,
    filteredEmptyDescription: undefined,
    variant: 'card',
  },
)

const state = computed<ListState>(() => {
  if (props.loading && props.itemCount === 0) return 'loading'
  if (!props.loading && props.itemCount === 0 && props.baseEmpty) return 'base-empty'
  if (!props.loading && props.itemCount === 0) return 'filtered-empty'
  return 'ready'
})
</script>

<template>
  <AppEmptyState
    v-if="state === 'loading'"
    :title="loadingTitle"
    :description="loadingDescription"
    :variant="variant"
    loading
  />

  <AppEmptyState
    v-else-if="state === 'base-empty'"
    :title="baseEmptyTitle"
    :description="baseEmptyDescription"
    :variant="variant"
  >
    <template #actions>
      <slot name="baseActions" />
    </template>
  </AppEmptyState>

  <AppEmptyState
    v-else-if="state === 'filtered-empty'"
    :title="filteredEmptyTitle"
    :description="filteredEmptyDescription"
    :variant="variant"
  >
    <template #actions>
      <slot name="filteredActions" />
    </template>
  </AppEmptyState>

  <slot v-else />
</template>
