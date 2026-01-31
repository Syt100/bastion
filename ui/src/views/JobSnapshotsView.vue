<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NButton,
  NCard,
  NCheckbox,
  NCode,
  NDataTable,
  NIcon,
  NInput,
  NModal,
  NPopover,
  NSpace,
  NSpin,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { PinOutline } from '@vicons/ionicons5'

import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import { useJobsStore, type JobDetail, type RunArtifact, type SnapshotDeleteEvent, type SnapshotDeleteTaskDetail } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatToastError } from '@/lib/errors'
import { formatBytes } from '@/lib/format'

const props = defineProps<{
  embedded?: boolean
}>()

const { t } = useI18n()
const message = useMessage()

const route = useRoute()
const router = useRouter()
const ui = useUiStore()
const jobs = useJobsStore()

const isDesktop = useMediaQuery(MQ.mdUp)
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : null))
const nodeIdOrHub = computed(() => nodeId.value ?? 'hub')
const jobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))

const job = ref<JobDetail | null>(null)
const loading = ref<boolean>(false)
const items = ref<RunArtifact[]>([])
const checkedRowKeys = ref<string[]>([])

const deleteConfirmOpen = ref(false)
const deleteConfirmBusy = ref(false)
const deleteConfirmRunIds = ref<string[]>([])
const deleteConfirmForcePinned = ref(false)

const deleteLogOpen = ref(false)
const deleteLogLoading = ref(false)
const deleteLogRunId = ref<string | null>(null)
const deleteLogTask = ref<SnapshotDeleteTaskDetail | null>(null)
const deleteLogEvents = ref<SnapshotDeleteEvent[]>([])
const ignoreReason = ref('')

function openRunDetail(runId: string): void {
  void router.push(`/n/${encodeURIComponent(nodeIdOrHub.value)}/runs/${encodeURIComponent(runId)}`)
}

function formatTarget(row: RunArtifact): string {
  const snap = row.target_snapshot as any
  const target = snap?.target

  if (row.target_type === 'local_dir') {
    const baseDir = typeof target?.base_dir === 'string' ? target.base_dir : ''
    return baseDir ? `${t('snapshots.targets.localDir')}: ${baseDir}` : t('snapshots.targets.localDir')
  }
  if (row.target_type === 'webdav') {
    const baseUrl = typeof target?.base_url === 'string' ? target.base_url : ''
    return baseUrl ? `${t('snapshots.targets.webdav')}: ${baseUrl}` : t('snapshots.targets.webdav')
  }
  return row.target_type
}

function formatStatus(row: RunArtifact): { label: string; type: 'default' | 'success' | 'warning' | 'error' } {
  const s = row.status
  if (s === 'present') return { label: t('snapshots.status.present'), type: 'success' }
  if (s === 'deleting') return { label: t('snapshots.status.deleting'), type: 'warning' }
  if (s === 'deleted') return { label: t('snapshots.status.deleted'), type: 'default' }
  if (s === 'missing') return { label: t('snapshots.status.missing'), type: 'warning' }
  if (s === 'error') return { label: t('snapshots.status.error'), type: 'error' }
  return { label: String(s), type: 'default' }
}

function deleteTaskTagType(status: string): 'success' | 'error' | 'warning' | 'info' | 'default' {
  if (status === 'done') return 'success'
  if (status === 'abandoned') return 'error'
  if (status === 'retrying' || status === 'blocked') return 'warning'
  if (status === 'running') return 'info'
  return 'default'
}

function formatDeleteTaskStatus(status: string): string {
  const map: Record<string, string> = {
    queued: t('snapshots.deleteTaskStatus.queued'),
    running: t('snapshots.deleteTaskStatus.running'),
    retrying: t('snapshots.deleteTaskStatus.retrying'),
    blocked: t('snapshots.deleteTaskStatus.blocked'),
    done: t('snapshots.deleteTaskStatus.done'),
    ignored: t('snapshots.deleteTaskStatus.ignored'),
    abandoned: t('snapshots.deleteTaskStatus.abandoned'),
  }
  return map[status] ?? status
}

