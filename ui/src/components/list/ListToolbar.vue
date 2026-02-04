<script setup lang="ts">
import { Comment, Fragment, Text, computed, useSlots, type VNode } from 'vue'
import { NCard } from 'naive-ui'

const props = defineProps<{
  /**
   * Optional compact mode for very dense toolbars.
   * Keep the default spacious layout for most screens.
   */
  compact?: boolean
  /**
   * Force a stacked (single-column) layout even on desktop breakpoints.
   * Useful for toolbars rendered inside narrow panes (e.g. split views).
   */
  stacked?: boolean
  /**
   * Render the toolbar as an "inset" panel (no Naive UI Card wrapper).
   * Useful when the page already uses a surrounding card (e.g. settings pages).
   */
  embedded?: boolean
}>()

const slots = useSlots()

function isEmptyVNode(node: VNode): boolean {
  if (node.type === Comment) return true
  if (node.type === Text) return String(node.children ?? '').trim().length === 0
  if (node.type === Fragment) {
    const children = node.children
    if (Array.isArray(children)) return children.every((c) => isEmptyVNode(c as VNode))
    return true
  }
  return false
}

const hasActions = computed(() => {
  const nodes = slots.actions?.()
  if (!nodes || nodes.length === 0) return false
  return nodes.some((n) => !isEmptyVNode(n as VNode))
})
</script>

<template>
  <n-card v-if="!props.embedded" class="app-card" :bordered="false">
    <div
      :class="[
        'flex flex-col gap-3',
        props.stacked ? '' : 'md:flex-row md:items-end md:justify-between',
        !props.stacked && props.compact ? 'md:gap-2' : '',
      ]"
    >
      <div class="min-w-0 flex-1">
        <div :class="['flex flex-col gap-3', props.stacked ? '' : 'md:flex-row md:items-end md:flex-wrap']">
          <div v-if="$slots.search" :class="[props.stacked ? '' : 'min-w-[14rem] flex-1']">
            <slot name="search" />
          </div>
          <slot name="filters" />
          <slot name="sort" />
        </div>
      </div>

      <div v-if="hasActions" data-testid="list-toolbar-actions" class="flex items-center justify-end gap-2 flex-wrap">
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
      props.stacked ? '' : 'md:flex-row md:items-end md:justify-between',
      !props.stacked && props.compact ? 'md:gap-2' : '',
    ]"
    :style="{ background: 'var(--app-surface-2)' }"
  >
    <div class="min-w-0 flex-1">
      <div :class="['flex flex-col gap-3', props.stacked ? '' : 'md:flex-row md:items-end md:flex-wrap']">
        <div v-if="$slots.search" :class="[props.stacked ? '' : 'min-w-[14rem] flex-1']">
          <slot name="search" />
        </div>
        <slot name="filters" />
        <slot name="sort" />
      </div>
    </div>

    <div v-if="hasActions" data-testid="list-toolbar-actions" class="flex items-center justify-end gap-2 flex-wrap">
      <slot name="actions" />
    </div>
  </div>
</template>
