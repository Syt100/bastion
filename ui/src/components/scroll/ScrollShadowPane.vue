<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, onUpdated, ref, useAttrs } from 'vue'

type ShadowFrom = string

const props = defineProps<{
  /**
   * The color (or CSS variable) used for the fade shadows.
   * Examples: 'var(--app-surface)', 'var(--app-bg-solid)'.
   */
  shadowFrom?: ShadowFrom
  /**
   * Optional wrapper classes for sizing (e.g. 'flex-1 min-h-0').
   * Note: the scroll element itself receives the component's attrs (class, data-testid, etc.).
   */
  wrapperClass?: string
  shadowSizePx?: number
}>()

defineOptions({ inheritAttrs: false })
const attrs = useAttrs()

const scroller = ref<HTMLElement | null>(null)
const showTop = ref(false)
const showBottom = ref(false)

const shadowFrom = computed(() => props.shadowFrom ?? 'var(--app-surface)')
const shadowSize = computed(() => `${Math.max(8, Math.floor(props.shadowSizePx ?? 16))}px`)

const topShadowStyle = computed(() => ({
  height: shadowSize.value,
  background: `linear-gradient(to bottom, ${shadowFrom.value}, transparent)`,
}))

const bottomShadowStyle = computed(() => ({
  height: shadowSize.value,
  background: `linear-gradient(to top, ${shadowFrom.value}, transparent)`,
}))

function update(): void {
  const el = scroller.value
  if (!el) return

  const hasOverflow = el.scrollHeight > el.clientHeight + 1
  if (!hasOverflow) {
    showTop.value = false
    showBottom.value = false
    return
  }

  showTop.value = el.scrollTop > 0
  showBottom.value = el.scrollTop + el.clientHeight < el.scrollHeight - 1
}

let raf: number | null = null
function onScroll(): void {
  if (typeof requestAnimationFrame === 'undefined') {
    update()
    return
  }
  if (raf != null) cancelAnimationFrame(raf)
  raf = requestAnimationFrame(() => {
    raf = null
    update()
  })
}

let ro: ResizeObserver | null = null

onMounted(() => {
  update()
  const el = scroller.value
  if (!el) return
  el.addEventListener('scroll', onScroll, { passive: true })
  if (typeof ResizeObserver !== 'undefined') {
    ro = new ResizeObserver(() => update())
    ro.observe(el)
  }
})

onUpdated(() => {
  update()
})

onBeforeUnmount(() => {
  const el = scroller.value
  if (el) el.removeEventListener('scroll', onScroll)
  if (raf != null && typeof cancelAnimationFrame !== 'undefined') cancelAnimationFrame(raf)
  if (ro) ro.disconnect()
})
</script>

<template>
  <div class="relative h-full min-h-0" :class="props.wrapperClass">
    <div
      ref="scroller"
      class="h-full min-h-0 overflow-y-auto overscroll-contain"
      v-bind="attrs"
    >
      <slot />
    </div>

    <div v-show="showTop" class="pointer-events-none absolute top-0 left-0 right-0 z-10" :style="topShadowStyle" />
    <div
      v-show="showBottom"
      class="pointer-events-none absolute bottom-0 left-0 right-0 z-10"
      :style="bottomShadowStyle"
    />
  </div>
</template>
