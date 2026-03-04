<script setup lang="ts">
import { NIcon } from 'naive-ui'
import { computed, type Component } from 'vue'

const SIZE_MAP = {
  xs: 14,
  sm: 16,
  md: 18,
  lg: 20,
} as const

type IconSize = keyof typeof SIZE_MAP
type IconTone = 'default' | 'muted' | 'primary' | 'success' | 'warning' | 'danger'

const props = withDefaults(
  defineProps<{
    component?: Component
    size?: IconSize
    tone?: IconTone
  }>(),
  {
    component: undefined,
    size: 'sm',
    tone: 'default',
  },
)

const resolvedSize = computed<number>(() => SIZE_MAP[props.size])
const toneClass = computed<string>(() => `app-icon-tone-${props.tone}`)
</script>

<template>
  <n-icon :size="resolvedSize" :component="component" class="app-icon" :class="toneClass">
    <slot />
  </n-icon>
</template>
