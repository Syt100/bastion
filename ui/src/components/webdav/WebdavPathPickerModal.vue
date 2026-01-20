<script setup lang="ts">
import { ref } from 'vue'

import PathPickerModal, { type PathPickerModalExpose } from '@/components/pickers/pathPicker/PathPickerModal.vue'
import { webdavPickerCtx, webdavPickerDataSource, type WebdavPickerContext } from '@/components/pickers/pathPicker/webdavDataSource'
import type { PathPickerOpenOptions } from '@/components/pickers/pathPicker/types'

export type WebdavPathPickerOpenOptions = PathPickerOpenOptions
export type WebdavPathPickerModalExpose = {
  open: (ctx: WebdavPickerContext, initialPath?: string | WebdavPathPickerOpenOptions) => void
}

const emit = defineEmits<{
  (e: 'picked', paths: string[]): void
}>()

const picker = ref<PathPickerModalExpose | null>(null)

function open(ctx: WebdavPickerContext, initialPath?: string | WebdavPathPickerOpenOptions): void {
  picker.value?.open(webdavPickerCtx(ctx.nodeId, ctx.baseUrl, ctx.secretName), initialPath)
}

defineExpose<WebdavPathPickerModalExpose>({ open })
</script>

<template>
  <PathPickerModal
    ref="picker"
    :data-source="webdavPickerDataSource"
    @picked="emit('picked', $event)"
  />
</template>

