<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { NButton, NDropdown, NIcon, NSpin, NTag, useMessage } from 'naive-ui'
import { EllipsisHorizontal } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type RunDetail, type RunEvent } from '@/stores/jobs'
import { useOperationsStore, type Operation } from '@/stores/operations'
import { formatToastError } from '@/lib/errors'
import { copyText } from '@/lib/clipboard'
import { useRunEventsStream } from '@/lib/runEventsStream'
import NodeContextTag from '@/components/NodeContextTag.vue'
import RunDetailSummaryCard from '@/components/runs/RunDetailSummaryCard.vue'
import RunDetailDetailsTabs from '@/components/runs/RunDetailDetailsTabs.vue'
import RestoreWizardModal, { type RestoreWizardModalExpose } from '@/components/jobs/RestoreWizardModal.vue'
import VerifyWizardModal, { type VerifyWizardModalExpose } from '@/components/jobs/VerifyWizardModal.vue'
import OperationModal, { type OperationModalExpose } from '@/components/jobs/OperationModal.vue'
import { runStatusLabel } from '@/lib/runs'

const props = defineProps<{
  nodeId?: string
  runId: string
}>()

const { t } = useI18n()
const message = useMessage()

const jobs = useJobsStore()
const operationsStore = useOperationsStore()

const loading = ref<boolean>(false)
const cancelRunBusy = ref<boolean>(false)
const run = ref<RunDetail | null>(null)
const ops = ref<Operation[]>([])
const events = ref<RunEvent[]>([])

let pollTimer: number | null = null
const runEventsStream = useRunEventsStream({
  buildUrl: (id, afterSeq) =>
    wsUrl(`/api/runs/${encodeURIComponent(id)}/events/ws?after_seq=${encodeURIComponent(String(afterSeq))}`),
  onEvent: (event) => {
    events.value = [...events.value, event]
  },
})
const wsStatus = runEventsStream.status

const restoreModal = ref<RestoreWizardModalExpose | null>(null)
const verifyModal = ref<VerifyWizardModalExpose | null>(null)
const opModal = ref<OperationModalExpose | null>(null)

function wsUrl(path: string): string {
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${proto}//${window.location.host}${path}`
}

function stopPolling(): void {
  if (pollTimer !== null) {
    window.clearInterval(pollTimer)
    pollTimer = null
  }
}

async function refreshRunAndOps(): Promise<void> {
  const id = props.runId
  const [nextRun, nextOps] = await Promise.all([jobs.getRun(id), operationsStore.listRunOperations(id)])
  run.value = nextRun
  ops.value = nextOps
}

