<script setup lang="ts">
import { computed, h } from 'vue'
import { NButton, NDataTable, NSpace, NTag, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { runStatusLabel } from '@/lib/runs'
import type { DashboardOverviewResponse } from '@/stores/dashboard'

type RecentRun = DashboardOverviewResponse['recent_runs'][number]

const props = defineProps<{
  rows: RecentRun[]
  loading: boolean
}>()

const emit = defineEmits<{
  openRun: [row: RecentRun]
}>()

const { t } = useI18n()
const ui = useUiStore()

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

function statusTagType(status: string): 'success' | 'error' | 'warning' | 'info' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  if (status === 'running') return 'info'
  if (status === 'queued') return 'default'
  return 'default'
}

function nodeLabel(row: { node_id: string; node_name?: string | null }): string {
  if (row.node_id === 'hub') return t('jobs.nodes.hub')
  return row.node_name?.trim() || row.node_id
}

const columns = computed<DataTableColumns<RecentRun>>(() => [
  {
    title: t('dashboard.recent.columns.status'),
    key: 'status',
    render: (row) =>
      h(
        NSpace,
        { size: 8, align: 'center', wrapItem: false },
        {
          default: () => [
            h(
              NTag,
              { type: statusTagType(row.status), size: 'small', bordered: false },
              { default: () => runStatusLabel(t, row.status) },
            ),
            row.executed_offline
              ? h(NTag, { type: 'info', size: 'small', bordered: false }, { default: () => t('runs.badges.offline') })
              : null,
          ],
        },
      ),
  },
  { title: t('dashboard.recent.columns.job'), key: 'job_name', render: (row) => row.job_name },
  { title: t('dashboard.recent.columns.node'), key: 'node', render: (row) => nodeLabel(row) },
  {
    title: t('dashboard.recent.columns.startedAt'),
    key: 'started_at',
    render: (row) => h('span', { class: 'font-mono tabular-nums' }, formatUnixSeconds(row.started_at)),
  },
  {
    title: t('dashboard.recent.columns.endedAt'),
    key: 'ended_at',
    render: (row) => h('span', { class: 'font-mono tabular-nums' }, row.ended_at ? formatUnixSeconds(row.ended_at) : '-'),
  },
  { title: t('dashboard.recent.columns.error'), key: 'error', render: (row) => row.error ?? '-' },
  {
    title: t('dashboard.recent.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NButton,
        { size: 'small', tertiary: true, onClick: () => emit('openRun', row) },
        { default: () => t('dashboard.recent.actions.open') },
      ),
  },
])
</script>

<template>
  <div class="overflow-x-auto">
    <n-data-table :loading="props.loading" :columns="columns" :data="props.rows" />
  </div>
</template>
