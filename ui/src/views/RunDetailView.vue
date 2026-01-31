<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NSpin, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type RunDetail, type RunEvent } from '@/stores/jobs'
import { useOperationsStore, type Operation } from '@/stores/operations'
import { formatToastError } from '@/lib/errors'
import { copyText } from '@/lib/clipboard'

import RestoreWizardModal, { type RestoreWizardModalExpose } from '@/components/jobs/RestoreWizardModal.vue'
import VerifyWizardModal, { type VerifyWizardModalExpose } from '@/components/jobs/VerifyWizardModal.vue'
import OperationModal, { type OperationModalExpose } from '@/components/jobs/OperationModal.vue'
import RunDetailHeader from '@/components/runs/RunDetailHeader.vue'
import RunDetailSummaryCard from '@/components/runs/RunDetailSummaryCard.vue'
import RunDetailDetailsTabs from '@/components/runs/RunDetailDetailsTabs.vue'

type WsStatus = 'disconnected' | 'connecting' | 'live' | 'reconnecting' | 'error'

const { t } = useI18n()
const message = useMessage()

const jobs = useJobsStore()
const operationsStore = useOperationsStore()
const route = useRoute()
const router = useRouter()

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : 'hub'))
const runId = computed(() => (typeof route.params.runId === 'string' ? route.params.runId : null))

const loading = ref<boolean>(false)
const run = ref<RunDetail | null>(null)
const ops = ref<Operation[]>([])
const events = ref<RunEvent[]>([])

const wsStatus = ref<WsStatus>('disconnected')
let socket: WebSocket | null = null
let lastSeq = 0
let allowReconnect = false
let reconnectAttempts = 0
let reconnectTimer: number | null = null
let pollTimer: number | null = null

const restoreModal = ref<RestoreWizardModalExpose | null>(null)
const verifyModal = ref<VerifyWizardModalExpose | null>(null)
const opModal = ref<OperationModalExpose | null>(null)

function wsUrl(path: string): string {
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${proto}//${window.location.host}${path}`
}

function closeSocket(): void {
  if (socket) {
    socket.close()
    socket = null
  }
  if (reconnectTimer !== null) {
    window.clearTimeout(reconnectTimer)
    reconnectTimer = null
  }
  wsStatus.value = 'disconnected'
}

function stopPolling(): void {
  if (pollTimer !== null) {
    window.clearInterval(pollTimer)
    pollTimer = null
  }
}

function reconnectDelaySeconds(attempt: number): number {
  // 1s, 2s, 4s, 8s, ... capped.
  const cappedAttempt = Math.max(0, Math.min(10, attempt))
  return Math.min(30, Math.max(1, 1 << cappedAttempt))
}

function scheduleReconnect(id: string): void {
  if (!allowReconnect) return
  reconnectAttempts += 1
  const delay = reconnectDelaySeconds(reconnectAttempts - 1)
  wsStatus.value = 'reconnecting'

  if (reconnectTimer !== null) window.clearTimeout(reconnectTimer)
  reconnectTimer = window.setTimeout(() => {
    reconnectTimer = null
    if (!allowReconnect) return
    connectWs(id, lastSeq, true)
  }, delay * 1000)
}

function connectWs(id: string, afterSeq: number, isReconnect: boolean): void {
  closeSocket()
  wsStatus.value = isReconnect ? 'reconnecting' : 'connecting'

  const nextSocket = new WebSocket(
    wsUrl(`/api/runs/${encodeURIComponent(id)}/events/ws?after_seq=${encodeURIComponent(String(afterSeq))}`),
  )
  socket = nextSocket

  nextSocket.onopen = () => {
    wsStatus.value = 'live'
    reconnectAttempts = 0
  }

  nextSocket.onmessage = (evt: MessageEvent) => {
    let parsed: unknown
    try {
      parsed = JSON.parse(String(evt.data)) as unknown
    } catch {
      return
    }
    if (!parsed || typeof parsed !== 'object') return
    const e = parsed as RunEvent
    if (typeof e.seq !== 'number') return
    lastSeq = Math.max(lastSeq, e.seq)
    events.value = [...events.value, e]
  }

  nextSocket.onerror = () => {
    wsStatus.value = 'error'
  }

  nextSocket.onclose = () => {
    socket = null
    if (!allowReconnect) return
    scheduleReconnect(id)
  }
}

async function refreshRunAndOps(): Promise<void> {
  const id = runId.value
  if (!id) return
  const [nextRun, nextOps] = await Promise.all([jobs.getRun(id), operationsStore.listRunOperations(id)])
  run.value = nextRun
  ops.value = nextOps
}

async function loadAll(): Promise<void> {
  const id = runId.value
  if (!id) return

  loading.value = true
  run.value = null
  ops.value = []
  events.value = []
  lastSeq = 0
  closeSocket()
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
    lastSeq = nextEvents.reduce((max, e) => Math.max(max, e.seq), 0)

    allowReconnect = true
    connectWs(id, lastSeq, false)

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
  const id = runId.value
  if (!id) return
  restoreModal.value?.open(id, { defaultNodeId: nodeId.value })
}

function openVerify(): void {
  const id = runId.value
  if (!id) return
  verifyModal.value?.open(id)
}

function reconnectEventsWs(): void {
  const id = runId.value
  if (!id) return
  reconnectAttempts = 0
  allowReconnect = true
  connectWs(id, lastSeq, true)
}

async function copyRunId(): Promise<void> {
  const id = runId.value
  if (!id) return
  const ok = await copyText(id)
  if (ok) message.success(t('messages.copied'))
  else message.error(t('errors.copyFailed'))
}

async function openOperation(opId: string): Promise<void> {
  await opModal.value?.open(opId)
}

function backToJobs(): void {
  void router.push(`/n/${encodeURIComponent(nodeId.value)}/jobs`)
}

watch(
  runId,
  (id) => {
    allowReconnect = false
    closeSocket()
    stopPolling()
    if (id) void loadAll()
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  allowReconnect = false
  closeSocket()
  stopPolling()
})
</script>

<template>
  <div class="space-y-3" data-testid="run-detail">
    <run-detail-header
      :run-id="runId"
      :node-id="nodeId"
      :status="run?.status ?? null"
      :loading="loading"
      :can-restore="run?.status === 'success'"
      :can-verify="run?.status === 'success'"
      @back="backToJobs"
      @refresh="loadAll"
      @restore="openRestore"
      @verify="openVerify"
      @copy-run-id="copyRunId"
    />

    <n-spin v-if="loading" size="small" />

    <div
      class="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,420px)_minmax(0,1fr)] md:items-start"
      data-testid="run-detail-layout"
    >
      <div class="md:sticky md:top-3 self-start">
        <run-detail-summary-card :run="run" :events="events" />
      </div>

      <run-detail-details-tabs
        :run-id="runId"
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
