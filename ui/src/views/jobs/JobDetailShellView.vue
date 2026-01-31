<script setup lang="ts">
import { computed, provide, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NTabPane, NTabs, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import { useJobsStore, type JobDetail } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { JOB_DETAIL_CONTEXT } from '@/lib/jobDetailContext'

const { t } = useI18n()
const message = useMessage()
const route = useRoute()
const router = useRouter()

const ui = useUiStore()
const jobs = useJobsStore()

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : 'hub'))
const jobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))

const loading = ref<boolean>(false)
const job = ref<JobDetail | null>(null)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

async function refresh(): Promise<void> {
  if (!jobId.value) return
  loading.value = true
  try {
    job.value = await jobs.getJob(jobId.value)
  } catch (error) {
    job.value = null
    message.error(formatToastError(t('errors.fetchJobFailed'), error, t))
  } finally {
    loading.value = false
  }
}

function backToJobs(): void {
  void router.push(`/n/${encodeURIComponent(nodeId.value)}/jobs`)
}

const activeTab = computed<'runs' | 'snapshots' | 'retention' | 'settings'>(() => {
  const path = route.path
  if (path.endsWith('/snapshots')) return 'snapshots'
  if (path.endsWith('/retention')) return 'retention'
  if (path.endsWith('/settings')) return 'settings'
  return 'runs'
})

function navigateTab(key: string | number): void {
  if (typeof key !== 'string') return
  if (!jobId.value) return

  const base = `/n/${encodeURIComponent(nodeId.value)}/jobs/${encodeURIComponent(jobId.value)}`
  if (key === 'runs') {
    void router.push(base)
    return
  }
  void router.push(`${base}/${encodeURIComponent(key)}`)
}

provide(JOB_DETAIL_CONTEXT, { nodeId, jobId, job, loading, refresh })

watch(jobId, () => void refresh(), { immediate: true })

const jobTypeLabel = computed(() => {
  const type = job.value?.spec?.type
  if (type === 'filesystem' || type === 'sqlite' || type === 'vaultwarden') return t(`jobs.types.${type}`)
  return type ? String(type) : '-'
})

const overlapLabel = computed(() => {
  const policy = job.value?.overlap_policy
  if (policy === 'queue') return t('jobs.overlap.queue')
  if (policy === 'reject') return t('jobs.overlap.reject')
  return '-'
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="job?.name ?? t('jobs.detail.title')">
      <template #prefix>
        <NodeContextTag :node-id="nodeId" />
      </template>
      <template #subtitle>
        <p v-if="!job" class="text-sm opacity-70">{{ t('jobs.detail.subtitle') }}</p>
        <div v-else class="flex flex-wrap items-center gap-2 text-sm opacity-70">
          <span class="font-mono tabular-nums">{{ job.id }}</span>
          <n-tag size="small" :bordered="false" type="info">{{ jobTypeLabel }}</n-tag>
          <n-tag v-if="job.archived_at" size="small" :bordered="false" type="warning">{{ t('jobs.archived') }}</n-tag>
        </div>
      </template>
      <n-button :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button @click="backToJobs">{{ t('common.return') }}</n-button>
    </PageHeader>

    <AppEmptyState v-if="loading && !job" :title="t('common.loading')" loading />
    <AppEmptyState v-else-if="!loading && !job" :title="t('common.noData')" />

    <template v-else>
      <n-card size="small" class="app-card">
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3 text-sm">
          <div class="min-w-0">
            <div class="text-xs opacity-70">{{ t('jobs.fields.type') }}</div>
            <div class="mt-1 font-medium truncate">{{ jobTypeLabel }}</div>
          </div>
          <div class="min-w-0">
            <div class="text-xs opacity-70">{{ t('jobs.columns.schedule') }}</div>
            <div class="mt-1 font-mono tabular-nums truncate">{{ job.schedule ?? t('jobs.scheduleMode.manual') }}</div>
            <div class="mt-0.5 text-xs opacity-70 truncate">{{ job.schedule_timezone }}</div>
          </div>
          <div class="min-w-0">
            <div class="text-xs opacity-70">{{ t('jobs.columns.overlap') }}</div>
            <div class="mt-1 font-medium truncate">{{ overlapLabel }}</div>
          </div>
          <div class="min-w-0">
            <div class="text-xs opacity-70">{{ t('jobs.columns.updatedAt') }}</div>
            <div class="mt-1 font-mono tabular-nums truncate">{{ formatUnixSeconds(job.updated_at) }}</div>
          </div>
        </div>
      </n-card>

      <n-card class="app-card" :bordered="false">
        <n-tabs :value="activeTab" type="line" size="small" :pane-style="{ display: 'none' }" @update:value="navigateTab">
          <n-tab-pane name="runs" :tab="t('jobs.detail.tabs.runs')" />
          <n-tab-pane name="snapshots" :tab="t('jobs.detail.tabs.snapshots')" />
          <n-tab-pane name="retention" :tab="t('jobs.detail.tabs.retention')" />
          <n-tab-pane name="settings" :tab="t('jobs.detail.tabs.settings')" />
        </n-tabs>
      </n-card>

      <router-view />
    </template>
  </div>
</template>
