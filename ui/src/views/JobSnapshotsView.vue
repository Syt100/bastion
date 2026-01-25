<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NDataTable, NSpace, NTag, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import { useJobsStore, type JobDetail, type RunArtifact } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatToastError } from '@/lib/errors'
import { formatBytes } from '@/lib/format'

const { t } = useI18n()
const message = useMessage()

const route = useRoute()
const router = useRouter()
const ui = useUiStore()
const jobs = useJobsStore()

const isDesktop = useMediaQuery(MQ.mdUp)
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : null))
const nodeIdOrHub = computed(() => nodeId.value ?? 'hub')
const jobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))

const job = ref<JobDetail | null>(null)
const loading = ref<boolean>(false)
const items = ref<RunArtifact[]>([])

function openRunDetail(runId: string): void {
  void router.push(`/n/${encodeURIComponent(nodeIdOrHub.value)}/runs/${encodeURIComponent(runId)}`)
}

function formatTarget(row: RunArtifact): string {
  const snap = row.target_snapshot as any
  const target = snap?.target

  if (row.target_type === 'local_dir') {
    const baseDir = typeof target?.base_dir === 'string' ? target.base_dir : ''
    return baseDir ? `${t('snapshots.targets.localDir')}: ${baseDir}` : t('snapshots.targets.localDir')
  }
  if (row.target_type === 'webdav') {
    const baseUrl = typeof target?.base_url === 'string' ? target.base_url : ''
    return baseUrl ? `${t('snapshots.targets.webdav')}: ${baseUrl}` : t('snapshots.targets.webdav')
  }
  return row.target_type
}

function formatStatus(row: RunArtifact): { label: string; type: 'default' | 'success' | 'warning' | 'error' } {
  const s = row.status
  if (s === 'present') return { label: t('snapshots.status.present'), type: 'success' }
  if (s === 'deleting') return { label: t('snapshots.status.deleting'), type: 'warning' }
  if (s === 'deleted') return { label: t('snapshots.status.deleted'), type: 'default' }
  if (s === 'missing') return { label: t('snapshots.status.missing'), type: 'warning' }
  if (s === 'error') return { label: t('snapshots.status.error'), type: 'error' }
  return { label: String(s), type: 'default' }
}

async function refresh(): Promise<void> {
  const id = jobId.value
  if (!id) return
  loading.value = true
  try {
    job.value = await jobs.getJob(id)
    const res = await jobs.listJobSnapshots(id, { limit: 200 })
    items.value = res.items
  } catch (error) {
    message.error(formatToastError(t('errors.fetchSnapshotsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void refresh()
})

watch(jobId, () => {
  void refresh()
})

const columns = computed<DataTableColumns<RunArtifact>>(() => {
  const cols: DataTableColumns<RunArtifact> = [
    {
      title: t('snapshots.columns.endedAt'),
      key: 'ended_at',
      render: (row) => h('span', { class: 'font-mono tabular-nums' }, formatUnixSeconds(row.ended_at)),
    },
    {
      title: t('snapshots.columns.status'),
      key: 'status',
      render: (row) => {
        const s = formatStatus(row)
        return h(NTag, { size: 'small', bordered: false, type: s.type }, { default: () => s.label })
      },
    },
    {
      title: t('snapshots.columns.format'),
      key: 'artifact_format',
      render: (row) => h('span', { class: 'font-mono' }, row.artifact_format ?? '-'),
    },
    {
      title: t('snapshots.columns.target'),
      key: 'target',
      render: (row) => h('span', { class: 'truncate' }, formatTarget(row)),
    },
    {
      title: t('snapshots.columns.source'),
      key: 'source',
      render: (row) => {
        const files = row.source_files ?? null
        const dirs = row.source_dirs ?? null
        const bytes = row.source_bytes ?? null
        const parts: string[] = []
        if (files != null) parts.push(`${files}${t('snapshots.units.files')}`)
        if (dirs != null) parts.push(`${dirs}${t('snapshots.units.dirs')}`)
        if (bytes != null) parts.push(formatBytes(bytes))
        return parts.length ? parts.join(' / ') : '-'
      },
    },
    {
      title: t('snapshots.columns.transfer'),
      key: 'transfer',
      render: (row) => (row.transfer_bytes != null ? formatBytes(row.transfer_bytes) : '-'),
    },
    {
      title: t('snapshots.columns.actions'),
      key: 'actions',
      render: (row) =>
        h(
          NSpace,
          { size: 8 },
          {
            default: () => [
              h(
                NButton,
                { size: 'small', onClick: () => openRunDetail(row.run_id) },
                { default: () => t('snapshots.actions.viewRun') },
              ),
            ],
          },
        ),
    },
  ]
  return cols
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      :title="t('snapshots.title')"
      :subtitle="job ? `${t('snapshots.subtitlePrefix')}: ${job.name}` : t('snapshots.subtitle')"
    >
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button @click="$router.push(`/n/${encodeURIComponent(nodeIdOrHub)}/jobs`)">{{ t('common.return') }}</n-button>
    </PageHeader>

    <div v-if="!isDesktop" class="space-y-3">
      <AppEmptyState v-if="loading && items.length === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!loading && items.length === 0" :title="t('common.noData')" />

      <n-card v-for="row in items" :key="row.run_id" size="small" class="app-card">
        <template #header>
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="text-sm font-mono tabular-nums">{{ formatUnixSeconds(row.ended_at) }}</div>
              <div class="text-xs opacity-70 mt-0.5 truncate">{{ formatTarget(row) }}</div>
            </div>
            <n-tag size="small" :bordered="false" :type="formatStatus(row).type">{{ formatStatus(row).label }}</n-tag>
          </div>
        </template>

        <div class="text-sm">
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('snapshots.columns.format') }}</div>
            <div class="font-mono">{{ row.artifact_format ?? '-' }}</div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('snapshots.columns.source') }}</div>
            <div class="text-right">
              <span v-if="row.source_files != null">{{ row.source_files }}{{ t('snapshots.units.files') }}</span>
              <span v-else>-</span>
              <span v-if="row.source_dirs != null"> / {{ row.source_dirs }}{{ t('snapshots.units.dirs') }}</span>
              <span v-if="row.source_bytes != null"> / {{ formatBytes(row.source_bytes) }}</span>
            </div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('snapshots.columns.transfer') }}</div>
            <div class="text-right">{{ row.transfer_bytes != null ? formatBytes(row.transfer_bytes) : '-' }}</div>
          </div>
        </div>

        <template #footer>
          <div class="flex justify-end gap-2">
            <n-button size="small" @click="openRunDetail(row.run_id)">{{ t('snapshots.actions.viewRun') }}</n-button>
          </div>
        </template>
      </n-card>
    </div>

    <div v-else>
      <n-card class="app-card">
        <div class="overflow-x-auto">
          <n-data-table :loading="loading" :columns="columns" :data="items" />
        </div>
      </n-card>
    </div>
  </div>
</template>