async function loadAll(): Promise<void> {
  const id = props.runId
  loading.value = true
  run.value = null
  ops.value = []
  events.value = []
  runEventsStream.stop()
  runEventsStream.setLastSeq(0)
  stopPolling()

  try {
    const [nextRun, nextOps, nextEvents] = await Promise.all([
      jobs.getRun(id),
      operationsStore.listRunOperations(id),
      jobs.listRunEvents(id),
    ])
    run.value = nextRun
    ops.value = nextOps
    events.value = nextEvents
    const maxSeq = nextEvents.reduce((max, e) => Math.max(max, e.seq), 0)
    runEventsStream.setLastSeq(maxSeq)

    runEventsStream.start(id, maxSeq)

    stopPolling()
    pollTimer = window.setInterval(async () => {
      try {
        const current = run.value
        const hasRunningOp = ops.value.some((o) => o.status === 'running')
        if (current?.status !== 'running' && !hasRunningOp) {
          stopPolling()
          return
        }
        await refreshRunAndOps()
      } catch {
        // Stop polling on repeated errors; user can manually refresh.
        stopPolling()
      }
    }, 1000)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchRunsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

function openRestore(): void {
  restoreModal.value?.open(props.runId, { defaultNodeId: resolvedNodeId.value })
}

function openVerify(): void {
  verifyModal.value?.open(props.runId)
}

async function requestCancelRun(): Promise<void> {
  const current = run.value
  if (!current) return
  if ((current.status !== 'queued' && current.status !== 'running') || current.cancel_requested_at != null || cancelRunBusy.value) {
    return
  }
  if (!window.confirm(t('runs.actions.cancelConfirm'))) return

  cancelRunBusy.value = true
  try {
    const next = await jobs.cancelRun(current.id)
    run.value = next
    if (next.status === 'canceled') {
      message.success(t('messages.runCanceled'))
    } else {
      message.success(t('messages.runCancelRequested'))
    }
  } catch (error) {
    message.error(formatToastError(t('errors.cancelRunFailed'), error, t))
  } finally {
    cancelRunBusy.value = false
  }
}

function reconnectEventsWs(): void {
  runEventsStream.reconnect(props.runId)
}

async function copyRunId(): Promise<void> {
  const ok = await copyText(props.runId)
  if (ok) message.success(t('messages.copied'))
  else message.error(t('errors.copyFailed'))
}

async function openOperation(opId: string): Promise<void> {
  await opModal.value?.open(opId)
}

watch(
  () => props.runId,
  (id) => {
    runEventsStream.stop()
    stopPolling()
    if (id) void loadAll()
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  runEventsStream.stop()
  stopPolling()
})

const canRestore = computed(() => run.value?.status === 'success')
const canVerify = computed(() => run.value?.status === 'success')
const resolvedNodeId = computed(() => props.nodeId || run.value?.node_id || 'hub')
const runCancelRequested = computed(() => run.value?.cancel_requested_at != null)
const runCancelInProgress = computed(
  () => run.value?.status === 'running' && (runCancelRequested.value || cancelRunBusy.value),
)
const canCancelRun = computed(() => {
  const status = run.value?.status
  if ((status !== 'queued' && status !== 'running') || cancelRunBusy.value) return false
  return !runCancelRequested.value
})
const runStatusText = computed(() => {
  const status = run.value?.status
  if (!status) return ''
  if (runCancelInProgress.value) return t('runs.statuses.canceling')
  return runStatusLabel(t, status)
})

function statusTagType(status: RunDetail['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  if (status === 'running') return 'warning'
  return 'default'
}
</script>

<template>
  <div class="space-y-3" data-testid="run-detail-panel">
    <div class="flex items-start justify-between gap-3 flex-wrap">
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <NodeContextTag :node-id="resolvedNodeId" />
          <n-tag v-if="run?.status" data-testid="run-status-tag" size="small" :bordered="false" :type="statusTagType(run.status)">
            {{ runStatusText }}
          </n-tag>
        </div>

        <div class="mt-2 flex items-center gap-2 min-w-0 text-sm app-text-muted">
          <span class="font-mono tabular-nums truncate">{{ props.runId }}</span>
          <n-button size="tiny" quaternary @click="copyRunId">{{ t('common.copy') }}</n-button>
        </div>
      </div>

      <div class="flex items-center gap-2 flex-wrap justify-end">
        <n-button size="small" :loading="loading" @click="loadAll">{{ t('common.refresh') }}</n-button>
        <n-button
          v-if="run && (run.status === 'queued' || run.status === 'running')"
          data-testid="run-cancel-button"
          size="small"
          type="warning"
          :loading="cancelRunBusy"
          :disabled="!canCancelRun"
          @click="requestCancelRun"
        >
          {{ runCancelInProgress ? t('runs.actions.canceling') : t('runs.actions.cancel') }}
        </n-button>
        <n-button size="small" type="primary" :disabled="!canRestore" @click="openRestore">
          {{ t('runs.actions.restore') }}
        </n-button>

        <n-dropdown
          trigger="click"
          :options="[
            {
              label: t('runs.actions.verify'),
              key: 'verify',
              disabled: !canVerify,
            },
          ]"
          @select="(key) => (key === 'verify' ? openVerify() : null)"
        >
          <n-button size="small" quaternary>
            <template #icon>
              <n-icon :component="EllipsisHorizontal" />
            </template>
            {{ t('common.more') }}
          </n-button>
        </n-dropdown>
      </div>
    </div>

    <n-spin v-if="loading" size="small" />

    <div
      class="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,420px)_minmax(0,1fr)] md:items-start"
      data-testid="run-detail-layout"
    >
      <div class="md:sticky md:top-3 self-start">
        <run-detail-summary-card :run="run" :events="events" />
      </div>

      <run-detail-details-tabs
        :run-id="props.runId"
        :events="events"
        :ops="ops"
        :ws-status="wsStatus"
        :summary="run?.summary ?? null"
        @open-operation="openOperation"
        @reconnect="reconnectEventsWs"
      />
    </div>

    <restore-wizard-modal ref="restoreModal" @started="(id) => openOperation(id)" />
    <verify-wizard-modal ref="verifyModal" @started="(id) => openOperation(id)" />
    <operation-modal ref="opModal" />
  </div>
</template>
