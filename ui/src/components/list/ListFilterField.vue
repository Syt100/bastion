<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    label: string
    layout?: 'inline' | 'stack'
    controlWidth?: 'auto' | 'narrow' | 'default' | 'wide' | 'full'
  }>(),
  {
    layout: 'stack',
    controlWidth: 'default',
  },
)

const containerClass = computed<string>(() => {
  if (props.layout === 'inline') {
    if (props.controlWidth === 'full') {
      return 'w-full md:w-auto shrink-0 flex items-center gap-2'
    }
    return 'shrink-0 flex items-center gap-2 whitespace-nowrap'
  }
  return 'space-y-2'
})

const controlClass = computed<string>(() => {
  if (props.layout !== 'inline') return 'w-full'

  if (props.controlWidth === 'auto') return ''
  if (props.controlWidth === 'narrow') return 'w-28'
  if (props.controlWidth === 'wide') return 'w-56'
  if (props.controlWidth === 'full') return 'w-full md:w-56 md:flex-none'
  return 'w-40'
})
</script>

<template>
  <div :class="containerClass">
    <div class="app-filter-label">{{ label }}</div>
    <div :class="controlClass">
      <slot />
    </div>
  </div>
</template>
