<script setup lang="ts">
import { computed, h, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NAlert,
  NButton,
  NCard,
  NCode,
  NDataTable,
  NDrawer,
  NDrawerContent,
  NDropdown,
  NIcon,
  NModal,
  NSpin,
  NSpace,
  NTabs,
  NTabPane,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { EllipsisHorizontal } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import { useUiStore } from '@/stores/ui'
import { useJobsStore, type RunDetail, type RunEvent } from '@/stores/jobs'
import { useOperationsStore, type Operation } from '@/stores/operations'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { copyText } from '@/lib/clipboard'
import { MODAL_WIDTH } from '@/lib/modal'
import { MQ } from '@/lib/breakpoints'
import { useMediaQuery } from '@/lib/media'
import { runStatusLabel, runTargetTypeLabel } from '@/lib/runs'

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

const isDesktop = useMediaQuery(MQ.mdUp)
const detailTab = ref<'events' | 'operations' | 'summary'>('events')

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

type RunSummary = {
  target?: { type?: string; run_dir?: string; run_url?: string } | null
  entries_count?: number | null
  parts?: number | null
  filesystem?: { warnings_total?: number | null; errors_total?: number | null } | null
  sqlite?: { path?: string | null; snapshot_name?: string | null } | null
  vaultwarden?: { data_dir?: string | null; db?: string | null } | null
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as Record<string, unknown>
}

function asString(value: unknown): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}

function asNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null
}

const summary = computed<RunSummary | null>(() => {
  const obj = asRecord(run.value?.summary)
  if (!obj) return null
  return obj as RunSummary
})

const targetSummary = computed(() => {
  const target = asRecord(summary.value?.target ?? null)
  const type = asString(target?.type)
  const runDir = asString(target?.run_dir)
  const runUrl = asString(target?.run_url)
  return { type, location: runDir ?? runUrl }
})

const targetTypeLabel = computed(() => runTargetTypeLabel(t, targetSummary.value.type))

const entriesCount = computed(() => asNumber(summary.value?.entries_count ?? null))
const partsCount = computed(() => asNumber(summary.value?.parts ?? null))

const fsIssues = computed(() => asRecord(summary.value?.filesystem ?? null))
const warningsTotal = computed(() => asNumber(fsIssues.value?.warnings_total))
const errorsTotal = computed(() => asNumber(fsIssues.value?.errors_total))

function formatDuration(seconds: number | null): string {
  if (seconds == null || !Number.isFinite(seconds) || seconds < 0) return '-'
  const s = Math.floor(seconds)
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = s % 60
  if (h > 0) return `${h}h ${m}m ${sec}s`
  if (m > 0) return `${m}m ${sec}s`
  return `${sec}s`
}

const durationSeconds = computed<number | null>(() => {
  const r = run.value
  if (!r) return null
  if (!r.started_at || !r.ended_at) return null
  return Math.max(0, r.ended_at - r.started_at)
})

function runEventLevelTagType(level: string): 'success' | 'error' | 'warning' | 'default' {
  if (level === 'error') return 'error'
  if (level === 'warn' || level === 'warning') return 'warning'
  if (level === 'info') return 'success'
  return 'default'
}

function wsStatusTagType(status: WsStatus): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'live') return 'success'
  if (status === 'error') return 'error'
  if (status === 'reconnecting') return 'warning'
  return 'default'
}

const eventDetailShow = ref<boolean>(false)
const eventDetail = ref<RunEvent | null>(null)

function openEventDetails(e: RunEvent): void {
  eventDetail.value = e
  eventDetailShow.value = true
}

