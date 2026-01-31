<script setup lang="ts">
import { computed, provide, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NTabPane, NTabs, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import { useJobsStore, type JobDetail } from '@/stores/jobs'
import { formatToastError } from '@/lib/errors'
import { JOB_DETAIL_CONTEXT } from '@/lib/jobDetailContext'

const { t } = useI18n()
const message = useMessage()
const route = useRoute()
const router = useRouter()

const jobs = useJobsStore()

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : 'hub'))
const jobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))

const loading = ref<boolean>(false)
const job = ref<JobDetail | null>(null)

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
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      :title="job?.name ?? t('jobs.detail.title')"
      :subtitle="job ? job.id : t('jobs.detail.subtitle')"
    >
      <template #prefix>
        <NodeContextTag :node-id="nodeId" />
      </template>
      <n-button :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button @click="backToJobs">{{ t('common.return') }}</n-button>
    </PageHeader>

    <AppEmptyState v-if="loading && !job" :title="t('common.loading')" loading />
    <AppEmptyState v-else-if="!loading && !job" :title="t('common.noData')" />

    <template v-else>
      <n-card class="app-card">
        <n-tabs :value="activeTab" type="line" @update:value="navigateTab">
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
