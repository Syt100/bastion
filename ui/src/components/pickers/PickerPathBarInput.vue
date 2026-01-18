<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { NButton, NDrawer, NDrawerContent, NIcon, NInput, NPopover } from 'naive-ui'
import { ArrowUpOutline, RefreshOutline } from '@vicons/ionicons5'

import { MQ } from '@/lib/breakpoints'
import { useMediaQuery } from '@/lib/media'

export type PickerPathBarInputExpose = {
  focus: () => void
}

type BreadcrumbSegment = {
  label: string
  value: string
}

const props = withDefaults(
  defineProps<{
    value: string
    placeholder?: string
    ariaLabel?: string
    upTitle: string
    refreshTitle: string
    disabledUp?: boolean
    disabledRefresh?: boolean
    tailSegmentsDesktop?: number
    tailSegmentsMobile?: number
  }>(),
  {
    placeholder: '',
    ariaLabel: undefined,
    disabledUp: false,
    disabledRefresh: false,
    tailSegmentsDesktop: 2,
    tailSegmentsMobile: 1,
  },
)

const emit = defineEmits<{
  (e: 'update:value', value: string): void
  (e: 'up'): void
  (e: 'refresh'): void
  (e: 'enter'): void
  (e: 'navigate', value: string): void
}>()

const isDesktop = useMediaQuery(MQ.mdUp)

const input = ref<InstanceType<typeof NInput> | null>(null)
const rootEl = ref<HTMLElement | null>(null)

const editing = ref<boolean>(false)
const collapsedPopoverOpen = ref<boolean>(false)
const collapsedDrawerOpen = ref<boolean>(false)

const tailSegments = computed(() => (isDesktop.value ? props.tailSegmentsDesktop : props.tailSegmentsMobile))

function buildBreadcrumbSegments(rawValue: string): BreadcrumbSegment[] {
  const raw = rawValue.trim()
  if (!raw) return []

  const absolute = raw.startsWith('/')
  const parts = raw.split('/').filter(Boolean)

  const segments: BreadcrumbSegment[] = []
  if (absolute) {
    segments.push({ label: '/', value: '/' })
    let acc = ''
    for (const part of parts) {
      acc = `${acc}/${part}`
      segments.push({ label: part, value: acc })
    }
    return segments
  }

  let acc = ''
  for (const part of parts) {
    acc = acc ? `${acc}/${part}` : part
    segments.push({ label: part, value: acc })
  }
  return segments
}

const allSegments = computed(() => buildBreadcrumbSegments(props.value))

const collapseMiddle = computed(() => allSegments.value.length > tailSegments.value + 2)

const hiddenSegments = computed(() => {
  if (!collapseMiddle.value) return []
  return allSegments.value.slice(1, -tailSegments.value)
})

const shownSegments = computed<BreadcrumbSegment[]>(() => {
  if (!collapseMiddle.value) return allSegments.value
  const head = allSegments.value[0]
  const tail = allSegments.value.slice(-tailSegments.value)
  return [
    ...(head ? [head] : []),
    { label: '…', value: '' },
    ...tail,
  ]
})

function focus(): void {
  try {
    input.value?.focus?.()
  } catch {
    // ignore
  }
}

function enterEditMode(): void {
  if (editing.value) return
  editing.value = true
  nextTick().then(() => focus())
}

function maybeExitEditMode(): void {
  if (!editing.value) return
  editing.value = false
}

function onInputBlur(): void {
  // Keep edit mode when focus moves to the prefix buttons inside the same bar.
  setTimeout(() => {
    const active = document.activeElement as HTMLElement | null
    if (active && rootEl.value?.contains(active)) return
    maybeExitEditMode()
  }, 0)
}

function onContainerClick(): void {
  if (!editing.value) enterEditMode()
}

function onKeydown(e: KeyboardEvent): void {
  if (editing.value) return
  if (e.ctrlKey || e.metaKey || e.altKey) return

  if (e.key === 'Enter') {
    emit('enter')
    return
  }

  // Match the "address bar" behavior: start editing and replace the current value.
  if (e.key === 'Backspace' || e.key === 'Delete') {
    e.preventDefault()
    emit('update:value', '')
    enterEditMode()
    return
  }

  // Printable character.
  if (e.key.length === 1) {
    e.preventDefault()
    emit('update:value', e.key)
    enterEditMode()
  }
}

function onEnter(): void {
  emit('enter')
  maybeExitEditMode()
}

