<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import {
  NButton,
  NCard,
  NCode,
  NDataTable,
  NDrawer,
  NDrawerContent,
  NInput,
  NModal,
  NPopover,
  NSelect,
  NSpace,
  NSpin,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import {
  useIncompleteCleanupStore,
  type CleanupTargetType,
  type CleanupTaskListItem,
  type CleanupTaskStatus,
  type GetCleanupTaskResponse,
} from '@/stores/incompleteCleanup'
import { useUiStore } from '@/stores/ui'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { useLatestRequest } from '@/lib/latest'
import { MODAL_WIDTH } from '@/lib/modal'
import { copyText } from '@/lib/clipboard'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const cleanup = useIncompleteCleanupStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const loading = ref(false)
const helpOpen = ref(false)

const statusFilter = ref<CleanupTaskStatus[] | null>([])
const targetFilter = ref<CleanupTargetType[] | null>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const items = ref<CleanupTaskListItem[]>([])

const latest = useLatestRequest()

function isAbortError(error: unknown): boolean {
  if (!error || typeof error !== 'object') return false
  if (typeof DOMException !== 'undefined' && error instanceof DOMException) return error.name === 'AbortError'
  return 'name' in error && (error as { name?: unknown }).name === 'AbortError'
}

const statusOptions = computed(() => [
  { label: t('settings.maintenance.cleanup.status.queued'), value: 'queued' },
  { label: t('settings.maintenance.cleanup.status.running'), value: 'running' },
  { label: t('settings.maintenance.cleanup.status.retrying'), value: 'retrying' },
  { label: t('settings.maintenance.cleanup.status.blocked'), value: 'blocked' },
  { label: t('settings.maintenance.cleanup.status.done'), value: 'done' },
  { label: t('settings.maintenance.cleanup.status.ignored'), value: 'ignored' },
  { label: t('settings.maintenance.cleanup.status.abandoned'), value: 'abandoned' },
])

const targetOptions = computed(() => [
  { label: t('settings.maintenance.cleanup.target.webdav'), value: 'webdav' },
  { label: t('settings.maintenance.cleanup.target.localDir'), value: 'local_dir' },
])

function formatTarget(target: CleanupTargetType): string {
  return target === 'webdav'
    ? t('settings.maintenance.cleanup.target.webdav')
    : t('settings.maintenance.cleanup.target.localDir')
}

function statusTagType(status: string): 'success' | 'error' | 'warning' | 'info' | 'default' {
  if (status === 'done') return 'success'
  if (status === 'abandoned') return 'error'
  if (status === 'retrying' || status === 'blocked') return 'warning'
  if (status === 'running') return 'info'
  return 'default'
}

function formatStatus(status: string): string {
  const map: Record<string, string> = {
    queued: t('settings.maintenance.cleanup.status.queued'),
    running: t('settings.maintenance.cleanup.status.running'),
    retrying: t('settings.maintenance.cleanup.status.retrying'),
    blocked: t('settings.maintenance.cleanup.status.blocked'),
    done: t('settings.maintenance.cleanup.status.done'),
    ignored: t('settings.maintenance.cleanup.status.ignored'),
    abandoned: t('settings.maintenance.cleanup.status.abandoned'),
  }
  return map[status] ?? status
}

function formatRunId(id: string): string {
  if (id.length <= 12) return id
  return `${id.slice(0, 8)}…${id.slice(-4)}`
}

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function lastErrorLabel(kind: string | null, message: string | null): string {
  const parts = []
  if (kind) parts.push(kind)
  if (message) parts.push(message)
  return parts.join(': ')
}

async function copyToClipboard(value: string): Promise<void> {
  const ok = await copyText(value)
  if (ok) {
    message.success(t('messages.copied'))
  } else {
    message.error(t('errors.copyFailed'))
  }
}

async function refresh(): Promise<void> {
  const req = latest.next()
  loading.value = true
  try {
    const statuses = statusFilter.value ?? []
    const targetTypes = targetFilter.value ?? []
    const res = await cleanup.listTasks({
      status: statuses.length ? statuses : undefined,
      targetType: targetTypes.length ? targetTypes : undefined,
      page: page.value,
      pageSize: pageSize.value,
      signal: req.signal,
    })
    if (req.isStale()) return
    items.value = res.items
    total.value = res.total
    page.value = res.page
    pageSize.value = res.page_size
  } catch (e) {
    if (req.isStale() || isAbortError(e)) return
    message.error(formatToastError(t('errors.fetchIncompleteCleanupTasksFailed'), e, t))
  } finally {
    if (req.isStale()) return
    loading.value = false
  }
}

async function retryNow(runId: string): Promise<void> {
  try {
    await cleanup.retryNow(runId)
    message.success(t('messages.incompleteCleanupRetryScheduled'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.incompleteCleanupRetryFailed'), e, t))
  }
}

const ignoreOpen = ref(false)
const ignoreSaving = ref(false)
const ignoreRunId = ref<string | null>(null)
const ignoreReason = ref('')

function openIgnore(runId: string): void {
  ignoreRunId.value = runId
  ignoreReason.value = ''
  ignoreOpen.value = true
}

async function confirmIgnore(): Promise<void> {
  const runId = ignoreRunId.value
  if (!runId) return

  ignoreSaving.value = true
  try {
    const reason = ignoreReason.value.trim() || undefined
    await cleanup.ignore(runId, reason)
    message.success(t('messages.incompleteCleanupIgnored'))
    ignoreOpen.value = false
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.incompleteCleanupIgnoreFailed'), e, t))
  } finally {
    ignoreSaving.value = false
  }
}

async function unignore(runId: string): Promise<void> {
  try {
    await cleanup.unignore(runId)
    message.success(t('messages.incompleteCleanupUnignored'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.incompleteCleanupUnignoreFailed'), e, t))
  }
}

const detailOpen = ref(false)
const detailLoading = ref(false)
const detail = ref<GetCleanupTaskResponse | null>(null)

async function openDetails(runId: string): Promise<void> {
  detailOpen.value = true
  detailLoading.value = true
  detail.value = null
  try {
    detail.value = await cleanup.getTask(runId)
  } catch (e) {
    message.error(formatToastError(t('errors.fetchIncompleteCleanupTaskFailed'), e, t))
    detailOpen.value = false
  } finally {
    detailLoading.value = false
  }
}

function eventLevelTagType(level: string): 'success' | 'error' | 'warning' | 'default' {
  if (level === 'error') return 'error'
  if (level === 'warn' || level === 'warning') return 'warning'
  if (level === 'info') return 'success'
  return 'default'
}

watch([statusFilter, targetFilter], () => {
  page.value = 1
  void refresh()
})

onMounted(refresh)

const columns = computed<DataTableColumns<CleanupTaskListItem>>(() => [
  {
    title: t('settings.maintenance.cleanup.columns.job'),
    key: 'job_name',
    minWidth: 160,
    render: (row) =>
      h('div', { class: 'min-w-0 truncate', title: row.job_name }, row.job_name),
  },
  {
    title: t('settings.maintenance.cleanup.columns.target'),
    key: 'target_type',
    minWidth: 100,
    render: (row) => formatTarget(row.target_type),
  },
  {
    title: t('settings.maintenance.cleanup.columns.status'),
    key: 'status',
    minWidth: 100,
    render: (row) =>
      h(
        NTag,
        { size: 'small', type: statusTagType(row.status), bordered: false },
        { default: () => formatStatus(row.status) },
      ),
  },
  {
    title: t('settings.maintenance.cleanup.columns.nextAttempt'),
    key: 'next_attempt_at',
    minWidth: 170,
    render: (row) => formatUnixSeconds(row.next_attempt_at),
  },
  {
    title: t('settings.maintenance.cleanup.columns.updatedAt'),
    key: 'updated_at',
    minWidth: 170,
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('settings.maintenance.cleanup.columns.lastError'),
    key: 'last_error',
    minWidth: 220,
    render: (row) =>
      row.last_error || row.last_error_kind
        ? (() => {
            const full = lastErrorLabel(row.last_error_kind ?? null, row.last_error ?? null)
            return h(
              NPopover,
              { trigger: 'hover', placement: 'top-start', showArrow: false },
              {
                trigger: () =>
                  h(
                    'div',
                    {
                      class: 'min-w-0 w-full flex items-center gap-2 cursor-pointer',
                      title: full,
                      onClick: () => void openDetails(row.run_id),
                    },
                    [
                      row.last_error_kind
                        ? h(
                            NTag,
                            { size: 'small', type: 'error', bordered: false },
                            { default: () => row.last_error_kind },
                          )
                        : null,
                      h('span', { class: 'min-w-0 flex-1 truncate' }, row.last_error ?? ''),
                    ],
                  ),
                default: () =>
                  h(
                    'div',
                    { class: 'max-w-[640px] whitespace-pre-wrap break-words text-sm' },
                    full || '-',
                  ),
              },
            )
          })()
        : '-',
  },
  {
    title: t('settings.maintenance.cleanup.columns.actions'),
    key: 'actions',
    minWidth: 220,
    align: 'right',
    render: (row) =>
      h(
        NSpace,
        { size: 8, justify: 'end' },
        {
          default: () => [
            h(
              NButton,
              { size: 'small', onClick: () => void openDetails(row.run_id) },
              { default: () => t('common.more') },
            ),
            h(
              NButton,
              {
                size: 'small',
                disabled: row.status === 'running' || row.status === 'done',
                onClick: () => void retryNow(row.run_id),
              },
              { default: () => t('settings.maintenance.cleanup.actions.retryNow') },
            ),
            row.status === 'ignored'
              ? h(
                  NButton,
                  { size: 'small', onClick: () => void unignore(row.run_id) },
                  { default: () => t('settings.maintenance.cleanup.actions.unignore') },
                )
              : h(
                  NButton,
                  {
                    size: 'small',
                    disabled: row.status === 'running' || row.status === 'done',
                    type: 'warning',
                    tertiary: true,
                    onClick: () => openIgnore(row.run_id),
                  },
                  { default: () => t('settings.maintenance.cleanup.actions.ignore') },
                ),
          ],
        },
      ),
  },
])

const statusHelpItems = computed(() => [
  { status: 'queued', body: t('settings.maintenance.cleanup.statusHelp.queued') },
  { status: 'running', body: t('settings.maintenance.cleanup.statusHelp.running') },
  { status: 'retrying', body: t('settings.maintenance.cleanup.statusHelp.retrying') },
  { status: 'blocked', body: t('settings.maintenance.cleanup.statusHelp.blocked') },
  { status: 'done', body: t('settings.maintenance.cleanup.statusHelp.done') },
  { status: 'ignored', body: t('settings.maintenance.cleanup.statusHelp.ignored') },
  { status: 'abandoned', body: t('settings.maintenance.cleanup.statusHelp.abandoned') },
])

const actionHelpItems = computed(() => [
  { key: 'more', label: t('common.more'), body: t('settings.maintenance.cleanup.actionHelp.more') },
  {
    key: 'retryNow',
    label: t('settings.maintenance.cleanup.actions.retryNow'),
    body: t('settings.maintenance.cleanup.actionHelp.retryNow'),
  },
  { key: 'ignore', label: t('settings.maintenance.cleanup.actions.ignore'), body: t('settings.maintenance.cleanup.actionHelp.ignore') },
  {
    key: 'unignore',
    label: t('settings.maintenance.cleanup.actions.unignore'),
    body: t('settings.maintenance.cleanup.actionHelp.unignore'),
  },
])
</script>

<template>
  <n-card class="app-card" :title="t('settings.maintenance.cleanup.title')">
    <div class="space-y-4">
      <div class="flex flex-col md:flex-row md:flex-wrap md:items-center gap-2">
        <div class="w-full md:w-56 md:flex-none">
          <n-select
            v-model:value="statusFilter"
            multiple
            clearable
            max-tag-count="responsive"
            :placeholder="t('settings.maintenance.cleanup.status.all')"
            :options="statusOptions"
            class="w-full"
          />
        </div>
        <div class="w-full md:w-56 md:flex-none">
          <n-select
            v-model:value="targetFilter"
            multiple
            clearable
            max-tag-count="responsive"
            :placeholder="t('settings.maintenance.cleanup.target.all')"
            :options="targetOptions"
            class="w-full"
          />
        </div>
        <div class="flex items-center gap-2 w-full md:w-auto">
          <n-button class="flex-1 md:flex-none" :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
          <n-button size="small" circle @click="helpOpen = true">?</n-button>
        </div>
      </div>

      <div v-if="!isDesktop" class="space-y-3">
        <n-card
          v-for="row in items"
          :key="row.run_id"
          size="small"
          class="app-card"
        >
          <template #header>
            <div class="flex items-center justify-between gap-3">
              <div class="font-medium truncate">{{ row.job_name }}</div>
              <n-tag size="small" :type="statusTagType(row.status)" :bordered="false">
                {{ formatStatus(row.status) }}
              </n-tag>
            </div>
          </template>

          <div class="text-xs opacity-70 space-y-1">
            <div>{{ t('settings.maintenance.cleanup.columns.runId') }}: {{ formatRunId(row.run_id) }}</div>
            <div>{{ t('settings.maintenance.cleanup.columns.node') }}: {{ row.node_id }}</div>
            <div>{{ t('settings.maintenance.cleanup.columns.target') }}: {{ formatTarget(row.target_type) }}</div>
            <div>{{ t('settings.maintenance.cleanup.columns.attempts') }}: {{ row.attempts }}</div>
            <div>{{ t('settings.maintenance.cleanup.columns.nextAttempt') }}: {{ formatUnixSeconds(row.next_attempt_at) }}</div>
            <div>{{ t('settings.maintenance.cleanup.columns.updatedAt') }}: {{ formatUnixSeconds(row.updated_at) }}</div>
            <div v-if="row.last_error">{{ t('settings.maintenance.cleanup.columns.lastError') }}: {{ row.last_error }}</div>
          </div>

          <div class="mt-3 flex flex-wrap items-center justify-end gap-2">
            <n-button size="small" @click="openDetails(row.run_id)">{{ t('common.more') }}</n-button>
            <n-button size="small" :disabled="row.status === 'running' || row.status === 'done'" @click="retryNow(row.run_id)">
              {{ t('settings.maintenance.cleanup.actions.retryNow') }}
            </n-button>
            <n-button
              v-if="row.status !== 'ignored'"
              size="small"
              type="warning"
              tertiary
              :disabled="row.status === 'running' || row.status === 'done'"
              @click="openIgnore(row.run_id)"
            >
              {{ t('settings.maintenance.cleanup.actions.ignore') }}
            </n-button>
            <n-button v-else size="small" @click="unignore(row.run_id)">
              {{ t('settings.maintenance.cleanup.actions.unignore') }}
            </n-button>
          </div>
        </n-card>
      </div>

      <div v-else class="overflow-x-auto">
        <n-data-table table-layout="fixed" :loading="loading" :columns="columns" :data="items" />
      </div>

      <div class="flex items-center justify-between text-sm">
        <div class="opacity-70">{{ t('settings.maintenance.cleanup.total', { total }) }}</div>
        <div class="flex items-center gap-2">
          <n-button size="small" :disabled="page <= 1" @click="page -= 1; refresh()">{{ t('common.back') }}</n-button>
          <div class="text-xs opacity-70">{{ page }}</div>
          <n-button size="small" :disabled="page * pageSize >= total" @click="page += 1; refresh()">
            {{ t('common.next') }}
          </n-button>
        </div>
      </div>
    </div>
  </n-card>

  <n-modal v-model:show="ignoreOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('settings.maintenance.cleanup.ignoreTitle')">
    <div class="space-y-3">
      <div class="text-sm opacity-80">{{ t('settings.maintenance.cleanup.ignoreHelp') }}</div>
      <n-input v-model:value="ignoreReason" type="textarea" :placeholder="t('settings.maintenance.cleanup.ignorePlaceholder')" />
      <div class="flex items-center justify-end gap-2">
        <n-button :disabled="ignoreSaving" @click="ignoreOpen = false">{{ t('common.cancel') }}</n-button>
        <n-button type="warning" :loading="ignoreSaving" @click="confirmIgnore">{{ t('settings.maintenance.cleanup.actions.ignore') }}</n-button>
      </div>
    </div>
  </n-modal>

  <n-modal
    v-if="isDesktop"
    v-model:show="detailOpen"
    preset="card"
    :style="{ width: MODAL_WIDTH.lg }"
    :title="t('settings.maintenance.cleanup.detailTitle')"
  >
    <div v-if="detailLoading" class="py-10 flex justify-center">
      <n-spin />
    </div>
    <div v-else-if="detail" class="space-y-4">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="font-medium truncate">{{ detail.task.job_name }}</div>
        <n-tag size="small" :type="statusTagType(detail.task.status)" :bordered="false">
          {{ formatStatus(detail.task.status) }}
        </n-tag>
      </div>

      <div class="text-sm space-y-2">
        <div class="grid grid-cols-[auto_1fr_auto] gap-x-2 gap-y-1 items-start">
          <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.runId') }}:</div>
          <div class="min-w-0 break-all">{{ detail.task.run_id }}</div>
          <n-button size="tiny" tertiary @click="copyToClipboard(detail.task.run_id)">{{ t('common.copy') }}</n-button>

          <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.node') }}:</div>
          <div class="min-w-0 break-all">{{ detail.task.node_id }}</div>
          <n-button size="tiny" tertiary @click="copyToClipboard(detail.task.node_id)">{{ t('common.copy') }}</n-button>

          <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.target') }}:</div>
          <div class="min-w-0">{{ formatTarget(detail.task.target_type) }}</div>
          <div />

          <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.attempts') }}:</div>
          <div class="min-w-0">{{ detail.task.attempts }}</div>
          <div />

          <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.nextAttempt') }}:</div>
          <div class="min-w-0">{{ formatUnixSeconds(detail.task.next_attempt_at) }}</div>
          <div />

          <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.updatedAt') }}:</div>
          <div class="min-w-0">{{ formatUnixSeconds(detail.task.updated_at) }}</div>
          <div />
        </div>

        <div v-if="detail.task.last_error || detail.task.last_error_kind" class="space-y-1">
          <div class="flex items-center justify-between gap-2">
            <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.lastError') }}:</div>
            <n-button
              size="tiny"
              tertiary
              :disabled="!(detail.task.last_error || detail.task.last_error_kind)"
              @click="copyToClipboard(lastErrorLabel(detail.task.last_error_kind ?? null, detail.task.last_error ?? null))"
            >
              {{ t('common.copy') }}
            </n-button>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <n-tag v-if="detail.task.last_error_kind" size="small" type="error" :bordered="false">
              {{ detail.task.last_error_kind }}
            </n-tag>
            <div class="text-sm whitespace-pre-wrap break-words">{{ detail.task.last_error }}</div>
          </div>
        </div>

        <div v-if="detail.task.ignore_reason" class="text-sm">
          <span class="opacity-70">{{ t('settings.maintenance.cleanup.columns.ignoreReason') }}:</span>
          {{ detail.task.ignore_reason }}
        </div>
      </div>

      <div>
        <div class="text-sm font-medium mb-2">{{ t('settings.maintenance.cleanup.targetSnapshot') }}</div>
        <n-code :code="formatJson(detail.task.target_snapshot)" language="json" show-line-numbers />
      </div>

      <div>
        <div class="text-sm font-medium mb-2">{{ t('settings.maintenance.cleanup.eventsTitle') }}</div>
        <div v-if="detail.events.length === 0" class="text-sm opacity-70">{{ t('common.noData') }}</div>
        <div v-else class="space-y-2">
          <n-card v-for="e in detail.events" :key="e.seq" size="small" class="app-card">
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <div class="text-sm font-medium truncate">{{ e.kind }}</div>
                <div class="text-xs opacity-70 mt-0.5">{{ formatUnixSeconds(e.ts) }}</div>
              </div>
              <n-tag size="small" :type="eventLevelTagType(e.level)" :bordered="false">{{ e.level }}</n-tag>
            </div>
            <div class="text-sm mt-2">{{ e.message }}</div>
            <div v-if="e.fields" class="mt-2">
              <n-code :code="formatJson(e.fields)" language="json" />
            </div>
          </n-card>
        </div>
      </div>
    </div>
  </n-modal>

  <n-drawer v-else v-model:show="detailOpen" placement="bottom" height="80vh">
    <n-drawer-content :title="t('settings.maintenance.cleanup.detailTitle')" closable>
      <div v-if="detailLoading" class="py-10 flex justify-center">
        <n-spin />
      </div>
      <div v-else-if="detail" class="space-y-4">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div class="font-medium truncate">{{ detail.task.job_name }}</div>
          <n-tag size="small" :type="statusTagType(detail.task.status)" :bordered="false">
            {{ formatStatus(detail.task.status) }}
          </n-tag>
        </div>

        <div class="text-sm space-y-2">
          <div class="grid grid-cols-[auto_1fr_auto] gap-x-2 gap-y-1 items-start">
            <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.runId') }}:</div>
            <div class="min-w-0 break-all">{{ detail.task.run_id }}</div>
            <n-button size="tiny" tertiary @click="copyToClipboard(detail.task.run_id)">{{ t('common.copy') }}</n-button>

            <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.node') }}:</div>
            <div class="min-w-0 break-all">{{ detail.task.node_id }}</div>
            <n-button size="tiny" tertiary @click="copyToClipboard(detail.task.node_id)">{{ t('common.copy') }}</n-button>

            <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.target') }}:</div>
            <div class="min-w-0">{{ formatTarget(detail.task.target_type) }}</div>
            <div />

            <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.attempts') }}:</div>
            <div class="min-w-0">{{ detail.task.attempts }}</div>
            <div />

            <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.nextAttempt') }}:</div>
            <div class="min-w-0">{{ formatUnixSeconds(detail.task.next_attempt_at) }}</div>
            <div />

            <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.updatedAt') }}:</div>
            <div class="min-w-0">{{ formatUnixSeconds(detail.task.updated_at) }}</div>
            <div />
          </div>

          <div v-if="detail.task.last_error || detail.task.last_error_kind" class="space-y-1">
            <div class="flex items-center justify-between gap-2">
              <div class="opacity-70">{{ t('settings.maintenance.cleanup.columns.lastError') }}:</div>
              <n-button
                size="tiny"
                tertiary
                :disabled="!(detail.task.last_error || detail.task.last_error_kind)"
                @click="copyToClipboard(lastErrorLabel(detail.task.last_error_kind ?? null, detail.task.last_error ?? null))"
              >
                {{ t('common.copy') }}
              </n-button>
            </div>
            <div class="flex flex-wrap items-center gap-2">
              <n-tag v-if="detail.task.last_error_kind" size="small" type="error" :bordered="false">
                {{ detail.task.last_error_kind }}
              </n-tag>
              <div class="text-sm whitespace-pre-wrap break-words">{{ detail.task.last_error }}</div>
            </div>
          </div>

          <div v-if="detail.task.ignore_reason" class="text-sm">
            <span class="opacity-70">{{ t('settings.maintenance.cleanup.columns.ignoreReason') }}:</span>
            {{ detail.task.ignore_reason }}
          </div>
        </div>

        <div>
          <div class="text-sm font-medium mb-2">{{ t('settings.maintenance.cleanup.targetSnapshot') }}</div>
          <n-code :code="formatJson(detail.task.target_snapshot)" language="json" />
        </div>

        <div>
          <div class="text-sm font-medium mb-2">{{ t('settings.maintenance.cleanup.eventsTitle') }}</div>
          <div v-if="detail.events.length === 0" class="text-sm opacity-70">{{ t('common.noData') }}</div>
          <div v-else class="space-y-2">
            <n-card v-for="e in detail.events" :key="e.seq" size="small" class="app-card">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="text-sm font-medium truncate">{{ e.kind }}</div>
                  <div class="text-xs opacity-70 mt-0.5">{{ formatUnixSeconds(e.ts) }}</div>
                </div>
                <n-tag size="small" :type="eventLevelTagType(e.level)" :bordered="false">{{ e.level }}</n-tag>
              </div>
              <div class="text-sm mt-2">{{ e.message }}</div>
              <div v-if="e.fields" class="mt-2">
                <n-code :code="formatJson(e.fields)" language="json" />
              </div>
            </n-card>
          </div>
        </div>
      </div>
    </n-drawer-content>
  </n-drawer>

  <n-modal v-model:show="helpOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('settings.maintenance.cleanup.statusHelpTitle')">
    <div class="space-y-3">
      <div class="text-sm opacity-80">{{ t('settings.maintenance.cleanup.statusHelpIntro') }}</div>
      <div class="space-y-2">
        <div v-for="row in statusHelpItems" :key="row.status" class="flex items-start gap-2">
          <n-tag size="small" :type="statusTagType(row.status)" :bordered="false">
            {{ formatStatus(row.status) }}
          </n-tag>
          <div class="text-sm opacity-80">{{ row.body }}</div>
        </div>
      </div>

      <div class="space-y-2">
        <div class="text-sm font-medium">{{ t('settings.maintenance.cleanup.actionHelpTitle') }}</div>
        <div class="space-y-2">
          <div v-for="row in actionHelpItems" :key="row.key" class="flex items-start gap-2">
            <n-tag size="small" :bordered="false">{{ row.label }}</n-tag>
            <div class="text-sm opacity-80">{{ row.body }}</div>
          </div>
        </div>
      </div>
    </div>
  </n-modal>
</template>
