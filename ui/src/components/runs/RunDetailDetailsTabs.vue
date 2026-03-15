<script setup lang="ts">
import { computed, h, ref } from 'vue'
import {
  NButton,
  NCard,
  NCode,
  NDataTable,
  NInput,
  NSelect,
  NSpace,
  NTabPane,
  NTabs,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { copyText } from '@/lib/clipboard'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { useMediaQuery } from '@/lib/media'
import {
  runEventLevelTagType,
  uniqueRunEventKinds,
  RUN_EVENT_DETAIL_HEADER_META_FIELDS_WITH_IDENTIFIERS,
} from '@/lib/run_events'
import { runTargetTypeLabel } from '@/lib/runs'
import { parseRunSummary } from '@/lib/run_summary'
import { operationKindLabel, operationStatusLabel } from '@/lib/operations'
import RunEventDetailDialog from '@/components/runs/RunEventDetailDialog.vue'
import { useUiStore } from '@/stores/ui'
import type { RunEvent } from '@/stores/runs'
import type { Operation } from '@/stores/operations'

type WsStatus = 'disconnected' | 'connecting' | 'live' | 'reconnecting' | 'error'

const props = defineProps<{
  runId: string
  events: RunEvent[]
  consoleLoading: boolean
  window: {
    first_seq?: number | null
    last_seq?: number | null
    has_older: boolean
    has_newer: boolean
  } | null
  locators: {
    first_error_seq?: number | null
    root_cause_seq?: number | null
  } | null
  filters: {
    search: string
    level: string | null
    kind: string | null
  }
  ops: Operation[]
  wsStatus: WsStatus
  summary: unknown | null
}>()

const emit = defineEmits<{
  (e: 'update:search', value: string): void
  (e: 'update:level', value: string | null): void
  (e: 'update:kind', value: string | null): void
  (e: 'load-older'): void
  (e: 'load-newer'): void
  (e: 'jump-latest'): void
  (e: 'jump-first-error'): void
  (e: 'open-operation', opId: string): void
  (e: 'reconnect'): void
}>()

const { t } = useI18n()
const message = useMessage()
const ui = useUiStore()
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))
const isDesktop = useMediaQuery(MQ.mdUp)

const detailTab = ref<'events' | 'operations' | 'summary'>('events')

const kindOptions = computed(() =>
  uniqueRunEventKinds(props.events).map((k) => ({ label: k, value: k })),
)

