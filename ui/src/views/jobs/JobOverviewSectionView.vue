<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import { useJobDetailContext } from '@/lib/jobDetailContext'

const { t } = useI18n()
const router = useRouter()

const ctx = useJobDetailContext()

const job = computed(() => ctx.job.value)

const overlapLabel = computed(() => {
  const policy = job.value?.overlap_policy
  if (policy === 'queue') return t('jobs.overlap.queue')
  if (policy === 'reject') return t('jobs.overlap.reject')
  return '-'
})

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

