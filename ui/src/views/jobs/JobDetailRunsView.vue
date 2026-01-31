<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NDataTable, NSpace, NTag, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import RestoreWizardModal, { type RestoreWizardModalExpose } from '@/components/jobs/RestoreWizardModal.vue'
import VerifyWizardModal, { type VerifyWizardModalExpose } from '@/components/jobs/VerifyWizardModal.vue'
import OperationModal, { type OperationModalExpose } from '@/components/jobs/OperationModal.vue'
import RunEventsModal, { type RunEventsModalExpose } from '@/components/jobs/RunEventsModal.vue'
import { useJobsStore, type RunListItem } from '@/stores/jobs'
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

const restoreModal = ref<RestoreWizardModalExpose | null>(null)
const verifyModal = ref<VerifyWizardModalExpose | null>(null)
const opModal = ref<OperationModalExpose | null>(null)
const runEventsModal = ref<RunEventsModalExpose | null>(null)

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
  void router.push(`/n/${encodeURIComponent(ctx.nodeId.value)}/runs/${encodeURIComponent(runId)}`)
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

const columns = computed<DataTableColumns<RunListItem>>(() => [
  {
    title: t('runs.columns.status'),
    key: 'status',
    render: (row) =>
      h(
        NSpace,
        { size: 8, align: 'center', wrapItem: false },
        {
          default: () => [
            h(NTag, { type: statusTagType(row.status), size: 'small', bordered: false }, { default: () => runStatusLabel(t, row.status) }),
            row.executed_offline
              ? h(NTag, { size: 'small', type: 'info', bordered: false }, { default: () => t('runs.badges.offline') })
              : null,
          ],
        },
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
    <div class="flex items-center justify-end gap-2">
      <n-button :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
    </div>

    <div v-if="!isDesktop" class="space-y-3">
      <AppEmptyState v-if="loading && runs.length === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!loading && runs.length === 0" :title="t('common.noData')" />

      <n-card
        v-for="row in runs"
        :key="row.id"
        size="small"
        class="app-card"
      >
        <template #header>
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="font-mono tabular-nums truncate">{{ row.id }}</div>
              <div class="text-xs opacity-70 mt-0.5">
                {{ formatUnixSeconds(row.started_at) }}
                <span class="mx-1">→</span>
                {{ row.ended_at ? formatUnixSeconds(row.ended_at) : '-' }}
              </div>
            </div>
            <div class="flex flex-wrap justify-end gap-1">
              <n-tag size="small" :bordered="false" :type="statusTagType(row.status)">{{ runStatusLabel(t, row.status) }}</n-tag>
              <n-tag v-if="row.executed_offline" size="small" :bordered="false" type="info">{{ t('runs.badges.offline') }}</n-tag>
            </div>
          </div>
        </template>

        <div v-if="row.error" class="text-xs text-red-600 truncate">{{ row.error }}</div>

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

    <n-card v-else class="app-card">
      <AppEmptyState v-if="loading && runs.length === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!loading && runs.length === 0" :title="t('common.noData')" />

      <div v-else class="overflow-x-auto">
        <n-data-table :loading="loading" :columns="columns" :data="runs" />
      </div>
    </n-card>

    <RunEventsModal ref="runEventsModal" />
    <RestoreWizardModal ref="restoreModal" @started="(id) => void openOperation(id)" />
    <VerifyWizardModal ref="verifyModal" @started="(id) => void openOperation(id)" />
    <OperationModal ref="opModal" />
  </div>
</template>

