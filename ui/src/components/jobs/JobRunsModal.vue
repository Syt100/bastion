<script setup lang="ts">
import { computed, h, ref } from 'vue'
import { NButton, NDataTable, NModal, NSpace, NTag, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import { useJobsStore, type RunListItem } from '@/stores/jobs'
import { MODAL_WIDTH } from '@/lib/modal'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { runStatusLabel } from '@/lib/runs'

export type JobRunsModalExpose = {
  open: (jobId: string) => Promise<void>
}

const emit = defineEmits<{
  (e: 'open-detail', runId: string): void
  (e: 'open-events', runId: string): void
  (e: 'open-restore', runId: string): void
  (e: 'open-verify', runId: string): void
}>()

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const jobs = useJobsStore()

const show = ref<boolean>(false)
const loading = ref<boolean>(false)
const jobId = ref<string | null>(null)
const runs = ref<RunListItem[]>([])

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

function statusTagType(status: RunListItem['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  return 'default'
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
            h(NTag, { type: statusTagType(row.status) }, { default: () => runStatusLabel(t, row.status) }),
            row.executed_offline
              ? h(NTag, { size: 'small', type: 'info' }, { default: () => t('runs.badges.offline') })
              : null,
          ],
        },
      ),
  },
  { title: t('runs.columns.startedAt'), key: 'started_at', render: (row) => formatUnixSeconds(row.started_at) },
  { title: t('runs.columns.endedAt'), key: 'ended_at', render: (row) => formatUnixSeconds(row.ended_at) },
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
            h(
              NButton,
              {
                size: 'small',
                onClick: () => {
                  show.value = false
                  emit('open-detail', row.id)
                },
              },
              { default: () => t('runs.actions.detail') },
            ),
            h(NButton, { size: 'small', onClick: () => emit('open-events', row.id) }, { default: () => t('runs.actions.events') }),
            h(
              NButton,
              { size: 'small', disabled: row.status !== 'success', onClick: () => emit('open-restore', row.id) },
              { default: () => t('runs.actions.restore') },
            ),
            h(
              NButton,
              { size: 'small', disabled: row.status !== 'success', onClick: () => emit('open-verify', row.id) },
              { default: () => t('runs.actions.verify') },
            ),
          ],
        },
      ),
  },
])

async function open(nextJobId: string): Promise<void> {
  show.value = true
  jobId.value = nextJobId
  loading.value = true
  runs.value = []
  try {
    runs.value = await jobs.listRuns(nextJobId)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchRunsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

defineExpose<JobRunsModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('runs.title')">
    <div class="space-y-3">
      <div class="text-sm app-text-muted">{{ jobId }}</div>
      <n-data-table :loading="loading" :columns="columns" :data="runs" />
      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.close') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>
