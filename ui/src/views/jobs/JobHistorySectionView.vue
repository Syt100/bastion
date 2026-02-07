<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NDataTable, NIcon, NSpace, NTag, useMessage, type DataTableColumns } from 'naive-ui'
import { RefreshOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import RestoreWizardModal, { type RestoreWizardModalExpose } from '@/components/jobs/RestoreWizardModal.vue'
import VerifyWizardModal, { type VerifyWizardModalExpose } from '@/components/jobs/VerifyWizardModal.vue'
import OperationModal, { type OperationModalExpose } from '@/components/jobs/OperationModal.vue'
import RunEventsModal, { type RunEventsModalExpose } from '@/components/jobs/RunEventsModal.vue'
import { useJobsStore, type RunListItem, type RunStatus } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { MQ } from '@/lib/breakpoints'
import { useMediaQuery } from '@/lib/media'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { runStatusLabel } from '@/lib/runs'
import { useJobDetailContext } from '@/lib/jobDetailContext'

const { t } = useI18n()
const message = useMessage()
const router = useRouter()

const ctx = useJobDetailContext()
const jobs = useJobsStore()
const ui = useUiStore()

const isDesktop = useMediaQuery(MQ.mdUp)
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const loading = ref<boolean>(false)
const runs = ref<RunListItem[]>([])
const statusFilter = ref<RunStatus | 'all'>('all')

const restoreModal = ref<RestoreWizardModalExpose | null>(null)
const verifyModal = ref<VerifyWizardModalExpose | null>(null)
const opModal = ref<OperationModalExpose | null>(null)
const runEventsModal = ref<RunEventsModalExpose | null>(null)

const CONSISTENCY_CHANGED_BADGE_THRESHOLD = 10

function statusTagType(status: RunListItem['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  return 'default'
}

async function refresh(): Promise<void> {
  const id = ctx.jobId.value
  if (!id) return
  loading.value = true
  try {
    runs.value = await jobs.listRuns(id)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchRunsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

watch(
  () => ctx.jobId.value,
  (id) => {
    runs.value = []
    if (id) void refresh()
  },
  { immediate: true },
)

function openRunDetail(runId: string): void {
  const jobId = ctx.jobId.value
  if (!jobId) return
  void router.push(
    `/n/${encodeURIComponent(ctx.nodeId.value)}/jobs/${encodeURIComponent(jobId)}/history/runs/${encodeURIComponent(runId)}`,
  )
}

async function openRunEvents(runId: string): Promise<void> {
  await runEventsModal.value?.open(runId)
}

function openRestoreWizard(runId: string): void {
  restoreModal.value?.open(runId, { defaultNodeId: ctx.nodeId.value })
}

function openVerifyWizard(runId: string): void {
  verifyModal.value?.open(runId)
}

async function openOperation(opId: string): Promise<void> {
  await opModal.value?.open(opId)
}

const visibleRuns = computed<RunListItem[]>(() => {
  const filter = statusFilter.value
  if (filter === 'all') return runs.value
  return runs.value.filter((r) => r.status === filter)
})

function statusChipButtonType(value: RunStatus | 'all'): 'default' | 'success' | 'error' | 'warning' | 'info' {
  if (value === 'success') return 'success'
  if (value === 'failed') return 'error'
  if (value === 'rejected') return 'warning'
  if (value === 'running' || value === 'queued') return 'info'
  return 'default'
}

const statusChips = computed(() => [
  { value: 'all' as const, label: t('runs.filters.all') },
  { value: 'success' as const, label: runStatusLabel(t, 'success') },
  { value: 'failed' as const, label: runStatusLabel(t, 'failed') },
  { value: 'running' as const, label: runStatusLabel(t, 'running') },
  { value: 'queued' as const, label: runStatusLabel(t, 'queued') },
  { value: 'rejected' as const, label: runStatusLabel(t, 'rejected') },
])

const columns = computed<DataTableColumns<RunListItem>>(() => [
  {
    title: t('runs.columns.status'),
    key: 'status',
    render: (row) => {
      const errors = row.issues_errors_total ?? 0
      const warnings = row.issues_warnings_total ?? 0
      const consistencyTotal = row.consistency_total ?? 0
      const consistencySignal = row.consistency_signal_total ?? 0

      const alertTags = []
      if (errors > 0) {
        alertTags.push(
          h(NTag, { size: 'small', type: 'error', bordered: false }, { default: () => t('runs.badges.errors', { count: errors }) }),
        )
      }
      if (warnings > 0) {
        alertTags.push(
          h(NTag, { size: 'small', type: 'warning', bordered: false }, { default: () => t('runs.badges.warnings', { count: warnings }) }),
        )
      }
      if (consistencySignal > 0) {
        alertTags.push(
          h(NTag, { size: 'small', type: 'warning', bordered: false }, { default: () => t('runs.badges.sourceRisk', { count: consistencySignal }) }),
        )
      } else if (consistencyTotal >= CONSISTENCY_CHANGED_BADGE_THRESHOLD) {
        alertTags.push(
          h(NTag, { size: 'small', type: 'warning', bordered: false }, { default: () => t('runs.badges.sourceChanged', { count: consistencyTotal }) }),
        )
      }

      // Runs list should stay scannable: show high-signal alert digests only.
      const cappedAlerts = alertTags.slice(0, 3)

      return h(
        NSpace,
        { size: 8, align: 'center', wrapItem: false },
        {
          default: () => [
            h(
              NTag,
              { type: statusTagType(row.status), size: 'small', bordered: false },
              { default: () => runStatusLabel(t, row.status) },
            ),
            ...cappedAlerts,
            row.executed_offline
              ? h(NTag, { size: 'small', type: 'info', bordered: false }, { default: () => t('runs.badges.offline') })
              : null,
          ],
        },
      )
    },
  },
  {
    title: t('runs.columns.id'),
    key: 'id',
    render: (row) =>
      h(
        'span',
        {
          class: 'font-mono tabular-nums text-xs truncate block max-w-[12rem]',
          title: row.id,
        },
        row.id,
      ),
  },
  { title: t('runs.columns.startedAt'), key: 'started_at', render: (row) => h('span', { class: 'font-mono tabular-nums' }, formatUnixSeconds(row.started_at)) },
  { title: t('runs.columns.endedAt'), key: 'ended_at', render: (row) => h('span', { class: 'font-mono tabular-nums' }, formatUnixSeconds(row.ended_at)) },
  { title: t('runs.columns.error'), key: 'error', render: (row) => row.error ?? '-' },
  {
    title: t('runs.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(NButton, { size: 'small', onClick: () => openRunDetail(row.id) }, { default: () => t('runs.actions.detail') }),
            h(NButton, { size: 'small', onClick: () => void openRunEvents(row.id) }, { default: () => t('runs.actions.events') }),
            h(
              NButton,
              { size: 'small', disabled: row.status !== 'success', onClick: () => openRestoreWizard(row.id) },
              { default: () => t('runs.actions.restore') },
            ),
            h(
              NButton,
              { size: 'small', disabled: row.status !== 'success', onClick: () => openVerifyWizard(row.id) },
              { default: () => t('runs.actions.verify') },
            ),
          ],
        },
      ),
  },
])
</script>

<template>
  <div class="space-y-3">
    <n-card size="small" class="app-card" :bordered="false" data-testid="job-history-panel">
      <template #header>
        <div class="flex items-center justify-between gap-3 flex-wrap">
          <div class="text-sm font-medium">{{ t('runs.title') }}</div>

          <div class="flex items-center gap-2 flex-wrap justify-end">
            <div class="flex flex-wrap justify-end gap-2">
              <n-button
                v-for="chip in statusChips"
                :key="chip.value"
                size="small"
                :tertiary="statusFilter !== chip.value"
                :secondary="statusFilter === chip.value"
                :type="statusFilter === chip.value ? statusChipButtonType(chip.value) : 'default'"
                @click="statusFilter = chip.value"
              >
                {{ chip.label }}
              </n-button>
            </div>

            <n-button
              data-testid="job-history-refresh"
              size="small"
              tertiary
              :loading="loading"
              :title="t('common.refresh')"
              @click="refresh"
            >
              <template #icon>
                <n-icon :component="RefreshOutline" />
              </template>
              <span v-if="isDesktop">{{ t('common.refresh') }}</span>
            </n-button>
          </div>
        </div>
      </template>

      <template v-if="!isDesktop">
        <AppEmptyState v-if="loading && visibleRuns.length === 0" :title="t('common.loading')" loading />
        <AppEmptyState v-else-if="!loading && visibleRuns.length === 0" :title="t('common.noData')" />

        <div v-else class="space-y-3">
          <n-card
            v-for="row in visibleRuns"
            :key="row.id"
            size="small"
            class="app-card"
            :bordered="false"
          >
            <template #header>
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="font-mono tabular-nums truncate">{{ row.id }}</div>
                  <div class="text-xs app-text-muted mt-0.5">
                    {{ formatUnixSeconds(row.started_at) }}
                    <span class="mx-1">→</span>
                    {{ row.ended_at ? formatUnixSeconds(row.ended_at) : '-' }}
                  </div>
                </div>
                <div class="flex flex-wrap justify-end gap-1">
                  <n-tag size="small" :bordered="false" :type="statusTagType(row.status)">{{ runStatusLabel(t, row.status) }}</n-tag>
                  <n-tag
                    v-if="row.issues_errors_total != null && row.issues_errors_total > 0"
                    size="small"
                    :bordered="false"
                    type="error"
                  >{{ t('runs.badges.errors', { count: row.issues_errors_total }) }}</n-tag>
                  <n-tag
                    v-if="row.issues_warnings_total != null && row.issues_warnings_total > 0"
                    size="small"
                    :bordered="false"
                    type="warning"
                  >{{ t('runs.badges.warnings', { count: row.issues_warnings_total }) }}</n-tag>
                  <n-tag
                    v-if="row.consistency_signal_total != null && row.consistency_signal_total > 0"
                    size="small"
                    :bordered="false"
                    type="warning"
                  >{{ t('runs.badges.sourceRisk', { count: row.consistency_signal_total }) }}</n-tag>
                  <n-tag
                    v-else-if="row.consistency_total != null && row.consistency_total >= CONSISTENCY_CHANGED_BADGE_THRESHOLD"
                    size="small"
                    :bordered="false"
                    type="warning"
                  >{{ t('runs.badges.sourceChanged', { count: row.consistency_total }) }}</n-tag>
                  <n-tag v-if="row.executed_offline" size="small" :bordered="false" type="info">{{ t('runs.badges.offline') }}</n-tag>
                </div>
              </div>
            </template>

            <div v-if="row.error" class="text-xs text-[var(--app-danger)] truncate">{{ row.error }}</div>

            <template #footer>
              <div class="flex flex-wrap justify-end gap-2">
                <n-button size="small" @click="openRunDetail(row.id)">{{ t('runs.actions.detail') }}</n-button>
                <n-button size="small" @click="openRunEvents(row.id)">{{ t('runs.actions.events') }}</n-button>
                <n-button size="small" :disabled="row.status !== 'success'" @click="openRestoreWizard(row.id)">{{ t('runs.actions.restore') }}</n-button>
                <n-button size="small" :disabled="row.status !== 'success'" @click="openVerifyWizard(row.id)">{{ t('runs.actions.verify') }}</n-button>
              </div>
            </template>
          </n-card>
        </div>
      </template>

      <template v-else>
        <AppEmptyState v-if="loading && visibleRuns.length === 0" :title="t('common.loading')" loading />
        <AppEmptyState v-else-if="!loading && visibleRuns.length === 0" :title="t('common.noData')" />

        <div v-else class="overflow-x-auto">
          <n-data-table :loading="loading" :columns="columns" :data="visibleRuns" />
        </div>
      </template>
    </n-card>

    <RunEventsModal ref="runEventsModal" />
    <RestoreWizardModal ref="restoreModal" @started="(id) => void openOperation(id)" />
    <VerifyWizardModal ref="verifyModal" @started="(id) => void openOperation(id)" />
    <OperationModal ref="opModal" />
  </div>
</template>
