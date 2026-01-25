<script setup lang="ts">
import { computed, h, ref } from 'vue'
import {
  NButton,
  NCard,
  NCode,
  NDataTable,
  NDrawer,
  NDrawerContent,
  NInput,
  NModal,
  NSelect,
  NSpace,
  NTabs,
  NTabPane,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { MQ } from '@/lib/breakpoints'
import { copyText } from '@/lib/clipboard'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { filterRunEvents, findFirstEventSeq, uniqueRunEventKinds } from '@/lib/run_events'
import { runTargetTypeLabel } from '@/lib/runs'
import { parseRunSummary } from '@/lib/run_summary'
import { useUiStore } from '@/stores/ui'
import type { RunEvent } from '@/stores/jobs'
import type { Operation } from '@/stores/operations'

type WsStatus = 'disconnected' | 'connecting' | 'live' | 'reconnecting' | 'error'

const props = defineProps<{
  runId?: string | null | undefined
  events: RunEvent[]
  ops: Operation[]
  wsStatus: WsStatus
  summary: unknown | null
}>()

const emit = defineEmits<{
  (e: 'open-operation', opId: string): void
  (e: 'reconnect'): void
}>()

const { t } = useI18n()
const message = useMessage()
const ui = useUiStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const detailTab = ref<'events' | 'operations' | 'summary'>('events')

const searchQuery = ref<string>('')
const levelFilter = ref<string | null>(null)
const kindFilter = ref<string | null>(null)

const kindOptions = computed(() =>
  uniqueRunEventKinds(props.events).map((k) => ({ label: k, value: k })),
)

const filteredEvents = computed(() =>
  filterRunEvents(props.events, {
    query: searchQuery.value,
    level: levelFilter.value,
    kind: kindFilter.value,
  }),
)

function opStatusTagType(status: Operation['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'running') return 'warning'
  return 'default'
}

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

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

async function copySummaryJson(): Promise<void> {
  if (props.summary == null) return
  const ok = await copyText(formatJson(props.summary))
  if (ok) message.success(t('messages.copied'))
  else message.error(t('errors.copyFailed'))
}

const opColumns = computed<DataTableColumns<Operation>>(() => [
  {
    title: t('operations.kind'),
    key: 'kind',
    render: (row) => row.kind,
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
      const stage = (p as { stage?: unknown }).stage
      return typeof stage === 'string' ? stage : '-'
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
          default: () => [
            h(NButton, { size: 'small', onClick: () => emit('open-operation', row.id) }, { default: () => t('runEvents.actions.details') }),
          ],
        },
      ),
  },
])

const parsedSummary = computed(() => parseRunSummary(props.summary))
const targetTypeLabel = computed(() => runTargetTypeLabel(t, parsedSummary.value.targetType))

const eventsListEl = ref<HTMLDivElement | null>(null)

function scrollToSeq(seq: number): void {
  const root = eventsListEl.value
  if (!root) return
  const el = root.querySelector<HTMLElement>(`[data-event-seq=\"${seq}\"]`)
  if (!el) return
  el.scrollIntoView({ block: 'nearest' })
}

function jumpToFirstError(): void {
  const seq = findFirstEventSeq(filteredEvents.value, (e) => e.level === 'error')
  if (seq != null) scrollToSeq(seq)
}

function jumpToFirstWarn(): void {
  const seq = findFirstEventSeq(filteredEvents.value, (e) => e.level === 'warn' || e.level === 'warning')
  if (seq != null) scrollToSeq(seq)
}

function jumpToLatest(): void {
  const list = filteredEvents.value
  const last = list.length > 0 ? list[list.length - 1] : null
  if (last) scrollToSeq(last.seq)
}

