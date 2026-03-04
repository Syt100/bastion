<script setup lang="ts">
import { NSelect, type SelectOption } from 'naive-ui'
import { computed } from 'vue'

type SelectPrimitive = string | number
type SelectValue = SelectPrimitive | SelectPrimitive[] | null

const props = withDefaults(
  defineProps<{
    value: SelectValue
    options: SelectOption[]
    placeholder?: string
    multiple?: boolean
    clearable?: boolean
    filterable?: boolean
    loading?: boolean
    maxTagCount?: number | 'responsive'
    consistentMenuWidth?: boolean
    width?: 'default' | 'wide'
    size?: 'small' | 'medium' | 'large'
    containerClass?: string
  }>(),
  {
    multiple: false,
    clearable: false,
    filterable: false,
    loading: false,
    width: 'default',
    size: 'small',
  },
)

const emit = defineEmits<{
  'update:value': [value: SelectValue]
}>()

const resolvedContainerClass = computed<string>(() => {
  if (props.containerClass && props.containerClass.trim().length > 0) return props.containerClass
  if (props.width === 'wide') return 'min-w-[14rem] flex-1 md:flex-none md:w-72'
  return 'w-full md:w-56 md:flex-none'
})
</script>

<template>
  <div :class="resolvedContainerClass">
    <n-select
      :value="value"
      :size="size"
      :multiple="multiple"
      :clearable="clearable"
      :filterable="filterable"
      :loading="loading"
      :max-tag-count="maxTagCount"
      :consistent-menu-width="consistentMenuWidth"
      :placeholder="placeholder"
      :options="options"
      class="w-full"
      @update:value="(next: SelectValue) => emit('update:value', next)"
    />
  </div>
</template>
