<script setup lang="ts">
import { NButton } from 'naive-ui'

withDefaults(
  defineProps<{
    ariaLabel: string
    size?: 'tiny' | 'small' | 'medium' | 'large'
    circle?: boolean
    quaternary?: boolean
  }>(),
  {
    size: 'tiny',
    circle: true,
    quaternary: true,
  },
)
</script>

<template>
  <n-button :size="size" :circle="circle" :quaternary="quaternary" :aria-label="ariaLabel">
    <slot />
    <template v-if="$slots.icon" #icon>
      <slot name="icon" />
    </template>
  </n-button>
</template>
