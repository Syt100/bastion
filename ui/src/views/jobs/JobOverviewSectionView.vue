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

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object' && !Array.isArray(value)
}

type MetaCard = {
  label: string
  value: string
  tagType: 'default' | 'info' | 'success' | 'warning' | 'error'
  hint?: string
}

const metaSourceType = computed<MetaCard>(() => {
  const type = job.value?.spec?.type
  if (type === 'filesystem') return { label: t('jobs.workspace.overview.cards.sourceType'), value: t('jobs.types.filesystem'), tagType: 'info' }
  if (type === 'sqlite') return { label: t('jobs.workspace.overview.cards.sourceType'), value: t('jobs.types.sqlite'), tagType: 'warning' }
  if (type === 'vaultwarden') return { label: t('jobs.workspace.overview.cards.sourceType'), value: t('jobs.types.vaultwarden'), tagType: 'default' }
  return { label: t('jobs.workspace.overview.cards.sourceType'), value: type ? String(type) : '-', tagType: 'default' }
})

const metaTargetType = computed<MetaCard>(() => {
  const spec = job.value?.spec as Record<string, unknown> | undefined
  const target = isRecord(spec?.target) ? spec?.target : null
  const type = isRecord(target) && typeof target.type === 'string' ? target.type : null
  if (type === 'webdav') return { label: t('jobs.workspace.overview.cards.targetType'), value: t('jobs.targets.webdav'), tagType: 'info' }
  if (type === 'local_dir') return { label: t('jobs.workspace.overview.cards.targetType'), value: t('jobs.targets.localDir'), tagType: 'default' }
  return { label: t('jobs.workspace.overview.cards.targetType'), value: type ? String(type) : '-', tagType: 'default' }
})

const metaArtifactFormat = computed<MetaCard>(() => {
  const spec = job.value?.spec as Record<string, unknown> | undefined
  const pipeline = isRecord(spec?.pipeline) ? spec?.pipeline : null
  const formatRaw = isRecord(pipeline) && typeof pipeline.format === 'string' ? pipeline.format : 'archive_v1'
  const value = formatRaw === 'raw_tree_v1' ? 'raw_tree_v1' : 'archive_v1'
  return {
    label: t('jobs.workspace.overview.cards.backupFormat'),
    value,
    tagType: value === 'archive_v1' ? 'info' : 'default',
    mono: true,
  }
})

const metaEncryption = computed<MetaCard>(() => {
  const spec = job.value?.spec as Record<string, unknown> | undefined
  const pipeline = isRecord(spec?.pipeline) ? spec?.pipeline : null
  const formatRaw = isRecord(pipeline) && typeof pipeline.format === 'string' ? pipeline.format : 'archive_v1'
  const format = formatRaw === 'raw_tree_v1' ? 'raw_tree_v1' : 'archive_v1'

  const enc = isRecord(pipeline) && isRecord(pipeline.encryption) ? pipeline.encryption : null
  const encType = isRecord(enc) && typeof enc.type === 'string' ? enc.type : 'none'
  const enabled = format !== 'raw_tree_v1' && encType === 'age_x25519'
  const keyName = enabled && isRecord(enc) && typeof enc.key_name === 'string' && enc.key_name.trim() ? enc.key_name.trim() : 'default'

  return {
    label: t('jobs.workspace.overview.cards.encryption'),
    value: enabled ? t('jobs.workspace.overview.encryption.enabled') : t('jobs.workspace.overview.encryption.disabled'),
    tagType: enabled ? 'success' : 'default',
    hint: enabled ? t('jobs.workspace.overview.encryption.key', { name: keyName }) : undefined,
  }
})

function openLatestRun(): void {
  const id = ctx.jobId.value
  const r = latestRun.value
  if (!id || !r) return
  void router.push(
    `/n/${encodeURIComponent(ctx.nodeId.value)}/jobs/${encodeURIComponent(id)}/overview/runs/${encodeURIComponent(r.id)}`,
  )
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

      <div class="grid grid-cols-2 gap-2 md:grid-cols-4">
        <n-card size="small" class="app-card" :bordered="false" data-testid="job-overview-meta-source">
          <div class="text-xs opacity-70">{{ metaSourceType.label }}</div>
          <div class="mt-2">
            <n-tag size="medium" :bordered="false" :type="metaSourceType.tagType" class="text-base">
              {{ metaSourceType.value }}
            </n-tag>
          </div>
        </n-card>

        <n-card size="small" class="app-card" :bordered="false" data-testid="job-overview-meta-target">
          <div class="text-xs opacity-70">{{ metaTargetType.label }}</div>
          <div class="mt-2">
            <n-tag size="medium" :bordered="false" :type="metaTargetType.tagType" class="text-base">
              {{ metaTargetType.value }}
            </n-tag>
          </div>
        </n-card>

        <n-card size="small" class="app-card" :bordered="false" data-testid="job-overview-meta-format">
          <div class="text-xs opacity-70">{{ metaArtifactFormat.label }}</div>
          <div class="mt-2">
            <n-tag
              size="medium"
              :bordered="false"
              :type="metaArtifactFormat.tagType"
              class="text-base font-mono"
            >
              {{ metaArtifactFormat.value }}
            </n-tag>
          </div>
        </n-card>

        <n-card size="small" class="app-card" :bordered="false" data-testid="job-overview-meta-encryption">
          <div class="text-xs opacity-70">{{ metaEncryption.label }}</div>
          <div class="mt-2">
            <n-tag size="medium" :bordered="false" :type="metaEncryption.tagType" class="text-base">
              {{ metaEncryption.value }}
            </n-tag>
          </div>
          <div v-if="metaEncryption.hint" class="mt-1 text-xs opacity-70 font-mono truncate">
            {{ metaEncryption.hint }}
          </div>
        </n-card>
      </div>
    </template>
  </div>
</template>
