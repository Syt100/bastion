<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { NButton, NDropdown, NIcon, NSpin, NTag, useMessage } from 'naive-ui'
import { EllipsisHorizontal } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import { useRunsStore, type RunEvent, type RunEventConsoleResponse, type RunWorkspaceDetail } from '@/stores/runs'
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

const runsStore = useRunsStore()
const operationsStore = useOperationsStore()

const loading = ref(false)
const consoleLoading = ref(false)
const cancelRunBusy = ref(false)
const detail = ref<RunWorkspaceDetail | null>(null)
const ops = ref<Operation[]>([])
const eventConsole = ref<RunEventConsoleResponse | null>(null)
const eventItems = ref<RunEvent[]>([])
const searchQuery = ref('')
const levelFilter = ref<string | null>(null)
const kindFilter = ref<string | null>(null)
const beforeSeq = ref<number | null>(null)
const afterSeq = ref<number | null>(null)
const anchorMode = ref<'tail' | 'first_error' | `seq:${number}`>('tail')

let pollTimer: number | null = null
const runEventsStream = useRunEventsStream({
  buildUrl: (id, afterSeq) =>
    wsUrl(`/api/runs/${encodeURIComponent(id)}/events/ws?after_seq=${encodeURIComponent(String(afterSeq))}`),
  onEvent: (event) => {
    if (!isFollowingLatest.value) return
    if (eventItems.value.some((item) => item.seq === event.seq)) return
    eventItems.value = [...eventItems.value, event]
    if (eventConsole.value) {
      eventConsole.value = {
        ...eventConsole.value,
        window: {
          ...eventConsole.value.window,
          first_seq: eventConsole.value.window.first_seq ?? event.seq,
          last_seq: event.seq,
          has_newer: false,
        },
      }
    }
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

const isFollowingLatest = computed(() => {
  return !searchQuery.value.trim() && !levelFilter.value && !kindFilter.value && beforeSeq.value == null && afterSeq.value == null && anchorMode.value === 'tail'
})

async function refreshRunAndOps(): Promise<void> {
  const id = props.runId
  const [nextDetail, nextOps] = await Promise.all([runsStore.getWorkspace(id), operationsStore.listRunOperations(id)])
  detail.value = nextDetail
  ops.value = nextOps
}

async function loadEventConsole(): Promise<void> {
  const id = props.runId
  if (!id) return

  consoleLoading.value = true
  try {
    const response = await runsStore.listEventConsole(id, {
      q: searchQuery.value.trim() || undefined,
      levels: levelFilter.value ? [levelFilter.value] : undefined,
      kinds: kindFilter.value ? [kindFilter.value] : undefined,
      limit: 100,
      beforeSeq: beforeSeq.value ?? undefined,
      afterSeq: afterSeq.value ?? undefined,
      anchor: beforeSeq.value == null && afterSeq.value == null ? anchorMode.value : undefined,
    })
    eventConsole.value = response
    eventItems.value = response.items.slice()
    const lastSeq = response.window.last_seq ?? 0
    runEventsStream.setLastSeq(lastSeq)
    if (isFollowingLatest.value) {
      runEventsStream.start(id, lastSeq)
    } else {
      runEventsStream.stop()
    }
  } catch (error) {
    message.error(formatToastError(t('errors.fetchRunEventsFailed'), error, t))
  } finally {
    consoleLoading.value = false
  }
}

async function loadAll(): Promise<void> {
  loading.value = true
  detail.value = null
  ops.value = []
  eventConsole.value = null
  eventItems.value = []
  runEventsStream.stop()
  stopPolling()

  try {
    await refreshRunAndOps()
    await loadEventConsole()

    stopPolling()
    pollTimer = window.setInterval(async () => {
      try {
        const current = detail.value?.run
        const hasRunningOp = ops.value.some((operation) => operation.status === 'running')
        if (current?.status !== 'running' && current?.status !== 'queued' && !hasRunningOp) {
          stopPolling()
          return
        }
        await refreshRunAndOps()
      } catch {
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
  const current = detail.value?.run
  if (!current || !detail.value?.capabilities.can_cancel || cancelRunBusy.value) return
  if (!window.confirm(t('runs.actions.cancelConfirm'))) return

  cancelRunBusy.value = true
  try {
    await runsStore.cancelRun(current.id)
    await refreshRunAndOps()
    message.success(t('messages.runCancelRequested'))
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

function resetConsoleWindow(anchor: 'tail' | 'first_error' | `seq:${number}` = 'tail'): void {
  beforeSeq.value = null
  afterSeq.value = null
  anchorMode.value = anchor
  void loadEventConsole()
}

function loadOlderEvents(): void {
  const firstSeq = eventConsole.value?.window.first_seq
  if (firstSeq == null) return
  beforeSeq.value = firstSeq
  afterSeq.value = null
  anchorMode.value = 'tail'
  void loadEventConsole()
}

function loadNewerEvents(): void {
  const lastSeq = eventConsole.value?.window.last_seq
  if (lastSeq == null) return
  afterSeq.value = lastSeq
  beforeSeq.value = null
  anchorMode.value = 'tail'
  void loadEventConsole()
}

function updateSearch(value: string): void {
  searchQuery.value = value
}

function updateLevel(value: string | null): void {
  levelFilter.value = value
}

function updateKind(value: string | null): void {
  kindFilter.value = value
}

watch(
  () => props.runId,
  (id) => {
    runEventsStream.stop()
    stopPolling()
    searchQuery.value = ''
    levelFilter.value = null
    kindFilter.value = null
    beforeSeq.value = null
    afterSeq.value = null
    anchorMode.value = 'tail'
    if (id) void loadAll()
  },
  { immediate: true },
)

watch([searchQuery, levelFilter, kindFilter], () => {
  if (!props.runId) return
  beforeSeq.value = null
  afterSeq.value = null
  anchorMode.value = 'tail'
  void loadEventConsole()
})

watch(isFollowingLatest, (value) => {
  if (!props.runId) return
  if (value) {
    const lastSeq = eventConsole.value?.window.last_seq ?? 0
    runEventsStream.setLastSeq(lastSeq)
    runEventsStream.start(props.runId, lastSeq)
  } else {
    runEventsStream.stop()
  }
})

onBeforeUnmount(() => {
  runEventsStream.stop()
  stopPolling()
})

const runData = computed(() => detail.value?.run ?? null)
const canRestore = computed(() => detail.value?.capabilities.can_restore ?? false)
const canVerify = computed(() => detail.value?.capabilities.can_verify ?? false)
const resolvedNodeId = computed(() => props.nodeId || runData.value?.node_id || 'hub')
const runCancelRequested = computed(() => runData.value?.cancel_requested_at != null)
const runCancelInProgress = computed(
  () => (runData.value?.status === 'running' || runData.value?.status === 'queued') && (runCancelRequested.value || cancelRunBusy.value),
)
const canCancelRun = computed(() => {
  if (!detail.value?.capabilities.can_cancel || cancelRunBusy.value) return false
  return !runCancelRequested.value
})
const runStatusText = computed(() => {
  const status = runData.value?.status
  if (!status) return ''
  if (runCancelInProgress.value) return t('runs.statuses.canceling')
  return runStatusLabel(t, status)
})

function statusTagType(status: RunWorkspaceDetail['run']['status']): 'success' | 'error' | 'warning' | 'default' {
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
          <n-tag
            v-if="runData?.status"
            data-testid="run-status-tag"
            size="small"
            :bordered="false"
            :type="statusTagType(runData.status)"
          >
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
          v-if="runData && (runData.status === 'queued' || runData.status === 'running')"
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
      class="grid grid-cols-1 gap-3 xl:grid-cols-[minmax(0,420px)_minmax(0,1fr)] xl:items-start"
      data-testid="run-detail-layout"
    >
      <div class="xl:sticky xl:top-3 self-start">
        <RunDetailSummaryCard :detail="detail" :events="eventItems" />
      </div>

      <RunDetailDetailsTabs
        :run-id="props.runId"
        :events="eventItems"
        :console-loading="consoleLoading"
        :window="eventConsole?.window ?? null"
        :locators="eventConsole?.locators ?? null"
        :filters="{
          search: searchQuery,
          level: levelFilter,
          kind: kindFilter,
        }"
        :ops="ops"
        :ws-status="wsStatus"
        :summary="detail?.summary ?? null"
        @update:search="updateSearch"
        @update:level="updateLevel"
        @update:kind="updateKind"
        @load-older="loadOlderEvents"
        @load-newer="loadNewerEvents"
        @jump-latest="resetConsoleWindow('tail')"
        @jump-first-error="resetConsoleWindow('first_error')"
        @open-operation="openOperation"
        @reconnect="reconnectEventsWs"
      />
    </div>

    <RestoreWizardModal ref="restoreModal" @started="(id) => openOperation(id)" />
    <VerifyWizardModal ref="verifyModal" @started="(id) => openOperation(id)" />
    <OperationModal ref="opModal" />
  </div>
</template>
