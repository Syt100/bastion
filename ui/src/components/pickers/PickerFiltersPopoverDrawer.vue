<script setup lang="ts">
import { NBadge, NButton, NDrawer, NDrawerContent, NPopover } from 'naive-ui'
import { FilterOutline } from '@vicons/ionicons5'
import AppIcon from '@/components/AppIcon.vue'

withDefaults(
  defineProps<{
    isDesktop: boolean
    title: string
    activeCount: number
    widthClass?: string
    popoverOpen: boolean
    drawerOpen: boolean
  }>(),
  {
    widthClass: 'w-80',
  },
)

const emit = defineEmits<{
  (e: 'update:popoverOpen', value: boolean): void
  (e: 'update:drawerOpen', value: boolean): void
}>()
</script>

<template>
  <n-popover
    v-if="isDesktop"
    :show="popoverOpen"
    trigger="click"
    placement="bottom-end"
    :show-arrow="false"
    @update:show="(v) => emit('update:popoverOpen', Boolean(v))"
  >
    <template #trigger>
      <n-badge :value="activeCount" :show="activeCount > 0">
        <n-button size="small" secondary class="app-motion-soft app-pressable" :title="title" :aria-label="title">
          <template #icon>
            <AppIcon :component="FilterOutline" size="sm" tone="muted" />
          </template>
        </n-button>
      </n-badge>
    </template>

    <div :class="widthClass">
      <slot />
      <slot name="popoverFooter" />
    </div>
  </n-popover>

  <n-badge v-else :value="activeCount" :show="activeCount > 0">
    <n-button
      size="small"
      secondary
      class="app-motion-soft app-pressable"
      :title="title"
      :aria-label="title"
      @click="emit('update:drawerOpen', true)"
    >
      <template #icon>
        <AppIcon :component="FilterOutline" size="sm" tone="muted" />
      </template>
    </n-button>
  </n-badge>

  <n-drawer
    v-if="!isDesktop"
    :show="drawerOpen"
    placement="bottom"
    height="80vh"
    @update:show="(v) => emit('update:drawerOpen', Boolean(v))"
  >
    <n-drawer-content :title="title" closable>
      <slot />
      <slot name="drawerFooter" />
    </n-drawer-content>
  </n-drawer>
</template>