function exportFilteredEvents(): void {
  const runId = props.runId?.trim() || 'run'
  const filename = `${runId}-events.json`
  const payload = JSON.stringify(filteredEvents.value, null, 2)

  const blob = new Blob([payload], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  window.setTimeout(() => URL.revokeObjectURL(url), 0)
}

const eventDetailShow = ref<boolean>(false)
const eventDetail = ref<RunEvent | null>(null)

function openEventDetails(e: RunEvent): void {
  eventDetail.value = e
  eventDetailShow.value = true
}

async function copyEventJson(e: RunEvent): Promise<void> {
  const ok = await copyText(formatJson(e))
  if (ok) message.success(t('messages.copied'))
  else message.error(t('errors.copyFailed'))
}
</script>

<template>
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
            <span class="text-xs opacity-70">{{ filteredEvents.length }} / {{ events.length }} events</span>
            <n-button
              v-if="wsStatus === 'disconnected' || wsStatus === 'error'"
              size="tiny"
              quaternary
              @click="emit('reconnect')"
            >
              {{ t('runEvents.actions.reconnect') }}
            </n-button>
          </div>
        </div>

        <div class="mt-3 flex flex-wrap items-start gap-2">
          <div class="flex flex-1 flex-wrap items-center gap-2 min-w-0">
            <n-input v-model:value="searchQuery" size="small" clearable :placeholder="t('common.search')" class="min-w-[12rem] flex-1" />
            <div class="flex flex-wrap items-center gap-2 min-w-0">
              <n-select v-model:value="levelFilter" size="small" clearable :placeholder="t('runEvents.filters.level')" :options="[
                { label: 'error', value: 'error' },
                { label: 'warn', value: 'warn' },
                { label: 'info', value: 'info' },
              ]" class="flex-[0_1_10rem] min-w-[8rem]" />
              <n-select
                v-model:value="kindFilter"
                size="small"
                clearable
                filterable
                :placeholder="t('runEvents.filters.kind')"
                :options="kindOptions"
                :consistent-menu-width="false"
                class="flex-[0_1_10rem] min-w-[8rem]"
              />
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-2 w-full md:w-auto md:justify-end">
            <n-button size="small" quaternary :disabled="findFirstEventSeq(filteredEvents, (e) => e.level === 'error') == null" @click="jumpToFirstError">
              {{ t('runEvents.actions.firstError') }}
            </n-button>
            <n-button size="small" quaternary :disabled="findFirstEventSeq(filteredEvents, (e) => e.level === 'warn' || e.level === 'warning') == null" @click="jumpToFirstWarn">
              {{ t('runEvents.actions.firstWarn') }}
            </n-button>
            <n-button size="small" quaternary :disabled="filteredEvents.length === 0" @click="jumpToLatest">
              {{ t('runEvents.actions.latest') }}
            </n-button>
            <n-button size="small" quaternary :disabled="filteredEvents.length === 0" @click="exportFilteredEvents">
              {{ t('runEvents.actions.export') }}
            </n-button>
          </div>
        </div>

        <div v-if="filteredEvents.length === 0" class="mt-3 text-sm opacity-70">{{ t('common.noData') }}</div>

        <div
          v-else
          data-testid="run-detail-events-list"
          class="mt-3 max-h-[60vh] overflow-auto rounded-md app-border-subtle divide-y divide-black/5 dark:divide-white/10"
          ref="eventsListEl"
        >
          <button
            v-for="item in filteredEvents"
            :key="item.seq"
            type="button"
            :data-event-seq="item.seq"
            class="w-full text-left px-3 py-2 flex items-center gap-2 transition hover:bg-black/5 dark:hover:bg-white/5"
            @click="openEventDetails(item)"
          >
            <span class="shrink-0 font-mono tabular-nums text-xs opacity-70 whitespace-nowrap" :title="formatUnixSeconds(item.ts)">
              {{ formatUnixSeconds(item.ts) }}
            </span>
            <n-tag class="shrink-0 w-16 inline-flex justify-center" size="tiny" :type="runEventLevelTagType(item.level)" :bordered="false">
              <span class="block w-full truncate text-center">{{ item.level }}</span>
            </n-tag>
            <span class="min-w-0 flex-1 truncate text-sm" :title="item.message">{{ item.message }}</span>
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

      <n-tab-pane name="summary" :disabled="!summary">
        <template #tab>
          <span>{{ t('runs.detail.summaryTitle') }}</span>
        </template>

        <div v-if="!summary" class="text-sm opacity-70 py-2">{{ t('common.noData') }}</div>
        <div v-else data-testid="run-detail-summary">
          <div class="flex items-center justify-between gap-3 mb-3">
            <div class="text-sm opacity-70">{{ t('runs.detail.summaryHelp') }}</div>
            <n-button size="small" quaternary @click="copySummaryJson">{{ t('common.copy') }}</n-button>
          </div>

          <div class="grid grid-cols-1 gap-3" :class="parsedSummary.sqlitePath || parsedSummary.sqliteSnapshotName || parsedSummary.vaultwardenDataDir ? 'md:grid-cols-2' : 'md:grid-cols-1'">
            <div class="rounded border border-black/5 dark:border-white/10 p-3">
              <div class="text-sm font-medium mb-2">{{ t('runs.detail.summaryHighlights') }}</div>
              <div class="text-xs opacity-70 space-y-1">
                <div v-if="parsedSummary.targetType || parsedSummary.targetLocation">
                  {{ t('runs.detail.target') }}:
                  <span class="font-mono tabular-nums">{{ targetTypeLabel }}</span>
                  <span v-if="parsedSummary.targetLocation" class="font-mono tabular-nums"> · {{ parsedSummary.targetLocation }}</span>
                </div>
                <div v-if="parsedSummary.entriesCount != null">{{ t('runs.detail.entries', { count: parsedSummary.entriesCount }) }}</div>
                <div v-if="parsedSummary.partsCount != null">{{ t('runs.detail.parts', { count: parsedSummary.partsCount }) }}</div>
                <div v-if="parsedSummary.errorsTotal != null && parsedSummary.errorsTotal > 0">{{ t('runs.detail.errors', { count: parsedSummary.errorsTotal }) }}</div>
                <div v-if="parsedSummary.warningsTotal != null && parsedSummary.warningsTotal > 0">{{ t('runs.detail.warnings', { count: parsedSummary.warningsTotal }) }}</div>
              </div>
            </div>

            <div v-if="parsedSummary.sqlitePath || parsedSummary.sqliteSnapshotName || parsedSummary.vaultwardenDataDir" class="rounded border border-black/5 dark:border-white/10 p-3">
              <div class="text-sm font-medium mb-2">{{ t('runs.detail.summaryDetails') }}</div>
              <div class="text-xs opacity-70 space-y-1">
                <div v-if="parsedSummary.sqlitePath">
                  sqlite: <span class="font-mono tabular-nums">{{ parsedSummary.sqlitePath }}</span>
                </div>
                <div v-if="parsedSummary.sqliteSnapshotName">
                  snapshot: <span class="font-mono tabular-nums">{{ parsedSummary.sqliteSnapshotName }}</span>
                </div>
                <div v-if="parsedSummary.vaultwardenDataDir">
                  vaultwarden: <span class="font-mono tabular-nums">{{ parsedSummary.vaultwardenDataDir }}</span>
                </div>
              </div>
            </div>
          </div>

          <details class="mt-3 rounded border border-black/5 dark:border-white/10 p-3">
            <summary class="cursor-pointer select-none text-sm font-medium">
              {{ t('runs.detail.rawJson') }}
            </summary>
            <div class="mt-3">
              <n-code :code="formatJson(summary)" language="json" />
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
        <n-button size="tiny" quaternary @click="copyEventJson(eventDetail)">{{ t('common.copy') }}</n-button>
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
          <n-button size="tiny" quaternary @click="copyEventJson(eventDetail)">{{ t('common.copy') }}</n-button>
        </div>
        <div class="font-mono text-sm whitespace-pre-wrap break-words">{{ eventDetail.message }}</div>
        <n-code v-if="eventDetail.fields" :code="formatJson(eventDetail.fields)" language="json" />
      </div>
    </n-drawer-content>
  </n-drawer>
</template>
