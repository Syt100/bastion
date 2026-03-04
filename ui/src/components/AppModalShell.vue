<script setup lang="ts">
import { NModal } from 'naive-ui'
import { computed, type CSSProperties } from 'vue'

import { MODAL_WIDTH } from '@/lib/modal'

const props = withDefaults(
  defineProps<{
    show: boolean
    title: string
    width?: string
    maskClosable?: boolean
    scrollBody?: boolean
    bodyClass?: string
    footerClass?: string
    contentStyle?: CSSProperties
  }>(),
  {
    width: MODAL_WIDTH.md,
    maskClosable: true,
    scrollBody: true,
    bodyClass: undefined,
    footerClass: undefined,
    contentStyle: undefined,
  },
)

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
}>()

const modalStyle = computed<CSSProperties>(() => ({ width: props.width }))
const resolvedContentStyle = computed<CSSProperties>(() => {
  if (props.contentStyle) return props.contentStyle
  return {
    display: 'flex',
    flexDirection: 'column',
    maxHeight: 'calc(100vh - 64px)',
    minHeight: 0,
  }
})
</script>

<template>
  <n-modal
    :show="show"
    preset="card"
    :mask-closable="maskClosable"
    :style="modalStyle"
    :content-style="resolvedContentStyle"
    :title="title"
    @update:show="(v) => emit('update:show', v)"
  >
    <template v-if="$slots.header" #header>
      <slot name="header" />
    </template>

    <template v-if="$slots['header-extra']" #header-extra>
      <slot name="header-extra" />
    </template>

    <div :class="[scrollBody ? 'app-modal-shell__body' : 'app-modal-shell__body-plain', bodyClass]">
      <slot />
    </div>

    <template v-if="$slots.footer" #footer>
      <div class="app-modal-shell__footer" :class="footerClass">
        <slot name="footer" />
      </div>
    </template>
  </n-modal>
</template>
