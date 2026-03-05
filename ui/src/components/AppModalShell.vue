<script setup lang="ts">
import { NModal } from 'naive-ui'
import { computed, type CSSProperties } from 'vue'

import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'

const props = withDefaults(
  defineProps<{
    show: boolean
    title: string
    width?: string
    containerStyle?: CSSProperties
    style?: CSSProperties
    maskClosable?: boolean
    scrollBody?: boolean
    bodyClass?: string
    footerClass?: string
    contentStyle?: CSSProperties
    bodyStyle?: CSSProperties
  }>(),
  {
    width: MODAL_WIDTH.md,
    containerStyle: undefined,
    style: undefined,
    maskClosable: true,
    scrollBody: true,
    bodyClass: undefined,
    footerClass: undefined,
    contentStyle: undefined,
    bodyStyle: undefined,
  },
)

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
}>()

const modalStyle = computed<CSSProperties>(() => ({
  width: props.width,
  maxHeight: MODAL_HEIGHT.max,
  ...(props.style ?? {}),
  ...(props.containerStyle ?? {}),
}))
const resolvedContentStyle = computed<CSSProperties>(() => {
  if (props.contentStyle) return props.contentStyle
  return {
    display: 'flex',
    flexDirection: 'column',
    minHeight: 0,
  }
})
const resolvedBodyStyle = computed<CSSProperties | undefined>(() => props.bodyStyle)
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

    <div :class="[scrollBody ? 'app-modal-shell__body' : 'app-modal-shell__body-plain', bodyClass]" :style="resolvedBodyStyle">
      <slot />
    </div>

    <template v-if="$slots.footer" #footer>
      <div class="app-modal-shell__footer" :class="footerClass">
        <slot name="footer" />
      </div>
    </template>
  </n-modal>
</template>
