<script setup lang="ts">
import { computed, h, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NDataTable, NEmpty, NSpace, NTag, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import BackupTrendChart from '@/components/BackupTrendChart.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import { useDashboardStore, type DashboardOverviewResponse } from '@/stores/dashboard'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { runStatusLabel } from '@/lib/runs'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'

const { t } = useI18n()
const message = useMessage()
const router = useRouter()

const ui = useUiStore()
const dashboard = useDashboardStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const overview = computed(() => dashboard.overview)
const offlineAgents = computed(() => overview.value?.stats.agents.offline ?? 0)
const failedNotifications = computed(() => overview.value?.stats.notifications.failed ?? 0)

async function refresh(): Promise<void> {
  try {
    await dashboard.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchDashboardFailed'), error, t))
  }
}

onMounted(() => {
  void refresh()
})

const trendDays = computed(() => overview.value?.trend_7d.map((d) => d.day) ?? [])
const trendSuccess = computed(() => overview.value?.trend_7d.map((d) => d.success) ?? [])
const trendFailed = computed(() => overview.value?.trend_7d.map((d) => d.failed) ?? [])

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

function openRun(row: { run_id: string; node_id: string; job_id: string }): void {
  void router.push(
    `/n/${encodeURIComponent(row.node_id)}/jobs/${encodeURIComponent(row.job_id)}/history/runs/${encodeURIComponent(row.run_id)}`,
  )
}

function openOfflineAgents(): void {
  void router.push({ path: '/agents', query: { status: 'offline' } })
}

function openNotificationFailures(): void {
  void router.push({ path: '/settings/notifications/queue', query: { status: 'failed' } })
}

type RecentRun = DashboardOverviewResponse['recent_runs'][number]

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
        { size: 'small', tertiary: true, onClick: () => openRun(row) },
        { default: () => t('dashboard.recent.actions.open') },
      ),
  },
])
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('dashboard.title')" :subtitle="t('dashboard.subtitle')">
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
    </PageHeader>

    <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
      <n-card size="small" class="app-card">
        <div class="text-sm opacity-70">{{ t('dashboard.health.offlineAgents') }}</div>
        <div class="mt-2 flex items-baseline justify-between gap-3">
          <div class="text-3xl font-semibold tabular-nums">
            {{ offlineAgents }}
          </div>
          <n-button size="small" tertiary @click="openOfflineAgents">
            {{ t('dashboard.health.viewOfflineAgents') }}
          </n-button>
        </div>
      </n-card>

      <n-card size="small" class="app-card">
        <div class="text-sm opacity-70">{{ t('dashboard.health.notificationFailures') }}</div>
        <div class="mt-2 flex items-baseline justify-between gap-3">
          <div class="text-3xl font-semibold tabular-nums">
            {{ failedNotifications }}
          </div>
          <n-button size="small" tertiary @click="openNotificationFailures">
            {{ t('dashboard.health.viewNotificationQueue') }}
          </n-button>
        </div>
      </n-card>
    </div>

    <div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-4">
      <n-card size="small" class="app-card">
        <div class="text-sm opacity-70">{{ t('dashboard.cards.agents') }}</div>
        <div class="mt-2 flex items-baseline justify-between">
          <div class="text-3xl font-semibold tabular-nums">
            {{ overview?.stats.agents.online ?? 0 }}
          </div>
          <div class="text-sm opacity-70 tabular-nums">
            {{ t('dashboard.cards.ofActive', { active: overview?.stats.agents.active ?? 0 }) }}
          </div>
        </div>
        <div class="mt-2 text-sm opacity-70 tabular-nums">
          {{ t('dashboard.cards.offline', { count: overview?.stats.agents.offline ?? 0 }) }}
          <span v-if="(overview?.stats.agents.revoked ?? 0) > 0" class="ml-2">
            {{ t('dashboard.cards.revoked', { count: overview?.stats.agents.revoked ?? 0 }) }}
          </span>
        </div>
      </n-card>

      <n-card size="small" class="app-card">
        <div class="text-sm opacity-70">{{ t('dashboard.cards.jobs') }}</div>
        <div class="mt-2 flex items-baseline justify-between">
          <div class="text-3xl font-semibold tabular-nums">
            {{ overview?.stats.jobs.active ?? 0 }}
          </div>
          <div class="text-sm opacity-70 tabular-nums">
            {{ t('dashboard.cards.archived', { count: overview?.stats.jobs.archived ?? 0 }) }}
          </div>
        </div>
      </n-card>

      <n-card size="small" class="app-card">
        <div class="text-sm opacity-70">{{ t('dashboard.cards.runs24h') }}</div>
        <div class="mt-2 grid grid-cols-3 gap-2 text-sm">
          <div>
            <div class="text-xs opacity-70">{{ t('dashboard.cards.success') }}</div>
            <div class="text-xl font-semibold tabular-nums">{{ overview?.stats.runs.success_24h ?? 0 }}</div>
          </div>
          <div>
            <div class="text-xs opacity-70">{{ t('dashboard.cards.failed') }}</div>
            <div class="text-xl font-semibold tabular-nums">{{ overview?.stats.runs.failed_24h ?? 0 }}</div>
          </div>
          <div>
            <div class="text-xs opacity-70">{{ t('dashboard.cards.rejected') }}</div>
            <div class="text-xl font-semibold tabular-nums">{{ overview?.stats.runs.rejected_24h ?? 0 }}</div>
          </div>
        </div>
      </n-card>

      <n-card size="small" class="app-card">
        <div class="text-sm opacity-70">{{ t('dashboard.cards.live') }}</div>
        <div class="mt-2 grid grid-cols-2 gap-2 text-sm">
          <div>
            <div class="text-xs opacity-70">{{ t('dashboard.cards.running') }}</div>
            <div class="text-2xl font-semibold tabular-nums">{{ overview?.stats.runs.running ?? 0 }}</div>
          </div>
          <div>
            <div class="text-xs opacity-70">{{ t('dashboard.cards.queued') }}</div>
            <div class="text-2xl font-semibold tabular-nums">{{ overview?.stats.runs.queued ?? 0 }}</div>
          </div>
        </div>
      </n-card>
    </div>

    <n-card class="app-card" :title="t('dashboard.trend7d')">
      <div class="h-64">
        <AppEmptyState v-if="dashboard.loading && !overview" :title="t('common.loading')" loading />
        <div v-else-if="trendDays.length === 0" class="h-full flex items-center justify-center">
          <n-empty :description="t('dashboard.trendEmpty')" />
        </div>
        <BackupTrendChart v-else :days="trendDays" :success="trendSuccess" :failed="trendFailed" class="h-full" />
      </div>
    </n-card>

    <n-card class="app-card" :title="t('dashboard.recent.title')">
      <AppEmptyState v-if="dashboard.loading && (overview?.recent_runs?.length ?? 0) === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!dashboard.loading && (overview?.recent_runs?.length ?? 0) === 0" :title="t('dashboard.recent.empty')" />

      <div v-else>
        <div v-if="!isDesktop" class="space-y-3">
          <n-card
            v-for="row in overview?.recent_runs ?? []"
            :key="row.run_id"
            size="small"
            class="app-card"
          >
            <template #header>
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="font-medium truncate">{{ row.job_name }}</div>
                  <div class="text-xs opacity-70 mt-0.5 truncate">{{ nodeLabel(row) }}</div>
                </div>
                <n-tag size="small" :bordered="false" :type="statusTagType(row.status)">{{ runStatusLabel(t, row.status) }}</n-tag>
              </div>
            </template>

            <div class="text-sm">
              <div class="flex items-start justify-between gap-4 py-1">
                <div class="opacity-70">{{ t('dashboard.recent.columns.startedAt') }}</div>
                <div class="font-mono tabular-nums">{{ formatUnixSeconds(row.started_at) }}</div>
              </div>
              <div class="flex items-start justify-between gap-4 py-1">
                <div class="opacity-70">{{ t('dashboard.recent.columns.endedAt') }}</div>
                <div class="font-mono tabular-nums">{{ row.ended_at ? formatUnixSeconds(row.ended_at) : '-' }}</div>
              </div>
              <div v-if="row.error" class="mt-2 text-xs text-red-600 truncate">{{ row.error }}</div>
            </div>

            <template #footer>
              <div class="flex justify-end">
                <n-button size="small" @click="openRun(row)">{{ t('dashboard.recent.actions.open') }}</n-button>
              </div>
            </template>
          </n-card>
        </div>

        <div v-else class="overflow-x-auto">
          <n-data-table :loading="dashboard.loading" :columns="columns" :data="overview?.recent_runs ?? []" />
        </div>
      </div>
    </n-card>
  </div>
</template>
