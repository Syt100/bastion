<script setup lang="ts">
import { computed, defineAsyncComponent, h, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  NButton,
  NCard,
  NDataTable,
  NEmpty,
  NSkeleton,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import { useDashboardStore, type DashboardOverviewResponse } from '@/stores/dashboard'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { runStatusLabel } from '@/lib/runs'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'

const BackupTrendChart = defineAsyncComponent(() => import('@/components/BackupTrendChart.vue'))

const { t } = useI18n()
const message = useMessage()
const router = useRouter()

const ui = useUiStore()
const dashboard = useDashboardStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const overview = computed(() => dashboard.overview)
const showInitialSkeleton = computed(() => dashboard.loading && !overview.value)
const offlineAgents = computed(() => overview.value?.stats.agents.offline ?? 0)
const failedNotifications = computed(() => overview.value?.stats.notifications.failed ?? 0)

const trendDays = computed(() => overview.value?.trend_7d.map((d) => d.day) ?? [])
const trendSuccess = computed(() => overview.value?.trend_7d.map((d) => d.success) ?? [])
const trendFailed = computed(() => overview.value?.trend_7d.map((d) => d.failed) ?? [])

const trendChartTarget = ref<HTMLElement | null>(null)
const trendChartReady = ref(false)
let trendChartObserver: IntersectionObserver | null = null

function stopTrendChartObserver(): void {
  trendChartObserver?.disconnect()
  trendChartObserver = null
}

function ensureTrendChartWhenVisible(): void {
  if (trendChartReady.value) return
  if (trendDays.value.length === 0) return

  const target = trendChartTarget.value
  if (!target) return

  if (typeof window === 'undefined' || typeof window.IntersectionObserver === 'undefined') {
    trendChartReady.value = true
    return
  }

  if (trendChartObserver) return

  trendChartObserver = new window.IntersectionObserver(
    (entries) => {
      if (!entries.some((entry) => entry.isIntersecting)) return
      trendChartReady.value = true
      stopTrendChartObserver()
    },
    {
      root: null,
      // Start preparing chart slightly before users reach the chart area.
      rootMargin: '200px 0px',
    },
  )

  trendChartObserver.observe(target)
}

watch([trendDays, trendChartTarget], () => {
  if (trendDays.value.length === 0) {
    trendChartReady.value = false
    stopTrendChartObserver()
    return
  }
  ensureTrendChartWhenVisible()
})

async function refresh(): Promise<void> {
  try {
    await dashboard.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchDashboardFailed'), error, t))
  }
}

onMounted(() => {
  void refresh()
  ensureTrendChartWhenVisible()
})

onBeforeUnmount(() => {
  stopTrendChartObserver()
})

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
      <n-card size="small" class="app-card" :bordered="false">
        <div class="text-sm app-text-muted">{{ t('dashboard.health.offlineAgents') }}</div>
        <div class="mt-2 flex items-baseline justify-between gap-3">
          <div class="text-3xl font-semibold tabular-nums">
            <n-skeleton v-if="showInitialSkeleton" text width="3rem" />
            <template v-else>{{ offlineAgents }}</template>
          </div>
          <n-skeleton v-if="showInitialSkeleton" text width="8rem" />
          <n-button v-else size="small" tertiary @click="openOfflineAgents">
            {{ t('dashboard.health.viewOfflineAgents') }}
          </n-button>
        </div>
      </n-card>

      <n-card size="small" class="app-card" :bordered="false">
        <div class="text-sm app-text-muted">{{ t('dashboard.health.notificationFailures') }}</div>
        <div class="mt-2 flex items-baseline justify-between gap-3">
          <div class="text-3xl font-semibold tabular-nums">
            <n-skeleton v-if="showInitialSkeleton" text width="3rem" />
            <template v-else>{{ failedNotifications }}</template>
          </div>
          <n-skeleton v-if="showInitialSkeleton" text width="9rem" />
          <n-button v-else size="small" tertiary @click="openNotificationFailures">
            {{ t('dashboard.health.viewNotificationQueue') }}
          </n-button>
        </div>
      </n-card>
    </div>

    <div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-4">
      <n-card size="small" class="app-card" :bordered="false">
        <div class="text-sm app-text-muted">{{ t('dashboard.cards.agents') }}</div>
        <div class="mt-2 flex items-baseline justify-between">
          <div class="text-3xl font-semibold tabular-nums">
            <n-skeleton v-if="showInitialSkeleton" text width="3.5rem" />
            <template v-else>{{ overview?.stats.agents.online ?? 0 }}</template>
          </div>
          <div class="text-sm app-text-muted tabular-nums">
            <n-skeleton v-if="showInitialSkeleton" text width="5rem" />
            <template v-else>{{ t('dashboard.cards.ofActive', { active: overview?.stats.agents.active ?? 0 }) }}</template>
          </div>
        </div>
        <div class="mt-2 text-sm app-text-muted tabular-nums">
          <n-skeleton v-if="showInitialSkeleton" text width="8rem" />
          <template v-else>
            {{ t('dashboard.cards.offline', { count: overview?.stats.agents.offline ?? 0 }) }}
            <span v-if="(overview?.stats.agents.revoked ?? 0) > 0" class="ml-2">
              {{ t('dashboard.cards.revoked', { count: overview?.stats.agents.revoked ?? 0 }) }}
            </span>
          </template>
        </div>
      </n-card>

      <n-card size="small" class="app-card" :bordered="false">
        <div class="text-sm app-text-muted">{{ t('dashboard.cards.jobs') }}</div>
        <div class="mt-2 flex items-baseline justify-between">
          <div class="text-3xl font-semibold tabular-nums">
            <n-skeleton v-if="showInitialSkeleton" text width="3.5rem" />
            <template v-else>{{ overview?.stats.jobs.active ?? 0 }}</template>
          </div>
          <div class="text-sm app-text-muted tabular-nums">
            <n-skeleton v-if="showInitialSkeleton" text width="4.5rem" />
            <template v-else>{{ t('dashboard.cards.archived', { count: overview?.stats.jobs.archived ?? 0 }) }}</template>
          </div>
        </div>
      </n-card>

      <n-card size="small" class="app-card" :bordered="false">
        <div class="text-sm app-text-muted">{{ t('dashboard.cards.runs24h') }}</div>
        <div class="mt-2 grid grid-cols-3 gap-2 text-sm">
          <div>
            <div class="text-xs app-text-muted">{{ t('dashboard.cards.success') }}</div>
            <div class="text-xl font-semibold tabular-nums">
              <n-skeleton v-if="showInitialSkeleton" text width="2.5rem" />
              <template v-else>{{ overview?.stats.runs.success_24h ?? 0 }}</template>
            </div>
          </div>
          <div>
            <div class="text-xs app-text-muted">{{ t('dashboard.cards.failed') }}</div>
            <div class="text-xl font-semibold tabular-nums">
              <n-skeleton v-if="showInitialSkeleton" text width="2.5rem" />
              <template v-else>{{ overview?.stats.runs.failed_24h ?? 0 }}</template>
            </div>
          </div>
          <div>
            <div class="text-xs app-text-muted">{{ t('dashboard.cards.rejected') }}</div>
            <div class="text-xl font-semibold tabular-nums">
              <n-skeleton v-if="showInitialSkeleton" text width="2.5rem" />
              <template v-else>{{ overview?.stats.runs.rejected_24h ?? 0 }}</template>
            </div>
          </div>
        </div>
      </n-card>

      <n-card size="small" class="app-card" :bordered="false">
        <div class="text-sm app-text-muted">{{ t('dashboard.cards.live') }}</div>
        <div class="mt-2 grid grid-cols-2 gap-2 text-sm">
          <div>
            <div class="text-xs app-text-muted">{{ t('dashboard.cards.running') }}</div>
            <div class="text-2xl font-semibold tabular-nums">
              <n-skeleton v-if="showInitialSkeleton" text width="3rem" />
              <template v-else>{{ overview?.stats.runs.running ?? 0 }}</template>
            </div>
          </div>
          <div>
            <div class="text-xs app-text-muted">{{ t('dashboard.cards.queued') }}</div>
            <div class="text-2xl font-semibold tabular-nums">
              <n-skeleton v-if="showInitialSkeleton" text width="3rem" />
              <template v-else>{{ overview?.stats.runs.queued ?? 0 }}</template>
            </div>
          </div>
        </div>
      </n-card>
    </div>

    <n-card class="app-card" :bordered="false" :title="t('dashboard.trend7d')">
      <div ref="trendChartTarget" class="h-64">
        <AppEmptyState v-if="showInitialSkeleton" :title="t('common.loading')" loading />
        <div v-else-if="trendDays.length === 0" class="h-full flex items-center justify-center">
          <n-empty :description="t('dashboard.trendEmpty')" />
        </div>
        <div v-else-if="!trendChartReady" class="h-full flex items-center justify-center px-4">
          <div class="w-full max-w-[30rem] space-y-2">
            <n-skeleton text :repeat="4" />
          </div>
        </div>
        <Suspense v-else>
          <template #default>
            <BackupTrendChart :days="trendDays" :success="trendSuccess" :failed="trendFailed" class="h-full" />
          </template>
          <template #fallback>
            <AppEmptyState :title="t('common.loading')" loading />
          </template>
        </Suspense>
      </div>
    </n-card>

    <n-card class="app-card" :bordered="false" :title="t('dashboard.recent.title')">
      <AppEmptyState v-if="dashboard.loading && (overview?.recent_runs?.length ?? 0) === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!dashboard.loading && (overview?.recent_runs?.length ?? 0) === 0" :title="t('dashboard.recent.empty')" />

      <div v-else>
        <div v-if="!isDesktop" class="space-y-3">
          <n-card
            v-for="row in overview?.recent_runs ?? []"
            :key="row.run_id"
            size="small"
            class="app-card"
            :bordered="false"
          >
            <template #header>
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="font-medium truncate">{{ row.job_name }}</div>
                  <div class="text-xs app-text-muted mt-0.5 truncate">{{ nodeLabel(row) }}</div>
                </div>
                <n-tag size="small" :bordered="false" :type="statusTagType(row.status)">{{ runStatusLabel(t, row.status) }}</n-tag>
              </div>
            </template>

            <div class="text-sm">
              <div class="flex items-start justify-between gap-4 py-1">
                <div class="app-text-muted">{{ t('dashboard.recent.columns.startedAt') }}</div>
                <div class="font-mono tabular-nums">{{ formatUnixSeconds(row.started_at) }}</div>
              </div>
              <div class="flex items-start justify-between gap-4 py-1">
                <div class="app-text-muted">{{ t('dashboard.recent.columns.endedAt') }}</div>
                <div class="font-mono tabular-nums">{{ row.ended_at ? formatUnixSeconds(row.ended_at) : '-' }}</div>
              </div>
              <div v-if="row.error" class="mt-2 text-xs text-[var(--app-danger)] truncate">{{ row.error }}</div>
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
