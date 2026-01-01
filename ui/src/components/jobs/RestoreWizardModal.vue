<script setup lang="ts">
import { computed, ref } from 'vue'
import { NButton, NForm, NFormItem, NInput, NModal, NSelect, NSpace, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useOperationsStore, type ConflictPolicy } from '@/stores/operations'
import { MODAL_WIDTH } from '@/lib/modal'

export type RestoreWizardModalExpose = {
  open: (runId: string) => void
}

const emit = defineEmits<{
  (e: 'started', opId: string): void
}>()

const { t } = useI18n()
const message = useMessage()

const operations = useOperationsStore()

const show = ref<boolean>(false)
const starting = ref<boolean>(false)
const runId = ref<string | null>(null)
const destinationDir = ref<string>('')
const conflictPolicy = ref<ConflictPolicy>('overwrite')

const conflictOptions = computed(() => [
  { label: t('restore.conflict.overwrite'), value: 'overwrite' },
  { label: t('restore.conflict.skip'), value: 'skip' },
  { label: t('restore.conflict.fail'), value: 'fail' },
])

function open(nextRunId: string): void {
  runId.value = nextRunId
  destinationDir.value = ''
  conflictPolicy.value = 'overwrite'
  show.value = true
}

async function start(): Promise<void> {
  const id = runId.value
  if (!id) return

  const destination = destinationDir.value.trim()
  if (!destination) {
    message.error(t('errors.restoreDestinationRequired'))
    return
  }

  starting.value = true
  try {
    const opId = await operations.startRestore(id, destination, conflictPolicy.value)
    show.value = false
    emit('started', opId)
  } catch (error) {
    const msg =
      error && typeof error === 'object' && 'message' in error
        ? String((error as { message: unknown }).message)
        : t('errors.restoreStartFailed')
    message.error(msg)
  } finally {
    starting.value = false
  }
}

defineExpose<RestoreWizardModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('restore.title')">
    <div class="space-y-4">
      <div class="text-sm opacity-70">{{ runId }}</div>
      <n-form label-placement="top">
        <n-form-item :label="t('restore.fields.destinationDir')">
          <div class="space-y-1 w-full">
            <n-input v-model:value="destinationDir" :placeholder="t('restore.fields.destinationDirPlaceholder')" />
            <div class="text-xs opacity-70">{{ t('restore.fields.destinationDirHelp') }}</div>
          </div>
        </n-form-item>
        <n-form-item :label="t('restore.fields.conflictPolicy')">
          <n-select v-model:value="conflictPolicy" :options="conflictOptions" />
        </n-form-item>
      </n-form>
      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
        <n-button type="primary" :loading="starting" @click="start">{{ t('restore.actions.start') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>

