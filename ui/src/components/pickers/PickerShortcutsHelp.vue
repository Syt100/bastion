<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NDrawer, NDrawerContent, NIcon, NPopover } from 'naive-ui'
import { HelpCircleOutline } from '@vicons/ionicons5'

export type PickerShortcutItem = {
  combo: string
  description: string
}

const props = withDefaults(
  defineProps<{
    isDesktop: boolean
    title: string
    shortcuts: PickerShortcutItem[]
    note?: string
    widthClass?: string
    popoverOpen: boolean
    drawerOpen: boolean
  }>(),
  {
    note: '',
    widthClass: 'w-80',
  },
)

const emit = defineEmits<{
  (e: 'update:popoverOpen', value: boolean): void
  (e: 'update:drawerOpen', value: boolean): void
}>()

function splitCombo(combo: string): string[] {
  return combo
    .split('+')
    .map((p) => p.trim())
    .filter(Boolean)
}

const renderedShortcuts = computed(() => {
  return props.shortcuts.map((it) => ({
    ...it,
    parts: splitCombo(it.combo),
  }))
})
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
      <n-button size="small" secondary :title="title" :aria-label="title">
        <template #icon>
          <n-icon><help-circle-outline /></n-icon>
        </template>
      </n-button>
    </template>

    <div :class="[widthClass, 'space-y-3']">
      <div class="space-y-2">
        <div v-for="it in renderedShortcuts" :key="it.combo" class="flex items-start gap-2">
          <div class="shrink-0 flex items-center gap-1">
            <template v-for="(part, idx) in it.parts" :key="`${it.combo}:${part}:${idx}`">
              <kbd class="app-kbd">{{ part }}</kbd>
              <span v-if="idx < it.parts.length - 1" class="text-xs app-text-muted">+</span>
            </template>
          </div>
          <div class="min-w-0 text-sm leading-snug">
            {{ it.description }}
          </div>
        </div>
      </div>

      <div v-if="note" class="text-xs app-text-muted leading-snug">
        {{ note }}
      </div>
    </div>
  </n-popover>

  <n-button
    v-else
    size="small"
    secondary
    :title="title"
    :aria-label="title"
    @click="emit('update:drawerOpen', true)"
  >
    <template #icon>
      <n-icon><help-circle-outline /></n-icon>
    </template>
  </n-button>

  <n-drawer
    v-if="!isDesktop"
    :show="drawerOpen"
    placement="bottom"
    height="60vh"
    @update:show="(v) => emit('update:drawerOpen', Boolean(v))"
  >
    <n-drawer-content :title="title" closable>
      <div class="space-y-3">
        <div class="space-y-2">
          <div v-for="it in renderedShortcuts" :key="it.combo" class="flex items-start gap-2">
            <div class="shrink-0 flex items-center gap-1">
              <template v-for="(part, idx) in it.parts" :key="`${it.combo}:${part}:${idx}`">
                <kbd class="app-kbd">{{ part }}</kbd>
                <span v-if="idx < it.parts.length - 1" class="text-xs app-text-muted">+</span>
              </template>
            </div>
            <div class="min-w-0 text-sm leading-snug">
              {{ it.description }}
            </div>
          </div>
        </div>

        <div v-if="note" class="text-xs app-text-muted leading-snug">
          {{ note }}
        </div>
      </div>
    </n-drawer-content>
  </n-drawer>
</template>
