<script setup lang="ts">
import { computed } from 'vue'
import { NPagination } from 'naive-ui'
import { LIST_PAGE_SIZE_OPTIONS } from '@/lib/listUi'

const props = withDefaults(
  defineProps<{
    page: number
    pageSize: number
    itemCount: number
    pageSizes?: number[]
    loading?: boolean
    totalLabel?: string
  }>(),
  {
    pageSizes: () => [...LIST_PAGE_SIZE_OPTIONS],
    loading: false,
    totalLabel: undefined,
  },
)

const emit = defineEmits<{
  'update:page': [value: number]
  'update:pageSize': [value: number]
}>()

const hasTotalLabel = computed(() => typeof props.totalLabel === 'string' && props.totalLabel.trim().length > 0)
</script>

<template>
  <div
    class="mt-4 flex items-center gap-3.5 app-motion-soft"
    :class="hasTotalLabel ? 'justify-between' : 'justify-end'"
  >
    <div v-if="hasTotalLabel" class="app-meta-text">{{ totalLabel }}</div>

    <n-pagination
      :page="page"
      :page-size="pageSize"
      :item-count="itemCount"
      :page-sizes="pageSizes"
      :disabled="loading"
      show-size-picker
      size="small"
      @update:page="(value) => emit('update:page', value)"
      @update:page-size="(value) => emit('update:pageSize', value)"
    />
  </div>
</template>
