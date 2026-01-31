<script setup lang="ts">
import { NCard } from 'naive-ui'

const props = defineProps<{
  /**
   * Optional compact mode for very dense toolbars.
   * Keep the default spacious layout for most screens.
   */
  compact?: boolean
  /**
   * Render the toolbar as an "inset" panel (no Naive UI Card wrapper).
   * Useful when the page already uses a surrounding card (e.g. settings pages).
   */
  embedded?: boolean
}>()
</script>

<template>
  <n-card v-if="!props.embedded" class="app-card" :bordered="false">
    <div
      :class="[
        'flex flex-col gap-3',
        'md:flex-row md:items-end md:justify-between',
        props.compact ? 'md:gap-2' : '',
      ]"
    >
      <div class="min-w-0 flex-1">
        <div class="flex flex-col gap-3 md:flex-row md:items-end md:flex-wrap">
          <div v-if="$slots.search" class="min-w-[14rem] flex-1">
            <slot name="search" />
          </div>
          <slot name="filters" />
          <slot name="sort" />
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 flex-wrap">
        <slot name="actions" />
      </div>
    </div>
  </n-card>

  <div
    v-else
    :class="[
      'rounded-xl app-border-subtle',
      props.compact ? 'p-3' : 'p-4',
      'flex flex-col gap-3',
      'md:flex-row md:items-end md:justify-between',
      props.compact ? 'md:gap-2' : '',
    ]"
    :style="{ background: 'var(--app-surface-2)' }"
  >
    <div class="min-w-0 flex-1">
      <div class="flex flex-col gap-3 md:flex-row md:items-end md:flex-wrap">
        <div v-if="$slots.search" class="min-w-[14rem] flex-1">
          <slot name="search" />
        </div>
        <slot name="filters" />
        <slot name="sort" />
      </div>
    </div>

    <div class="flex items-center justify-end gap-2 flex-wrap">
      <slot name="actions" />
    </div>
  </div>
</template>
