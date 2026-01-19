<script setup lang="ts">
import { ref } from 'vue'

import PathPickerModal, { type PathPickerModalExpose } from '@/components/pickers/pathPicker/PathPickerModal.vue'
import { fsPickerCtx, fsPickerDataSource } from '@/components/pickers/pathPicker/fsDataSource'
import type { PathPickerMode, PathPickerOpenOptions } from '@/components/pickers/pathPicker/types'

export type FsPathPickerMode = PathPickerMode
export type FsPathPickerOpenOptions = PathPickerOpenOptions

export type FsPathPickerModalExpose = {
  open: (nodeId: 'hub' | string, initialPath?: string | FsPathPickerOpenOptions) => void
}

const emit = defineEmits<{
  (e: 'picked', paths: string[]): void
}>()

const picker = ref<PathPickerModalExpose | null>(null)

function open(nodeId: 'hub' | string, initialPath?: string | FsPathPickerOpenOptions): void {
  picker.value?.open(fsPickerCtx(nodeId), initialPath)
}

defineExpose<FsPathPickerModalExpose>({ open })
</script>

<template>
  <PathPickerModal
    ref="picker"
    :data-source="fsPickerDataSource"
    @picked="emit('picked', $event)"
  />
</template>

