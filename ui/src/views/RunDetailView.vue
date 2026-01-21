<script setup lang="ts">
import { computed, h, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NAlert,
  NButton,
  NCard,
  NCode,
  NDataTable,
  NSpin,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import { useUiStore } from '@/stores/ui'
import { useJobsStore, type RunDetail, type RunEvent } from '@/stores/jobs'
import { useOperationsStore, type Operation } from '@/stores/operations'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'

import RestoreWizardModal, { type RestoreWizardModalExpose } from '@/components/jobs/RestoreWizardModal.vue'
import VerifyWizardModal, { type VerifyWizardModalExpose } from '@/components/jobs/VerifyWizardModal.vue'
import OperationModal, { type OperationModalExpose } from '@/components/jobs/OperationModal.vue'
import RunProgressPanel from '@/components/runs/RunProgressPanel.vue'

type WsStatus = 'disconnected' | 'connecting' | 'live' | 'reconnecting' | 'error'

type ProgressUnits = { files: number; dirs: number; bytes: number }
type ProgressSnapshot = {
  v: number
  kind: string
  stage: string
  ts: number
  done: ProgressUnits
  total?: ProgressUnits | null
  rate_bps?: number | null
  eta_seconds?: number | null
  detail?: unknown | null
}

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
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

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

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

function statusTagType(status: RunDetail['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  if (status === 'running') return 'warning'
  return 'default'
}

function opStatusTagType(status: Operation['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'running') return 'warning'
  return 'default'
}

function opKindLabel(kind: Operation['kind']): string {
  return kind
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

async function openOperation(opId: string): Promise<void> {
  await opModal.value?.open(opId)
}

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

const opColumns = computed<DataTableColumns<Operation>>(() => [
  {
    title: t('operations.kind'),
    key: 'kind',
    render: (row) => opKindLabel(row.kind),
  },
  {
    title: t('runs.columns.status'),
    key: 'status',
    render: (row) => h(NTag, { type: opStatusTagType(row.status) }, { default: () => row.status }),
  },
  {
    title: t('bulk.columns.progress'),
    key: 'progress',
    render: (row) => {
      const p = row.progress
      if (!p || typeof p !== 'object') return '-'
      const obj = p as ProgressSnapshot
      return typeof obj.stage === 'string' ? obj.stage : '-'
    },
  },
  {
    title: t('operations.startedAt'),
    key: 'started_at',
    render: (row) => formatUnixSeconds(row.started_at),
  },
  {
    title: t('operations.endedAt'),
    key: 'ended_at',
    render: (row) => formatUnixSeconds(row.ended_at),
  },
  {
    title: t('runs.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [h(NButton, { size: 'small', onClick: () => void openOperation(row.id) }, { default: () => t('runEvents.actions.details') })],
        },
      ),
  },
])

const eventColumns = computed<DataTableColumns<RunEvent>>(() => [
  { title: t('runs.columns.startedAt'), key: 'ts', render: (row) => formatUnixSeconds(row.ts) },
  { title: t('runs.columns.status'), key: 'level', render: (row) => row.level },
  { title: 'kind', key: 'kind', render: (row) => row.kind },
  { title: 'message', key: 'message', render: (row) => row.message },
])

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
  <div class="space-y-6">
    <page-header :title="t('runs.title')" :subtitle="runId || undefined">
      <n-button size="small" @click="backToJobs">{{ t('common.back') }}</n-button>
      <n-button size="small" :loading="loading" @click="loadAll">{{ t('common.refresh') }}</n-button>
      <n-button size="small" type="primary" :disabled="run?.status !== 'success'" @click="openRestore">
        {{ t('runs.actions.restore') }}
      </n-button>
      <n-button size="small" :disabled="run?.status !== 'success'" @click="openVerify">
        {{ t('runs.actions.verify') }}
      </n-button>
    </page-header>

    <n-spin v-if="loading" size="small" />

    <n-alert v-if="run?.error" type="error" :title="t('operations.errorTitle')">{{ run.error }}</n-alert>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <n-card :title="t('runs.title')">
        <div v-if="run" class="space-y-2">
          <div class="flex items-center gap-2">
            <n-tag :type="statusTagType(run.status)">{{ run.status }}</n-tag>
            <span class="text-sm opacity-70">{{ run.id }}</span>
          </div>
          <div class="text-sm opacity-70">{{ t('runs.columns.startedAt') }}: {{ formatUnixSeconds(run.started_at) }}</div>
          <div class="text-sm opacity-70">{{ t('runs.columns.endedAt') }}: {{ formatUnixSeconds(run.ended_at) }}</div>
        </div>
        <div v-else class="text-sm opacity-70">-</div>
      </n-card>

      <run-progress-panel :progress="run?.progress" />
    </div>

    <n-card :title="t('operations.title')">
      <n-data-table :columns="opColumns" :data="ops" :bordered="false" />
    </n-card>

    <n-card :title="t('runEvents.title')">
      <div class="flex items-center gap-2 mb-3">
        <n-tag size="small" :type="wsStatus === 'live' ? 'success' : wsStatus === 'error' ? 'error' : wsStatus === 'reconnecting' ? 'warning' : 'default'">
          {{ wsStatus }}
        </n-tag>
        <div class="text-xs opacity-70">{{ events.length }} events</div>
      </div>
      <n-data-table :columns="eventColumns" :data="events" :bordered="false" />
    </n-card>

    <n-card v-if="run?.summary" :title="t('operations.summary')">
      <n-code :code="formatJson(run.summary)" language="json" show-line-numbers />
    </n-card>

    <restore-wizard-modal ref="restoreModal" @started="(id) => openOperation(id)" />
    <verify-wizard-modal ref="verifyModal" @started="(id) => openOperation(id)" />
    <operation-modal ref="opModal" />
  </div>
</template>