function lastErrorLabel(kind?: string | null, msg?: string | null): string {
  const parts: string[] = []
  if (kind) parts.push(kind)
  if (msg) parts.push(msg)
  return parts.join(': ')
}

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function formatDeleteTaskExecutor(row: RunArtifact): string | null {
  if (!row.delete_task) return null
  const id = String(row.node_id ?? '').trim()
  if (!id || id === 'hub') return null
  return t('snapshots.deleteTaskExecutor', { node: id })
}

async function refresh(): Promise<void> {
  const id = jobId.value
  if (!id) return
  loading.value = true
  try {
    job.value = await jobs.getJob(id)
    const res = await jobs.listJobSnapshots(id, { limit: 200 })
    items.value = res.items
  } catch (error) {
    message.error(formatToastError(t('errors.fetchSnapshotsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void refresh()
})

watch(jobId, () => {
  void refresh()
})

const rowById = computed(() => new Map(items.value.map((r) => [r.run_id, r] as const)))

const deleteConfirmRows = computed<RunArtifact[]>(() => {
  const map = rowById.value
  return deleteConfirmRunIds.value.map((id) => map.get(id)).filter((v): v is RunArtifact => !!v)
})

const deleteConfirmPinnedCount = computed<number>(() => deleteConfirmRows.value.filter((r) => r.pinned_at != null).length)

function openDeleteConfirm(runIds: string[]): void {
  const unique = Array.from(new Set(runIds))
  deleteConfirmRunIds.value = unique
  deleteConfirmForcePinned.value = false
  deleteConfirmOpen.value = true
}

function setRowChecked(runId: string, checked: boolean): void {
  const set = new Set(checkedRowKeys.value)
  if (checked) set.add(runId)
  else set.delete(runId)
  checkedRowKeys.value = Array.from(set)
}

function updateCheckedRowKeys(keys: Array<string | number>): void {
  checkedRowKeys.value = keys.map((k) => String(k))
}

function openDeleteSelected(): void {
  if (!checkedRowKeys.value.length) return
  openDeleteConfirm(checkedRowKeys.value)
}

function openDeleteSingle(runId: string): void {
  openDeleteConfirm([runId])
}

async function confirmDelete(): Promise<void> {
  const id = jobId.value
  if (!id) return
  const runIds = deleteConfirmRunIds.value.slice()
  if (!runIds.length) return

  deleteConfirmBusy.value = true
  try {
    const force = deleteConfirmPinnedCount.value > 0 ? deleteConfirmForcePinned.value : false
    if (runIds.length === 1) {
      await jobs.deleteJobSnapshot(id, runIds[0], { force })
    } else {
      await jobs.deleteJobSnapshotsBulk(id, runIds, { force })
    }
    message.success(t('messages.snapshotDeleteQueued'))
    deleteConfirmOpen.value = false
    checkedRowKeys.value = []
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.deleteSnapshotsFailed'), error, t))
  } finally {
    deleteConfirmBusy.value = false
  }
}

async function pinSnapshot(runId: string): Promise<void> {
  const id = jobId.value
  if (!id) return
  try {
    await jobs.pinJobSnapshot(id, runId)
    message.success(t('messages.snapshotPinned'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.pinSnapshotFailed'), error, t))
  }
}

async function unpinSnapshot(runId: string): Promise<void> {
  const id = jobId.value
  if (!id) return
  try {
    await jobs.unpinJobSnapshot(id, runId)
    message.success(t('messages.snapshotUnpinned'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.unpinSnapshotFailed'), error, t))
  }
}

async function openDeleteLog(runId: string): Promise<void> {
  const id = jobId.value
  if (!id) return
  deleteLogOpen.value = true
  deleteLogLoading.value = true
  deleteLogRunId.value = runId
  deleteLogTask.value = null
  deleteLogEvents.value = []
  ignoreReason.value = ''
  try {
    const [task, events] = await Promise.all([
      jobs.getJobSnapshotDeleteTask(id, runId),
      jobs.getJobSnapshotDeleteEvents(id, runId),
    ])
    deleteLogTask.value = task
    deleteLogEvents.value = events
  } catch (error) {
    message.error(formatToastError(t('errors.fetchSnapshotDeleteTaskFailed'), error, t))
    deleteLogOpen.value = false
  } finally {
    deleteLogLoading.value = false
  }
}

async function retryDeleteNow(): Promise<void> {
  const id = jobId.value
  const runId = deleteLogRunId.value
  if (!id || !runId) return
  try {
    await jobs.retryJobSnapshotDeleteNow(id, runId)
    message.success(t('messages.snapshotDeleteRetryQueued'))
    await refresh()
    await openDeleteLog(runId)
  } catch (error) {
    message.error(formatToastError(t('errors.retrySnapshotDeleteFailed'), error, t))
  }
}

async function ignoreDeleteTask(): Promise<void> {
  const id = jobId.value
  const runId = deleteLogRunId.value
  if (!id || !runId) return
  try {
    const reason = ignoreReason.value.trim() || undefined
    await jobs.ignoreJobSnapshotDeleteTask(id, runId, reason)
    message.success(t('messages.snapshotDeleteIgnored'))
    await refresh()
    await openDeleteLog(runId)
  } catch (error) {
    message.error(formatToastError(t('errors.ignoreSnapshotDeleteFailed'), error, t))
  }
}

const columns = computed<DataTableColumns<RunArtifact>>(() => {
  const cols: DataTableColumns<RunArtifact> = [
    {
      type: 'selection',
    },
    {
      title: t('snapshots.columns.endedAt'),
      key: 'ended_at',
      render: (row) => h('span', { class: 'font-mono tabular-nums' }, formatUnixSeconds(row.ended_at)),
    },
    {
      title: t('snapshots.columns.status'),
      key: 'status',
      render: (row) => {
        const s = formatStatus(row)
        const tag = h(NTag, { size: 'small', bordered: false, type: s.type }, { default: () => s.label })
        if (row.pinned_at == null) return tag

        const tip = t('snapshots.pinnedTooltip')
        const pin = h(
          NPopover,
          { trigger: 'hover', placement: 'top', showArrow: false },
          {
            trigger: () =>
              h(
                'span',
                { class: 'inline-flex items-center cursor-default', title: tip },
                [h(NIcon, { component: PinOutline, class: 'text-amber-500 text-[14px]' })],
              ),
            default: () => h('div', { class: 'max-w-[320px] text-sm' }, tip),
          },
        )

        return h('div', { class: 'flex items-center gap-1' }, [tag, pin])
      },
    },
    {
      title: t('snapshots.columns.format'),
      key: 'artifact_format',
      render: (row) => h('span', { class: 'font-mono' }, row.artifact_format ?? '-'),
    },
    {
      title: t('snapshots.columns.target'),
      key: 'target',
      render: (row) => h('span', { class: 'truncate' }, formatTarget(row)),
    },
    {
      title: t('snapshots.columns.source'),
      key: 'source',
      render: (row) => {
        const files = row.source_files ?? null
        const dirs = row.source_dirs ?? null
        const bytes = row.source_bytes ?? null
        const parts: string[] = []
        if (files != null) parts.push(`${files}${t('snapshots.units.files')}`)
        if (dirs != null) parts.push(`${dirs}${t('snapshots.units.dirs')}`)
        if (bytes != null) parts.push(formatBytes(bytes))
        return parts.length ? parts.join(' / ') : '-'
      },
    },
    {
      title: t('snapshots.columns.transfer'),
      key: 'transfer',
      render: (row) => (row.transfer_bytes != null ? formatBytes(row.transfer_bytes) : '-'),
    },
    {
      title: t('snapshots.columns.deleteTask'),
      key: 'delete_task',
      render: (row) => {
        const task = row.delete_task
        if (!task) return '-'
        const label = `${formatDeleteTaskStatus(task.status)} (${task.attempts})`
        const executor = formatDeleteTaskExecutor(row)
        const err = lastErrorLabel(task.last_error_kind, task.last_error)
        return h(
          'div',
          { class: 'min-w-0' },
          [
            h(NTag, { size: 'small', bordered: false, type: deleteTaskTagType(task.status) }, { default: () => label }),
            executor ? h('div', { class: 'text-xs opacity-70 truncate mt-0.5' }, executor) : null,
            err ? h('div', { class: 'text-xs opacity-70 truncate mt-0.5' }, err) : null,
          ].filter(Boolean),
        )
      },
    },
    {
      title: t('snapshots.columns.actions'),
      key: 'actions',
      render: (row) =>
        h(
          NSpace,
          { size: 8 },
          {
            default: () => [
              h(
                NButton,
                { size: 'small', onClick: () => openRunDetail(row.run_id) },
                { default: () => t('snapshots.actions.viewRun') },
              ),
              row.status === 'present'
                ? h(
                    NButton,
                    {
                      size: 'small',
                      quaternary: true,
                      onClick: () => (row.pinned_at != null ? unpinSnapshot(row.run_id) : pinSnapshot(row.run_id)),
                    },
                    { default: () => (row.pinned_at != null ? t('snapshots.actions.unpin') : t('snapshots.actions.pin')) },
                  )
                : null,
              row.status === 'present'
                ? h(
                    NButton,
                    { size: 'small', type: 'error', onClick: () => openDeleteSingle(row.run_id) },
                    { default: () => t('snapshots.actions.delete') },
                  )
                : null,
              row.delete_task
                ? h(
                    NButton,
                    { size: 'small', onClick: () => openDeleteLog(row.run_id) },
                    { default: () => t('snapshots.actions.deleteLog') },
                  )
                : null,
            ],
          },
        ),
    },
  ]
  return cols
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      v-if="!props.embedded"
      :title="t('snapshots.title')"
      :subtitle="job ? `${t('snapshots.subtitlePrefix')}: ${job.name}` : t('snapshots.subtitle')"
    >
      <template #prefix>
        <NodeContextTag :node-id="nodeIdOrHub" />
      </template>
      <n-button v-if="checkedRowKeys.length" type="error" @click="openDeleteSelected">
        {{ t('snapshots.actions.deleteSelected', { count: checkedRowKeys.length }) }}
      </n-button>
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button @click="$router.push(`/n/${encodeURIComponent(nodeIdOrHub)}/jobs`)">{{ t('common.return') }}</n-button>
    </PageHeader>

    <div v-else class="flex items-center justify-end gap-2">
      <n-button v-if="checkedRowKeys.length" type="error" @click="openDeleteSelected">
        {{ t('snapshots.actions.deleteSelected', { count: checkedRowKeys.length }) }}
      </n-button>
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
    </div>

    <div v-if="!isDesktop" class="space-y-3">
      <AppEmptyState v-if="loading && items.length === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!loading && items.length === 0" :title="t('common.noData')" />

      <n-card v-for="row in items" :key="row.run_id" size="small" class="app-card">
        <template #header>
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="text-sm font-mono tabular-nums">{{ formatUnixSeconds(row.ended_at) }}</div>
              <div class="text-xs opacity-70 mt-0.5 truncate">{{ formatTarget(row) }}</div>
            </div>
            <div class="flex items-center gap-2">
              <n-checkbox
                :checked="checkedRowKeys.includes(row.run_id)"
                @update:checked="(v) => setRowChecked(row.run_id, v)"
              />
              <n-popover v-if="row.pinned_at != null" trigger="hover" placement="top" :show-arrow="false">
                <template #trigger>
                  <span class="inline-flex items-center cursor-default" :title="t('snapshots.pinnedTooltip')">
                    <n-icon :component="PinOutline" class="text-amber-500 text-[14px]" />
                  </span>
                </template>
                <div class="max-w-[320px] text-sm">{{ t('snapshots.pinnedTooltip') }}</div>
              </n-popover>
              <n-tag size="small" :bordered="false" :type="formatStatus(row).type">{{ formatStatus(row).label }}</n-tag>
            </div>
          </div>
        </template>

        <div class="text-sm">
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('snapshots.columns.format') }}</div>
            <div class="font-mono">{{ row.artifact_format ?? '-' }}</div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('snapshots.columns.source') }}</div>
            <div class="text-right">
              <span v-if="row.source_files != null">{{ row.source_files }}{{ t('snapshots.units.files') }}</span>
              <span v-else>-</span>
              <span v-if="row.source_dirs != null"> / {{ row.source_dirs }}{{ t('snapshots.units.dirs') }}</span>
              <span v-if="row.source_bytes != null"> / {{ formatBytes(row.source_bytes) }}</span>
            </div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('snapshots.columns.transfer') }}</div>
            <div class="text-right">{{ row.transfer_bytes != null ? formatBytes(row.transfer_bytes) : '-' }}</div>
          </div>
          <div v-if="row.delete_task" class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('snapshots.columns.deleteTask') }}</div>
            <div class="text-right">
              <n-tag size="small" :bordered="false" :type="deleteTaskTagType(row.delete_task.status)">
                {{ formatDeleteTaskStatus(row.delete_task.status) }} ({{ row.delete_task.attempts }})
              </n-tag>
              <div v-if="formatDeleteTaskExecutor(row)" class="text-xs opacity-70 mt-0.5">
                {{ formatDeleteTaskExecutor(row) }}
              </div>
              <div v-if="row.delete_task.last_error || row.delete_task.last_error_kind" class="text-xs opacity-70 mt-0.5">
                {{ lastErrorLabel(row.delete_task.last_error_kind, row.delete_task.last_error) }}
              </div>
            </div>
          </div>
        </div>

        <template #footer>
          <div class="flex justify-end gap-2">
            <n-button size="small" @click="openRunDetail(row.run_id)">{{ t('snapshots.actions.viewRun') }}</n-button>
            <n-button
              v-if="row.status === 'present'"
              size="small"
              quaternary
              @click="row.pinned_at != null ? unpinSnapshot(row.run_id) : pinSnapshot(row.run_id)"
            >
              {{ row.pinned_at != null ? t('snapshots.actions.unpin') : t('snapshots.actions.pin') }}
            </n-button>
            <n-button v-if="row.status === 'present'" size="small" type="error" @click="openDeleteSingle(row.run_id)">
              {{ t('snapshots.actions.delete') }}
            </n-button>
            <n-button v-if="row.delete_task" size="small" @click="openDeleteLog(row.run_id)">
              {{ t('snapshots.actions.deleteLog') }}
            </n-button>
          </div>
        </template>
      </n-card>
    </div>

    <div v-else>
      <n-card class="app-card">
        <div class="overflow-x-auto">
          <n-data-table
            :loading="loading"
            :columns="columns"
            :data="items"
            :row-key="(row) => row.run_id"
            :checked-row-keys="checkedRowKeys"
            @update:checked-row-keys="updateCheckedRowKeys"
          />
        </div>
      </n-card>
    </div>

    <n-modal v-model:show="deleteConfirmOpen" :mask-closable="!deleteConfirmBusy" preset="card" :style="{ width: isDesktop ? '720px' : '92vw' }">
      <template #header>{{ t('snapshots.deleteConfirm.title') }}</template>
      <div class="space-y-3">
        <div class="text-sm opacity-80">{{ t('snapshots.deleteConfirm.subtitle', { count: deleteConfirmRows.length }) }}</div>
        <div
          v-if="deleteConfirmPinnedCount > 0"
          class="rounded border border-amber-200/60 dark:border-amber-700/60 bg-amber-50/40 dark:bg-amber-900/10 p-3 space-y-2"
        >
          <div class="text-sm font-medium">{{ t('snapshots.deleteConfirm.pinnedWarningTitle') }}</div>
          <div class="text-sm opacity-80">{{ t('snapshots.deleteConfirm.pinnedWarning', { count: deleteConfirmPinnedCount }) }}</div>
          <n-checkbox :checked="deleteConfirmForcePinned" @update:checked="(v) => (deleteConfirmForcePinned = v)">
            {{ t('snapshots.deleteConfirm.forcePinnedLabel') }}
          </n-checkbox>
        </div>
        <div class="max-h-64 overflow-y-auto rounded border border-slate-200/60 dark:border-slate-700/60">
          <div
            v-for="row in deleteConfirmRows"
            :key="row.run_id"
            :class="[
              'px-3 py-2 border-b border-slate-200/60 dark:border-slate-700/60 last:border-b-0',
              row.pinned_at != null ? 'bg-amber-50/40 dark:bg-amber-900/10' : '',
            ]"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <div class="font-mono text-xs tabular-nums">{{ formatUnixSeconds(row.ended_at) }}</div>
                <div class="text-xs opacity-70 truncate mt-0.5">{{ formatTarget(row) }}</div>
              </div>
              <div class="flex items-center gap-1 text-xs font-mono opacity-70">
                <n-icon v-if="row.pinned_at != null" :component="PinOutline" class="text-amber-500 text-[14px]" />
                <span>{{ row.run_id.slice(0, 8) }}…</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <div class="flex justify-end gap-2">
          <n-button :disabled="deleteConfirmBusy" @click="deleteConfirmOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button
            type="error"
            :loading="deleteConfirmBusy"
            :disabled="deleteConfirmBusy || (deleteConfirmPinnedCount > 0 && !deleteConfirmForcePinned)"
            @click="confirmDelete"
          >
            {{ t('snapshots.actions.confirmDelete') }}
          </n-button>
        </div>
      </template>
    </n-modal>

    <n-modal v-model:show="deleteLogOpen" preset="card" :style="{ width: isDesktop ? '900px' : '92vw' }">
      <template #header>{{ t('snapshots.deleteLog.title') }}</template>
      <template #header-extra>
        <div class="flex gap-2">
          <n-button size="small" :disabled="deleteLogLoading || !deleteLogTask || deleteLogTask.status === 'running'" @click="retryDeleteNow">
            {{ t('snapshots.actions.retryNow') }}
          </n-button>
          <n-button size="small" :disabled="deleteLogLoading || !deleteLogTask || deleteLogTask.status === 'running'" @click="ignoreDeleteTask">
            {{ t('snapshots.actions.ignore') }}
          </n-button>
        </div>
      </template>

      <n-spin :show="deleteLogLoading">
        <div v-if="deleteLogTask" class="space-y-4">
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div class="opacity-70">{{ t('snapshots.deleteLog.status') }}</div>
            <div>
              <n-tag size="small" :bordered="false" :type="deleteTaskTagType(deleteLogTask.status)">
                {{ formatDeleteTaskStatus(deleteLogTask.status) }}
              </n-tag>
            </div>
            <div class="opacity-70">{{ t('snapshots.deleteLog.attempts') }}</div>
            <div class="font-mono tabular-nums">{{ deleteLogTask.attempts }}</div>
            <div class="opacity-70">{{ t('snapshots.deleteLog.lastError') }}</div>
            <div class="min-w-0">
              <div class="truncate">{{ lastErrorLabel(deleteLogTask.last_error_kind, deleteLogTask.last_error) || '-' }}</div>
            </div>
          </div>

          <div>
            <div class="text-sm font-medium mb-2">{{ t('snapshots.deleteLog.events') }}</div>
            <div v-if="deleteLogEvents.length === 0" class="text-sm opacity-70">{{ t('common.noData') }}</div>
            <div v-else class="space-y-2">
              <div
                v-for="e in deleteLogEvents"
                :key="e.seq"
                class="rounded border border-slate-200/60 dark:border-slate-700/60 px-3 py-2"
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <div class="text-xs font-mono tabular-nums">{{ formatUnixSeconds(e.ts) }}</div>
                    <div class="text-sm mt-0.5">{{ e.message }}</div>
                    <div class="text-xs opacity-70 mt-0.5">{{ e.kind }}</div>
                  </div>
                  <n-tag size="small" :bordered="false" :type="e.level === 'error' ? 'error' : e.level === 'warn' ? 'warning' : 'default'">
                    {{ e.level }}
                  </n-tag>
                </div>
                <n-code v-if="e.fields" class="mt-2" language="json" :code="formatJson(e.fields)" />
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <div class="text-sm font-medium">{{ t('snapshots.actions.ignore') }}</div>
            <n-input v-model:value="ignoreReason" type="text" :placeholder="t('snapshots.deleteLog.ignorePlaceholder')" />
          </div>
        </div>
      </n-spin>
    </n-modal>
  </div>
</template>
