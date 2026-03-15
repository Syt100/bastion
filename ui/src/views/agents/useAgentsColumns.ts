import { computed, h, type ComputedRef, type Ref } from 'vue'
import { NButton, NSpace, NTag, type DataTableColumns, type DropdownOption } from 'naive-ui'

import OverflowActionsButton from '@/components/list/OverflowActionsButton.vue'
import type { FleetListItem } from '@/stores/fleet'

type Translate = (key: string, params?: Record<string, unknown>) => string

export function useAgentsColumns(options: {
  t: Translate
  isDesktop: Ref<boolean>
  shortId: (value: string) => string
  copyToClipboard: (value: string) => Promise<void>
  configSyncStatusTagType: (status: FleetListItem['config_sync']['state']) => 'default' | 'success' | 'warning' | 'error'
  configSyncTitle: (row: FleetListItem) => string
  configSyncStatusLabel: (status: FleetListItem['config_sync']['state']) => string
  formatUnixSeconds: (value: number | null) => string
  syncNowLoading: Ref<string | null>
  openAgentWorkspace: (agentId: string) => void
  openAgentJobs: (agentId: string) => void
  syncConfigNow: (agentId: string) => Promise<void>
  agentOverflowOptions: (row: FleetListItem) => DropdownOption[]
  onSelectAgentOverflow: (row: FleetListItem, key: string | number) => void
}): { columns: ComputedRef<DataTableColumns<FleetListItem>> } {
  const {
    t,
    isDesktop,
    shortId,
    copyToClipboard,
    configSyncStatusTagType,
    configSyncTitle,
    configSyncStatusLabel,
    formatUnixSeconds,
    syncNowLoading,
    openAgentWorkspace,
    openAgentJobs,
    syncConfigNow,
    agentOverflowOptions,
    onSelectAgentOverflow,
  } = options

  const columns = computed<DataTableColumns<FleetListItem>>(() => [
    ...(isDesktop.value ? [{ type: 'selection' as const }] : []),
    {
      title: t('agents.columns.name'),
      key: 'name',
      render: (row) =>
        h(
          NButton,
          {
            text: true,
            type: 'primary',
            onClick: () => openAgentWorkspace(row.id),
          },
          { default: () => row.name ?? row.id },
        ),
    },
    {
      title: t('agents.columns.id'),
      key: 'id',
      render: (row) =>
        h('div', { class: 'flex items-center gap-2' }, [
          h('span', { class: 'font-mono text-xs' }, shortId(row.id)),
          h(
            NButton,
            { quaternary: true, size: 'small', onClick: () => copyToClipboard(row.id) },
            { default: () => t('agents.actions.copy') },
          ),
        ]),
    },
    {
      title: t('agents.columns.labels'),
      key: 'labels',
      render: (row) => {
        if (!row.labels?.length) return '-'
        return h(
          'div',
          { class: 'flex flex-wrap gap-1' },
          row.labels.map((label) => h(NTag, { size: 'small' }, { default: () => label })),
        )
      },
    },
    {
      title: t('agents.columns.status'),
      key: 'status',
      render: (row) => {
        const conn = row.status === 'revoked'
          ? h(NTag, { type: 'error', size: 'small' }, { default: () => t('agents.status.revoked') })
          : row.status === 'online'
            ? h(NTag, { type: 'success', size: 'small' }, { default: () => t('agents.status.online') })
            : h(NTag, { size: 'small' }, { default: () => t('agents.status.offline') })

        const cfg = h(
          NTag,
          {
            type: configSyncStatusTagType(row.config_sync.state),
            size: 'small',
            title: configSyncTitle(row),
          },
          { default: () => configSyncStatusLabel(row.config_sync.state) },
        )

        return h('div', { class: 'flex flex-wrap gap-1' }, [conn, cfg])
      },
    },
    {
      title: t('fleet.columns.workload'),
      key: 'workload',
      render: (row) =>
        h('div', { class: 'space-y-1 text-sm' }, [
          h('div', { class: 'font-medium' }, t('fleet.workload.jobs', { count: row.assigned_jobs_total })),
          h('div', { class: 'app-meta-text' }, t('fleet.workload.pendingTasks', { count: row.pending_tasks_total })),
        ]),
    },
    {
      title: t('agents.columns.lastSeen'),
      key: 'last_seen_at',
      render: (row) => formatUnixSeconds(row.last_seen_at ?? null),
    },
    {
      title: t('agents.columns.actions'),
      key: 'actions',
      render: (row) =>
        h(
          NSpace,
          { size: 8 },
          {
            default: () => [
              h(
                NButton,
                { tertiary: true, size: 'small', onClick: () => openAgentJobs(row.id) },
                { default: () => t('agents.actions.jobs') },
              ),
              h(
                NButton,
                {
                  tertiary: true,
                  size: 'small',
                  loading: syncNowLoading.value === row.id,
                  disabled: row.status === 'revoked',
                  onClick: () => syncConfigNow(row.id),
                },
                { default: () => t('agents.actions.syncNow') },
              ),
              h(OverflowActionsButton, {
                size: 'small',
                options: agentOverflowOptions(row),
                onSelect: (key: string | number) => onSelectAgentOverflow(row, key),
              }),
            ],
          },
        ),
    },
  ])

  return {
    columns,
  }
}
