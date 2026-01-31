<script setup lang="ts">
import { NCard } from 'naive-ui'

defineProps<{
  /**
   * Optional compact mode for very dense toolbars.
   * Keep the default spacious layout for most screens.
   */
  compact?: boolean
}>()
</script>

<template>
  <n-card class="app-card" :bordered="false">
    <div
      :class="[
        'flex flex-col gap-3',
        'md:flex-row md:items-end md:justify-between',
        compact ? 'md:gap-2' : '',
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
</template>

