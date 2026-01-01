<script setup lang="ts">
import { ref } from 'vue'
import { NAlert, NButton, NModal, NSpace, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useOperationsStore } from '@/stores/operations'
import { MODAL_WIDTH } from '@/lib/modal'

export type VerifyWizardModalExpose = {
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

function open(nextRunId: string): void {
  runId.value = nextRunId
  show.value = true
}

async function start(): Promise<void> {
  const id = runId.value
  if (!id) return

  starting.value = true
  try {
    const opId = await operations.startVerify(id)
    show.value = false
    emit('started', opId)
  } catch (error) {
    const msg =
      error && typeof error === 'object' && 'message' in error
        ? String((error as { message: unknown }).message)
        : t('errors.verifyStartFailed')
    message.error(msg)
  } finally {
    starting.value = false
  }
}

defineExpose<VerifyWizardModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('verify.title')">
    <div class="space-y-4">
      <div class="text-sm opacity-70">{{ runId }}</div>
      <n-alert type="info" :title="t('verify.helpTitle')">
        {{ t('verify.helpBody') }}
      </n-alert>
      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
        <n-button type="primary" :loading="starting" @click="start">{{ t('verify.actions.start') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>

