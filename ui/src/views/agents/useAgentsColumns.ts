import { computed, h, type ComputedRef, type Ref } from 'vue'
import { NButton, NSpace, NTag, type DataTableColumns, type DropdownOption } from 'naive-ui'

import OverflowActionsButton from '@/components/list/OverflowActionsButton.vue'
import type { AgentListItem } from '@/stores/agents'

type Translate = (key: string, params?: Record<string, unknown>) => string

export function useAgentsColumns(options: {
  t: Translate
  isDesktop: Ref<boolean>
  shortId: (value: string) => string
  copyToClipboard: (value: string) => Promise<void>
  configSyncStatusTagType: (status: AgentListItem['config_sync_status']) => 'default' | 'success' | 'warning' | 'error'
  configSyncTitle: (row: AgentListItem) => string
  configSyncStatusLabel: (status: AgentListItem['config_sync_status']) => string
  formatUnixSeconds: (value: number | null) => string
  syncNowLoading: Ref<string | null>
  openAgentJobs: (agentId: string) => void
  syncConfigNow: (agentId: string) => Promise<void>
  agentOverflowOptions: (row: AgentListItem) => DropdownOption[]
  onSelectAgentOverflow: (row: AgentListItem, key: string | number) => void
}): { columns: ComputedRef<DataTableColumns<AgentListItem>> } {
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
    openAgentJobs,
    syncConfigNow,
    agentOverflowOptions,
    onSelectAgentOverflow,
  } = options

  const columns = computed<DataTableColumns<AgentListItem>>(() => [
    ...(isDesktop.value ? [{ type: 'selection' as const }] : []),
    {
      title: t('agents.columns.name'),
      key: 'name',
      render: (row) => row.name ?? '-',
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
        const conn = row.revoked
          ? h(NTag, { type: 'error', size: 'small' }, { default: () => t('agents.status.revoked') })
          : row.online
            ? h(NTag, { type: 'success', size: 'small' }, { default: () => t('agents.status.online') })
            : h(NTag, { size: 'small' }, { default: () => t('agents.status.offline') })

        const cfg = h(
          NTag,
          {
            type: configSyncStatusTagType(row.config_sync_status),
            size: 'small',
            title: configSyncTitle(row),
          },
          { default: () => configSyncStatusLabel(row.config_sync_status) },
        )

        return h('div', { class: 'flex flex-wrap gap-1' }, [conn, cfg])
      },
    },
    {
      title: t('agents.columns.lastSeen'),
      key: 'last_seen_at',
      render: (row) => formatUnixSeconds(row.last_seen_at),
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
                  disabled: row.revoked,
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