function navigateTo(value: string): void {
  if (!value.trim()) return
  collapsedPopoverOpen.value = false
  collapsedDrawerOpen.value = false
  emit('update:value', value)
  emit('navigate', value)
  maybeExitEditMode()
}

defineExpose<PickerPathBarInputExpose>({ focus })
</script>

<template>
  <div ref="rootEl" @click="onContainerClick">
    <n-input
      ref="input"
      :value="editing ? value : ''"
      :placeholder="editing ? placeholder : ''"
      :aria-label="ariaLabel || placeholder"
      :readonly="!editing"
      @update:value="(v) => emit('update:value', v)"
      @keydown="onKeydown"
      @blur="onInputBlur"
      @keyup.enter="onEnter"
      @keyup.esc="maybeExitEditMode"
    >
      <template #prefix>
        <div class="flex items-center gap-1 -ml-1 min-w-0">
          <div class="flex items-center gap-1 shrink-0">
            <n-button
              circle
              quaternary
              size="small"
              :disabled="disabledUp"
              :title="upTitle"
              @click.stop="emit('up')"
            >
              <template #icon>
                <n-icon class="app-picker-path-action-icon opacity-80" :size="18">
                  <arrow-up-outline />
                </n-icon>
              </template>
            </n-button>
            <n-button
              circle
              quaternary
              size="small"
              :disabled="disabledRefresh"
              :title="refreshTitle"
              @click.stop="emit('refresh')"
            >
              <template #icon>
                <n-icon class="app-picker-path-action-icon opacity-80" :size="18">
                  <refresh-outline />
                </n-icon>
              </template>
            </n-button>
          </div>

          <div v-if="!editing" class="flex items-center min-w-0 overflow-hidden">
            <div v-if="allSegments.length === 0" class="text-xs opacity-60 truncate" @click.stop="enterEditMode">
              {{ placeholder }}
            </div>

            <div v-else class="flex items-center min-w-0 overflow-hidden">
              <template v-for="(seg, idx) in shownSegments" :key="`${idx}:${seg.label}:${seg.value}`">
                <span
                  v-if="idx > 0 && !(shownSegments[0]?.label === '/' && idx === 1)"
                  class="text-xs opacity-50 mx-0.5 shrink-0"
                >
                  /
                </span>

                <template v-if="seg.label !== '…'">
                  <button
                    type="button"
                    class="text-xs hover:underline truncate max-w-[16rem] text-[var(--n-text-color-1)]"
                    :title="seg.label"
                    @click.stop="navigateTo(seg.value)"
                  >
                    {{ seg.label }}
                  </button>
                </template>
                <template v-else>
                  <n-popover
                    v-if="isDesktop"
                    v-model:show="collapsedPopoverOpen"
                    trigger="click"
                    placement="bottom-start"
                    :show-arrow="false"
                  >
                    <template #trigger>
                      <button
                        type="button"
                        class="text-xs hover:underline shrink-0 text-[var(--n-text-color-1)]"
                        title="..."
                        @click.stop
                      >
                        …
                      </button>
                    </template>
                    <div class="space-y-1">
                      <button
                        v-for="h in hiddenSegments"
                        :key="h.value"
                        type="button"
                        class="block text-left w-full text-xs hover:underline"
                        :title="h.label"
                        @click="navigateTo(h.value)"
                      >
                        {{ h.label }}
                      </button>
                    </div>
                  </n-popover>

                  <button
                    v-else
                    type="button"
                    class="text-xs hover:underline shrink-0 text-[var(--n-text-color-1)]"
                    title="..."
                    @click.stop="collapsedDrawerOpen = true"
                  >
                    …
                  </button>
                </template>
              </template>
            </div>
          </div>
        </div>
      </template>
    </n-input>

    <n-drawer v-if="!isDesktop" v-model:show="collapsedDrawerOpen" placement="bottom" height="60vh">
      <n-drawer-content :title="placeholder" closable>
        <div class="space-y-2">
          <button
            v-for="h in hiddenSegments"
            :key="h.value"
            type="button"
            class="block text-left w-full px-2 py-1 rounded hover:bg-black/5 dark:hover:bg-white/10"
            :title="h.label"
            @click="navigateTo(h.value)"
          >
            <div class="text-sm truncate">{{ h.label }}</div>
          </button>
        </div>
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<style scoped>
/* Ionicons outline icons default to a fairly heavy stroke; soften for path bar actions. */
.app-picker-path-action-icon :deep(svg) {
  stroke-width: 24;
}
</style>
