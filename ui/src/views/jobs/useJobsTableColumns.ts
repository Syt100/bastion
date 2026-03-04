import { computed, h, type ComputedRef } from 'vue'
import { NButton, NTag, type DataTableColumns } from 'naive-ui'

import { runStatusLabel } from '@/lib/runs'
import type { JobListItem, RunStatus } from '@/stores/jobs'

type Translate = (key: string, params?: Record<string, unknown>) => string

export function useJobsTableColumns(options: {
  t: Translate
  tableNameSortOrder: ComputedRef<'ascend' | 'descend' | false>
  tableUpdatedSortOrder: ComputedRef<'ascend' | 'descend' | false>
  formatNodeLabel: (agentId: string | null) => string
  formatScheduleLabel: (job: JobListItem) => string
  runStatusTagType: (status: RunStatus) => 'success' | 'error' | 'warning' | 'default'
  isRowRunNowBusy: (jobId: string) => boolean
  openJob: (jobId: string) => void
  openEdit: (jobId: string) => Promise<void>
  runNow: (jobId: string) => Promise<void>
  formatUnixSecondsYmdHm: (value: number | null) => string
  formatUnixSecondsYmdHms: (value: number | null) => string
}) {
  const {
    t,
    tableNameSortOrder,
    tableUpdatedSortOrder,
    formatNodeLabel,
    formatScheduleLabel,
    runStatusTagType,
    isRowRunNowBusy,
    openJob,
    openEdit,
    runNow,
    formatUnixSecondsYmdHm,
    formatUnixSecondsYmdHms,
  } = options

  const tableColumns = computed<DataTableColumns<JobListItem>>(() => [
    { type: 'selection' as const },
    {
      title: t('jobs.columns.name'),
      key: 'name',
      sorter: 'default',
      sortOrder: tableNameSortOrder.value,
      fixed: 'left',
      width: 260,
      render: (row) =>
        h('div', { class: 'min-w-0' }, [
          h('div', { class: 'flex items-center gap-2 min-w-0' }, [
            h(
              'button',
              {
                type: 'button',
                class: 'text-left font-medium truncate hover:underline',
                title: row.name,
                onClick: () => openJob(row.id),
              },
              row.name,
            ),
            row.archived_at
              ? h(NTag, { size: 'small', bordered: false, type: 'warning' }, { default: () => t('jobs.archived') })
              : null,
          ]),
        ]),
    },
    {
      title: t('jobs.columns.node'),
      key: 'node',
      width: 160,
      render: (row) =>
        h(
          NTag,
          { size: 'small', bordered: false, type: row.agent_id ? 'default' : 'info' },
          { default: () => formatNodeLabel(row.agent_id) },
        ),
    },
    {
      title: t('jobs.columns.schedule'),
      key: 'schedule',
      width: 180,
      render: (row) => {
        const schedule = formatScheduleLabel(row)
        return h('div', { class: 'min-w-0' }, [
          h('div', { class: 'font-mono tabular-nums truncate', title: schedule }, schedule),
          row.schedule
            ? h('div', { class: 'app-meta-text font-mono tabular-nums truncate' }, row.schedule_timezone)
            : null,
        ])
      },
    },
    {
      title: t('runs.columns.status'),
      key: 'latest_run_status',
      width: 120,
      render: (row) => {
        const status = row.latest_run_status
        if (!status) return h(NTag, { size: 'small', bordered: false }, { default: () => t('runs.neverRan') })
        return h(
          NTag,
          { size: 'small', bordered: false, type: runStatusTagType(status) },
          { default: () => runStatusLabel(t, status) },
        )
      },
    },
    {
      title: t('dashboard.recent.columns.startedAt'),
      key: 'latest_run_started_at',
      width: 140,
      render: (row) =>
        h(
          'span',
          {
            class: 'font-mono tabular-nums app-meta-text',
            title: row.latest_run_started_at != null ? formatUnixSecondsYmdHms(row.latest_run_started_at) : '-',
          },
          row.latest_run_started_at != null ? formatUnixSecondsYmdHm(row.latest_run_started_at) : '-',
        ),
    },
    {
      title: t('jobs.columns.updatedAt'),
      key: 'updated_at',
      sorter: 'default',
      sortOrder: tableUpdatedSortOrder.value,
      width: 140,
      render: (row) =>
        h(
          'span',
          { class: 'font-mono tabular-nums app-meta-text', title: formatUnixSecondsYmdHms(row.updated_at) },
          formatUnixSecondsYmdHm(row.updated_at),
        ),
    },
    {
      title: t('jobs.columns.actions'),
      key: 'actions',
      fixed: 'right',
      width: 200,
      render: (row) =>
        h('div', { class: 'flex items-center gap-2 justify-end' }, [
          h(
            NButton,
            {
              size: 'small',
              loading: isRowRunNowBusy(row.id),
              disabled: !!row.archived_at || isRowRunNowBusy(row.id),
              onClick: () => void runNow(row.id),
            },
            { default: () => t('jobs.actions.runNow') },
          ),
          h(
            NButton,
            { size: 'small', disabled: !!row.archived_at, onClick: () => void openEdit(row.id) },
            { default: () => t('common.edit') },
          ),
        ]),
    },
  ])

  return {
    tableColumns,
  }
}
