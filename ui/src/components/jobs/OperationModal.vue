<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { NAlert, NButton, NCode, NModal, NSpin, NSpace, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useOperationsStore, type Operation, type OperationEvent } from '@/stores/operations'
import { useUiStore } from '@/stores/ui'
import { MODAL_WIDTH } from '@/lib/modal'
import { useUnixSecondsFormatter } from '@/lib/datetime'

export type OperationModalExpose = {
  open: (opId: string) => Promise<void>
}

const { t } = useI18n()

const operations = useOperationsStore()
const ui = useUiStore()

const show = ref<boolean>(false)
const loading = ref<boolean>(false)
const opId = ref<string | null>(null)
const op = ref<Operation | null>(null)
const events = ref<OperationEvent[]>([])

let pollTimer: number | null = null

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function opStatusTagType(status: Operation['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'running') return 'warning'
  return 'default'
}

function stopPolling(): void {
  if (pollTimer !== null) {
    window.clearInterval(pollTimer)
    pollTimer = null
  }
}

async function refresh(): Promise<void> {
  if (!opId.value) return
  const [nextOp, nextEvents] = await Promise.all([
    operations.getOperation(opId.value),
    operations.listEvents(opId.value),
  ])
  op.value = nextOp
  events.value = nextEvents
  if (nextOp.status !== 'running') {
    stopPolling()
  }
}

async function open(id: string): Promise<void> {
  opId.value = id
  op.value = null
  events.value = []
  show.value = true
  loading.value = true
  try {
    await refresh()
  } finally {
    loading.value = false
  }

  stopPolling()
  pollTimer = window.setInterval(async () => {
    try {
      await refresh()
    } catch {
      stopPolling()
    }
  }, 1000)
}

watch(show, (value) => {
  if (!value) stopPolling()
})

onBeforeUnmount(() => {
  stopPolling()
})

defineExpose<OperationModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('operations.title')">
    <div class="space-y-4">
      <div class="text-sm opacity-70">{{ opId }}</div>

      <div v-if="op" class="flex items-center gap-2">
        <n-tag :type="opStatusTagType(op.status)">{{ op.status }}</n-tag>
        <span class="text-sm opacity-70">{{ t('operations.kind') }}: {{ op.kind }}</span>
        <span class="text-sm opacity-70">{{ t('operations.startedAt') }}: {{ formatUnixSeconds(op.started_at) }}</span>
        <span v-if="op.ended_at" class="text-sm opacity-70">{{ t('operations.endedAt') }}: {{ formatUnixSeconds(op.ended_at) }}</span>
      </div>

      <n-spin v-if="loading" size="small" />

      <n-alert v-if="op?.error" type="error" :title="t('operations.errorTitle')">
        {{ op.error }}
      </n-alert>

      <div v-if="op?.summary" class="space-y-2">
        <div class="text-sm font-medium">{{ t('operations.summary') }}</div>
        <n-code :code="formatJson(op.summary)" language="json" show-line-numbers />
      </div>

      <div class="space-y-2">
        <div class="text-sm font-medium">{{ t('operations.events') }}</div>
        <div class="max-h-80 overflow-auto border rounded-md p-2 bg-[var(--n-color)]">
          <div v-if="events.length === 0" class="text-sm opacity-70">{{ t('operations.noEvents') }}</div>
          <div v-for="e in events" :key="e.seq" class="font-mono text-xs py-1 border-b last:border-b-0 opacity-90">
            <div class="flex flex-wrap gap-2">
              <span class="opacity-70">{{ formatUnixSeconds(e.ts) }}</span>
              <span class="opacity-70">{{ e.level }}</span>
              <span class="opacity-70">{{ e.kind }}</span>
              <span>{{ e.message }}</span>
            </div>
            <n-code v-if="e.fields" class="mt-1" :code="formatJson(e.fields)" language="json" />
          </div>
        </div>
      </div>

      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.close') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>
