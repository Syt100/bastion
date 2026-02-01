<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NSpin, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import { useJobDetailContext } from '@/lib/jobDetailContext'
import { useJobsStore, type RunListItem } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { runStatusLabel } from '@/lib/runs'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()

const ctx = useJobDetailContext()
const jobs = useJobsStore()
const ui = useUiStore()

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const job = computed(() => ctx.job.value)

const runsLoading = ref<boolean>(false)
const runs = ref<RunListItem[]>([])

const overlapLabel = computed(() => {
  const policy = job.value?.overlap_policy
  if (policy === 'queue') return t('jobs.overlap.queue')
  if (policy === 'reject') return t('jobs.overlap.reject')
  return '-'
})

function statusTagType(status: RunListItem['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  return 'default'
}

async function refreshRuns(): Promise<void> {
  const id = ctx.jobId.value
  if (!id) return
  runsLoading.value = true
  try {
    runs.value = await jobs.listRuns(id)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchRunsFailed'), error, t))
    runs.value = []
  } finally {
    runsLoading.value = false
  }
}

watch(
  () => ctx.jobId.value,
  (id) => {
    runs.value = []
    if (id) void refreshRuns()
  },
  { immediate: true },
)

const latestRun = computed<RunListItem | null>(() => {
  let best: RunListItem | null = null
  for (const run of runs.value) {
    if (!best || run.started_at > best.started_at) best = run
  }
  return best
})

const runs7d = computed<RunListItem[]>(() => {
  const cutoff = Math.floor(Date.now() / 1000) - 7 * 24 * 60 * 60
  return runs.value.filter((r) => r.started_at >= cutoff)
})

const runs7dTotal = computed(() => runs7d.value.length)
const runs7dSuccess = computed(() => runs7d.value.filter((r) => r.status === 'success').length)
const runs7dFailed = computed(() => runs7d.value.filter((r) => r.status === 'failed').length)
const runs7dRejected = computed(() => runs7d.value.filter((r) => r.status === 'rejected').length)

function openLatestRun(): void {
  const id = ctx.jobId.value
  const r = latestRun.value
  if (!id || !r) return
  void router.push(
    `/n/${encodeURIComponent(ctx.nodeId.value)}/jobs/${encodeURIComponent(id)}/overview/runs/${encodeURIComponent(r.id)}`,
  )
}

function goHistory(): void {
  const id = ctx.jobId.value
  if (!id) return
  void router.push(`/n/${encodeURIComponent(ctx.nodeId.value)}/jobs/${encodeURIComponent(id)}/history`)
}

function goData(): void {
  const id = ctx.jobId.value
  if (!id) return
  void router.push(`/n/${encodeURIComponent(ctx.nodeId.value)}/jobs/${encodeURIComponent(id)}/data`)
}
</script>

<template>
  <div class="space-y-3">
    <AppEmptyState v-if="ctx.loading.value && !job" :title="t('common.loading')" loading />
    <AppEmptyState v-else-if="!job" :title="t('common.noData')" />

    <template v-else>
      <n-card size="small" class="app-card" :bordered="false" data-testid="job-overview-run-summary">
        <template #header>
          <div class="text-sm font-medium">{{ t('jobs.workspace.overview.runs7dTitle') }}</div>
        </template>

        <div class="space-y-2">
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="text-xs opacity-70">{{ t('runs.latestRun') }}</div>
              <div class="mt-1 flex items-center gap-2 min-w-0">
                <n-tag
                  size="small"
                  :bordered="false"
                  :type="latestRun ? statusTagType(latestRun.status) : 'default'"
                >
                  {{ latestRun ? runStatusLabel(t, latestRun.status) : '-' }}
                </n-tag>
                <div class="font-mono tabular-nums text-sm truncate">
                  {{ latestRun ? formatUnixSeconds(latestRun.started_at) : '-' }}
                </div>
              </div>
              <div v-if="latestRun?.error" class="mt-1 text-xs text-red-600 truncate">{{ latestRun.error }}</div>
            </div>

            <n-button
              data-testid="job-overview-open-latest-run"
              size="small"
              :disabled="!latestRun"
              @click="openLatestRun"
            >
              {{ t('runs.actions.detail') }}
            </n-button>
          </div>

          <div class="flex items-center gap-2 overflow-x-auto pb-1">
            <n-tag size="small" :bordered="false">
              {{ t('jobs.workspace.overview.runs7dTotal', { n: runs7dTotal }) }}
            </n-tag>
            <n-tag size="small" :bordered="false" type="success">
              {{ runStatusLabel(t, 'success') }}: {{ runs7dSuccess }}
            </n-tag>
            <n-tag size="small" :bordered="false" type="error">
              {{ runStatusLabel(t, 'failed') }}: {{ runs7dFailed }}
            </n-tag>
            <n-tag v-if="runs7dRejected > 0" size="small" :bordered="false" type="warning">
              {{ runStatusLabel(t, 'rejected') }}: {{ runs7dRejected }}
            </n-tag>
          </div>

          <div v-if="!runsLoading && runs7dTotal === 0" class="text-xs opacity-70">
            {{ t('jobs.workspace.overview.runs7dEmpty') }}
          </div>

          <div v-if="runsLoading" class="flex justify-center py-1">
            <n-spin size="small" />
          </div>
        </div>
      </n-card>

      <div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-4">
        <n-card size="small" class="app-card" :bordered="false">
          <div class="text-xs opacity-70">{{ t('jobs.columns.schedule') }}</div>
          <div class="mt-2 font-mono tabular-nums truncate">{{ job.schedule ?? t('jobs.scheduleMode.manual') }}</div>
          <div class="mt-1 text-xs opacity-70 truncate">{{ job.schedule_timezone }}</div>
        </n-card>

        <n-card size="small" class="app-card" :bordered="false">
          <div class="text-xs opacity-70">{{ t('jobs.columns.overlap') }}</div>
          <div class="mt-2 flex items-center gap-2">
            <n-tag size="small" :bordered="false">{{ overlapLabel }}</n-tag>
          </div>
        </n-card>

        <n-card size="small" class="app-card" :bordered="false">
          <div class="text-xs opacity-70">{{ t('jobs.columns.node') }}</div>
          <div class="mt-2 font-medium truncate">{{ ctx.nodeId.value === 'hub' ? t('jobs.nodes.hub') : ctx.nodeId.value }}</div>
        </n-card>

        <n-card size="small" class="app-card" :bordered="false">
          <div class="text-xs opacity-70">{{ t('jobs.workspace.quickLinks') }}</div>
          <div class="mt-2 flex flex-wrap gap-2">
            <n-button size="small" @click="goHistory">{{ t('jobs.workspace.sections.history') }}</n-button>
            <n-button size="small" @click="goData">{{ t('jobs.workspace.sections.data') }}</n-button>
          </div>
        </n-card>
      </div>
    </template>
  </div>
</template>