async function copySummaryJson(): Promise<void> {
  const value = run.value?.summary
  if (value == null) return
  const ok = await copyText(formatJson(value))
  if (ok) message.success(t('messages.copied'))
  else message.error(t('errors.copyFailed'))
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
  <div class="space-y-4" data-testid="run-detail">
    <page-header :title="t('runs.title')">
      <template #subtitle>
        <div v-if="runId" class="flex items-center gap-2 text-sm opacity-70 min-w-0">
          <span class="font-mono tabular-nums truncate">{{ runId }}</span>
          <n-button size="tiny" quaternary @click="copyRunId">{{ t('common.copy') }}</n-button>
        </div>
      </template>

      <n-tag v-if="run" size="small" :bordered="false" :type="statusTagType(run.status)" class="mr-2">
        {{ runStatusLabel(t, run.status) }}
      </n-tag>
      <n-button size="small" @click="backToJobs">{{ t('common.back') }}</n-button>
      <n-button size="small" :loading="loading" @click="loadAll">{{ t('common.refresh') }}</n-button>
      <n-button size="small" type="primary" :disabled="run?.status !== 'success'" @click="openRestore">
        {{ t('runs.actions.restore') }}
      </n-button>
      <n-dropdown
        trigger="click"
        :options="[
          {
            label: t('runs.actions.verify'),
            key: 'verify',
            disabled: run?.status !== 'success',
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
    </page-header>

    <n-spin v-if="loading" size="small" />

    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
      <n-card
        :title="t('runs.detail.overviewTitle')"
        size="small"
        class="app-card lg:col-span-7"
        :bordered="false"
        data-testid="run-detail-overview"
      >
        <div v-if="!run" class="text-sm opacity-70">-</div>
        <div v-else class="space-y-2">
          <n-alert v-if="run.error" type="error" :title="t('runs.columns.error')" :bordered="false">
            {{ run.error }}
          </n-alert>

          <div class="flex flex-wrap items-center gap-2">
            <n-tag v-if="errorsTotal != null && errorsTotal > 0" size="small" type="error" :bordered="false">
              {{ t('runs.detail.errors', { count: errorsTotal }) }}
            </n-tag>
            <n-tag v-if="warningsTotal != null && warningsTotal > 0" size="small" type="warning" :bordered="false">
              {{ t('runs.detail.warnings', { count: warningsTotal }) }}
            </n-tag>
            <n-tag v-if="entriesCount != null" size="small" :bordered="false">
              {{ t('runs.detail.entries', { count: entriesCount }) }}
            </n-tag>
            <n-tag v-if="partsCount != null" size="small" :bordered="false">
              {{ t('runs.detail.parts', { count: partsCount }) }}
            </n-tag>
          </div>

          <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
            <dt class="opacity-70">{{ t('runs.columns.startedAt') }}</dt>
            <dd class="font-mono tabular-nums">{{ formatUnixSeconds(run.started_at) }}</dd>

            <dt class="opacity-70">{{ t('runs.columns.endedAt') }}</dt>
            <dd class="font-mono tabular-nums">{{ formatUnixSeconds(run.ended_at) }}</dd>

            <dt class="opacity-70">{{ t('runs.detail.duration') }}</dt>
            <dd class="font-mono tabular-nums">{{ formatDuration(durationSeconds) }}</dd>

            <dt class="opacity-70">{{ t('runs.detail.target') }}</dt>
            <dd class="min-w-0">
              <div class="flex items-start gap-2 min-w-0">
                <n-tag size="small" :bordered="false" class="shrink-0">{{ targetTypeLabel }}</n-tag>
                <span class="flex-1 min-w-0 font-mono tabular-nums break-all whitespace-normal">{{ targetSummary.location ?? '-' }}</span>
              </div>
            </dd>
          </dl>
        </div>
      </n-card>

      <div class="lg:col-span-5" data-testid="run-detail-progress">
        <run-progress-panel :progress="run?.progress" />
      </div>
    </div>

    <n-card class="app-card" size="small" :bordered="false" data-testid="run-detail-details">
      <n-tabs v-model:value="detailTab" type="line" size="small" animated>
        <n-tab-pane name="events">
          <template #tab>
            <div class="flex items-center gap-2">
              <span>{{ t('runEvents.title') }}</span>
              <n-tag size="tiny" :bordered="false">{{ events.length }}</n-tag>
            </div>
          </template>

          <div class="flex flex-wrap items-center justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0">
              <n-tag size="small" :type="wsStatusTagType(wsStatus)" :bordered="false">
                {{ t(`runEvents.ws.${wsStatus}`) }}
              </n-tag>
              <span class="text-xs opacity-70">{{ events.length }} events</span>
            </div>
          </div>

          <div v-if="events.length === 0" class="mt-3 text-sm opacity-70">{{ t('runEvents.noEvents') }}</div>

          <div
            v-else
            data-testid="run-detail-events-list"
            class="mt-3 max-h-[60vh] overflow-auto rounded-md app-border-subtle divide-y divide-black/5 dark:divide-white/10"
          >
            <button
              v-for="item in events"
              :key="item.seq"
              type="button"
              class="w-full text-left px-3 py-2 flex items-center gap-2 transition hover:bg-black/5 dark:hover:bg-white/5"
              @click="openEventDetails(item)"
            >
              <span class="shrink-0 font-mono tabular-nums text-xs opacity-70 whitespace-nowrap" :title="formatUnixSeconds(item.ts)">
                {{ formatUnixSeconds(item.ts) }}
              </span>
              <n-tag class="shrink-0 w-16 inline-flex justify-center" size="tiny" :type="runEventLevelTagType(item.level)" :bordered="false">
                <span class="block w-full truncate text-center">{{ item.level }}</span>
              </n-tag>
              <span class="min-w-0 flex-1 truncate text-sm">{{ item.message }}</span>
              <span v-if="item.kind && item.kind !== item.message" class="shrink-0 max-w-[12rem] truncate text-xs opacity-70 font-mono">
                {{ item.kind }}
              </span>
              <n-button v-if="item.fields" size="tiny" quaternary @click.stop="openEventDetails(item)">
                {{ t('runEvents.actions.details') }}
              </n-button>
            </button>
          </div>
        </n-tab-pane>

        <n-tab-pane name="operations">
          <template #tab>
            <div class="flex items-center gap-2">
              <span>{{ t('operations.title') }}</span>
              <n-tag size="tiny" :bordered="false">{{ ops.length }}</n-tag>
            </div>
          </template>

          <div v-if="ops.length === 0" class="text-sm opacity-70 py-2" data-testid="run-detail-operations-empty">
            {{ t('runs.detail.noOperations') }}
          </div>
          <n-data-table v-else :columns="opColumns" :data="ops" size="small" :bordered="false" data-testid="run-detail-operations-table" />
        </n-tab-pane>

        <n-tab-pane name="summary" :disabled="!run?.summary">
          <template #tab>
            <span>{{ t('runs.detail.summaryTitle') }}</span>
          </template>

          <div v-if="!run?.summary" class="text-sm opacity-70 py-2">{{ t('common.noData') }}</div>
          <div v-else data-testid="run-detail-summary">
            <div class="flex items-center justify-between gap-3 mb-3">
              <div class="text-sm opacity-70">{{ t('runs.detail.summaryHelp') }}</div>
              <n-button size="small" quaternary @click="copySummaryJson">{{ t('common.copy') }}</n-button>
            </div>

            <div class="grid grid-cols-1 gap-3" :class="summary?.sqlite?.path || summary?.sqlite?.snapshot_name || summary?.vaultwarden?.data_dir ? 'md:grid-cols-2' : 'md:grid-cols-1'">
              <div class="rounded border border-black/5 dark:border-white/10 p-3">
                <div class="text-sm font-medium mb-2">{{ t('runs.detail.summaryHighlights') }}</div>
                <div class="text-xs opacity-70 space-y-1">
                  <div v-if="targetSummary.type || targetSummary.location">
                    {{ t('runs.detail.target') }}:
                    <span class="font-mono tabular-nums">{{ targetTypeLabel }}</span>
                    <span v-if="targetSummary.location" class="font-mono tabular-nums"> · {{ targetSummary.location }}</span>
                  </div>
                  <div v-if="entriesCount != null">{{ t('runs.detail.entries', { count: entriesCount }) }}</div>
                  <div v-if="partsCount != null">{{ t('runs.detail.parts', { count: partsCount }) }}</div>
                  <div v-if="errorsTotal != null && errorsTotal > 0">{{ t('runs.detail.errors', { count: errorsTotal }) }}</div>
                  <div v-if="warningsTotal != null && warningsTotal > 0">{{ t('runs.detail.warnings', { count: warningsTotal }) }}</div>
                </div>
              </div>

              <div v-if="summary?.sqlite?.path || summary?.sqlite?.snapshot_name || summary?.vaultwarden?.data_dir" class="rounded border border-black/5 dark:border-white/10 p-3">
                <div class="text-sm font-medium mb-2">{{ t('runs.detail.summaryDetails') }}</div>
                <div class="text-xs opacity-70 space-y-1">
                  <div v-if="summary?.sqlite?.path">
                    sqlite: <span class="font-mono tabular-nums">{{ summary.sqlite.path }}</span>
                  </div>
                  <div v-if="summary?.sqlite?.snapshot_name">
                    snapshot: <span class="font-mono tabular-nums">{{ summary.sqlite.snapshot_name }}</span>
                  </div>
                  <div v-if="summary?.vaultwarden?.data_dir">
                    vaultwarden: <span class="font-mono tabular-nums">{{ summary.vaultwarden.data_dir }}</span>
                  </div>
                </div>
              </div>
            </div>

            <details class="mt-3 rounded border border-black/5 dark:border-white/10 p-3">
              <summary class="cursor-pointer select-none text-sm font-medium">
                {{ t('runs.detail.rawJson') }}
              </summary>
              <div class="mt-3">
                <n-code :code="formatJson(run.summary)" language="json" />
              </div>
            </details>
          </div>
        </n-tab-pane>
      </n-tabs>
    </n-card>

    <n-modal v-if="isDesktop" v-model:show="eventDetailShow" preset="card" :style="{ width: MODAL_WIDTH.md }" :title="t('runEvents.details.title')">
      <div v-if="eventDetail" class="space-y-3">
        <div class="text-sm opacity-70 flex flex-wrap items-center gap-2">
          <span class="tabular-nums">{{ formatUnixSeconds(eventDetail.ts) }}</span>
          <n-tag size="small" :type="runEventLevelTagType(eventDetail.level)">{{ eventDetail.level }}</n-tag>
          <span class="opacity-70">{{ eventDetail.kind }}</span>
        </div>
        <div class="font-mono text-sm whitespace-pre-wrap break-words">{{ eventDetail.message }}</div>
        <n-code v-if="eventDetail.fields" :code="formatJson(eventDetail.fields)" language="json" />
        <n-space justify="end">
          <n-button @click="eventDetailShow = false">{{ t('common.close') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-drawer v-else v-model:show="eventDetailShow" placement="bottom" height="70vh">
      <n-drawer-content :title="t('runEvents.details.title')" closable>
        <div v-if="eventDetail" class="space-y-3">
          <div class="text-sm opacity-70 flex flex-wrap items-center gap-2">
            <span class="tabular-nums">{{ formatUnixSeconds(eventDetail.ts) }}</span>
            <n-tag size="small" :type="runEventLevelTagType(eventDetail.level)">{{ eventDetail.level }}</n-tag>
            <span class="opacity-70">{{ eventDetail.kind }}</span>
          </div>
          <div class="font-mono text-sm whitespace-pre-wrap break-words">{{ eventDetail.message }}</div>
          <n-code v-if="eventDetail.fields" :code="formatJson(eventDetail.fields)" language="json" />
        </div>
      </n-drawer-content>
    </n-drawer>

    <restore-wizard-modal ref="restoreModal" @started="(id) => openOperation(id)" />
    <verify-wizard-modal ref="verifyModal" @started="(id) => openOperation(id)" />
    <operation-modal ref="opModal" />
  </div>
</template>