function opStatusTagType(status: Operation['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'running') return 'warning'
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
    render: (row) => operationKindLabel(t, row.kind),
  },
  {
    title: t('runs.columns.status'),
    key: 'status',
    render: (row) =>
      h(NTag, { type: opStatusTagType(row.status), bordered: false }, { default: () => operationStatusLabel(t, row.status) }),
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
const consistencyReport = computed(() => parsedSummary.value.consistency)

const eventDetailShow = ref(false)
const eventDetail = ref<RunEvent | null>(null)
const eventDetailHeaderMetaFields = RUN_EVENT_DETAIL_HEADER_META_FIELDS_WITH_IDENTIFIERS

function openEventDetails(event: RunEvent): void {
  eventDetail.value = event
  eventDetailShow.value = true
}

async function copyEventJson(event: RunEvent): Promise<void> {
  const ok = await copyText(formatJson(event))
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

        <div class="flex items-center justify-between gap-3 flex-wrap">
          <div class="flex items-center gap-2 flex-wrap min-w-0">
            <n-tag size="small" :type="wsStatusTagType(wsStatus)" :bordered="false">
              {{ t(`runEvents.ws.${wsStatus}`) }}
            </n-tag>
            <span class="text-xs app-text-muted">
              {{ t('runs.detail.eventWindow', { first: window?.first_seq ?? '-', last: window?.last_seq ?? '-' }) }}
            </span>
            <n-button
              v-if="wsStatus === 'disconnected' || wsStatus === 'error'"
              size="tiny"
              quaternary
              @click="emit('reconnect')"
            >
              {{ t('runEvents.actions.reconnect') }}
            </n-button>
          </div>

          <div class="flex items-center gap-2 flex-wrap">
            <n-button
              size="tiny"
              quaternary
              :disabled="consoleLoading || !locators?.first_error_seq"
              @click="emit('jump-first-error')"
            >
              {{ t('runEvents.actions.firstError') }}
            </n-button>
            <n-button size="tiny" quaternary :disabled="consoleLoading" @click="emit('jump-latest')">
              {{ t('runEvents.actions.latest') }}
            </n-button>
          </div>
        </div>

        <div class="mt-3 flex flex-col gap-2 md:flex-row md:flex-wrap md:items-center">
          <n-input
            :value="filters.search"
            size="small"
            clearable
            :placeholder="t('runs.filters.search')"
            class="w-full md:flex-1 md:min-w-[10rem] md:w-0"
            @update:value="(value) => emit('update:search', value)"
          />
          <div class="grid grid-cols-2 gap-2 w-full md:flex md:items-center md:gap-2 md:w-auto">
            <n-select
              :value="filters.level"
              size="small"
              clearable
              :placeholder="t('runEvents.filters.level')"
              :options="[
                { label: 'error', value: 'error' },
                { label: 'warn', value: 'warn' },
                { label: 'info', value: 'info' },
              ]"
              class="w-full md:w-[9rem]"
              @update:value="(value) => emit('update:level', value)"
            />
            <n-select
              :value="filters.kind"
              size="small"
              clearable
              filterable
              :placeholder="t('runEvents.filters.kind')"
              :options="kindOptions"
              class="w-full md:w-[11rem]"
              @update:value="(value) => emit('update:kind', value)"
            />
          </div>
        </div>

        <div class="mt-3 flex items-center justify-between gap-2 flex-wrap">
          <div class="text-xs app-text-muted">{{ consoleLoading ? t('common.loading') : t('common.ready') }}</div>
          <div class="flex items-center gap-2 flex-wrap">
            <n-button size="tiny" quaternary :disabled="consoleLoading || !window?.has_older" @click="emit('load-older')">
              {{ t('runs.detail.loadOlderEvents') }}
            </n-button>
            <n-button size="tiny" quaternary :disabled="consoleLoading || !window?.has_newer" @click="emit('load-newer')">
              {{ t('runs.detail.loadNewerEvents') }}
            </n-button>
          </div>
        </div>

        <div v-if="events.length === 0" class="mt-3 text-sm app-text-muted">{{ t('runEvents.noEvents') }}</div>

        <div
          v-else
          data-testid="run-detail-events-list"
          class="mt-3 max-h-[60vh] overflow-auto rounded-md app-border-subtle app-divide-y"
        >
          <div
            v-for="item in events"
            :key="item.seq"
            :data-event-seq="item.seq"
            class="w-full px-3 py-2 flex items-start gap-2"
          >
            <span class="shrink-0 font-mono tabular-nums text-xs app-text-muted whitespace-nowrap" :title="formatUnixSeconds(item.ts)">
              {{ formatUnixSeconds(item.ts) }}
            </span>

            <n-tag size="small" :bordered="false" :type="runEventLevelTagType(item.level)">
              {{ item.level }}
            </n-tag>

            <div class="min-w-0 flex-1">
              <div class="text-sm break-words">{{ item.message }}</div>
              <div class="text-xs app-text-muted mt-1">{{ item.kind }} · #{{ item.seq }}</div>
            </div>

            <div class="flex items-center gap-1">
              <n-button size="tiny" quaternary @click="copyEventJson(item)">
                {{ t('common.copy') }}
              </n-button>
              <n-button size="tiny" @click="openEventDetails(item)">
                {{ t('runEvents.actions.details') }}
              </n-button>
            </div>
          </div>
        </div>
      </n-tab-pane>

      <n-tab-pane name="operations">
        <template #tab>
          <div class="flex items-center gap-2">
            <span>{{ t('operations.title') }}</span>
            <n-tag size="tiny" :bordered="false">{{ ops.length }}</n-tag>
          </div>
        </template>

        <div v-if="ops.length === 0" class="text-sm app-text-muted">
          {{ t('runs.detail.noOperations') }}
        </div>
        <n-data-table
          v-else
          size="small"
          :columns="opColumns"
          :data="ops"
          :pagination="false"
          class="app-data-table"
        />
      </n-tab-pane>

      <n-tab-pane name="summary">
        <template #tab>
          <div class="flex items-center gap-2">
            <span>{{ t('runs.detail.summaryTitle') }}</span>
          </div>
        </template>

        <div class="space-y-4">
          <div class="flex items-start justify-between gap-3 flex-wrap">
            <div>
              <div class="text-sm font-medium">{{ t('runs.detail.summaryTitle') }}</div>
              <div class="text-sm app-text-muted">{{ t('runs.detail.summaryHelp') }}</div>
            </div>
            <n-button size="small" quaternary :disabled="summary == null" @click="copySummaryJson">
              {{ t('common.copy') }}
            </n-button>
          </div>

          <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div class="rounded-xl app-panel-inset p-4">
              <div class="text-sm font-medium mb-2">{{ t('runs.detail.summaryHighlights') }}</div>
              <div class="space-y-1 text-sm">
                <div>
                  {{ t('runs.detail.target') }}:
                  <span class="font-medium">{{ targetTypeLabel }}</span>
                </div>
                <div v-if="parsedSummary.entriesCount != null">{{ t('runs.detail.entries', { count: parsedSummary.entriesCount }) }}</div>
                <div v-if="parsedSummary.partsCount != null">{{ t('runs.detail.parts', { count: parsedSummary.partsCount }) }}</div>
                <div v-if="parsedSummary.errorsTotal != null && parsedSummary.errorsTotal > 0">{{ t('runs.detail.errors', { count: parsedSummary.errorsTotal }) }}</div>
                <div v-if="parsedSummary.warningsTotal != null && parsedSummary.warningsTotal > 0">{{ t('runs.detail.warnings', { count: parsedSummary.warningsTotal }) }}</div>
              </div>
            </div>

            <div class="rounded-xl app-panel-inset p-4">
              <div class="text-sm font-medium mb-2">{{ t('runs.detail.summaryDetails') }}</div>
              <div class="space-y-1 text-sm">
                <div v-if="parsedSummary.targetLocation">
                  <span class="app-text-muted">{{ t('runs.detail.target') }}:</span>
                  <span class="font-mono tabular-nums break-all">{{ parsedSummary.targetLocation }}</span>
                </div>
                <div v-if="parsedSummary.sqlitePath">
                  {{ t('runs.detail.sqlitePath') }}:
                  <span class="font-mono tabular-nums">{{ parsedSummary.sqlitePath }}</span>
                </div>
                <div v-if="parsedSummary.sqliteSnapshotName">
                  {{ t('runs.detail.sqliteSnapshot') }}:
                  <span class="font-mono tabular-nums">{{ parsedSummary.sqliteSnapshotName }}</span>
                </div>
                <div v-if="parsedSummary.vaultwardenDataDir">
                  {{ t('runs.detail.vaultwardenDataDir') }}:
                  <span class="font-mono tabular-nums">{{ parsedSummary.vaultwardenDataDir }}</span>
                </div>
              </div>
            </div>
          </div>

          <div v-if="consistencyReport" data-testid="run-detail-consistency" class="rounded-xl app-panel-inset p-4 space-y-3">
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="text-sm font-medium">{{ t('runs.consistency.title') }}</div>
                <div class="text-xs app-text-muted">{{ t('runs.consistency.help') }}</div>
              </div>
              <n-button size="small" quaternary @click="emit('jump-first-error')">
                {{ t('runs.consistency.viewEvents') }}
              </n-button>
            </div>

            <div class="flex flex-wrap gap-2">
              <n-tag size="small" :bordered="false" type="warning">
                {{ t('runs.badges.sourceChanged', { count: consistencyReport.total }) }}
              </n-tag>
              <n-tag size="small" :bordered="false">{{ t('runs.consistency.changed', { count: consistencyReport.changedTotal }) }}</n-tag>
              <n-tag size="small" :bordered="false">{{ t('runs.consistency.replaced', { count: consistencyReport.replacedTotal }) }}</n-tag>
              <n-tag size="small" :bordered="false">{{ t('runs.consistency.deleted', { count: consistencyReport.deletedTotal }) }}</n-tag>
              <n-tag size="small" :bordered="false">{{ t('runs.consistency.readError', { count: consistencyReport.readErrorTotal }) }}</n-tag>
            </div>
          </div>

          <n-code
            v-if="summary != null"
            :code="formatJson(summary)"
            language="json"
            show-line-numbers
            word-wrap
            class="max-h-[24rem] overflow-auto"
          />
        </div>
      </n-tab-pane>
    </n-tabs>

    <RunEventDetailDialog
      v-model:show="eventDetailShow"
      :event="eventDetail"
      :is-desktop="isDesktop"
      :header-meta-fields="eventDetailHeaderMetaFields"
      :title="t('runEvents.details.title')"
      :close-label="t('common.close')"
    />
  </n-card>
</template>
