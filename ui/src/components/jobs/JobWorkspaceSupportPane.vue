<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NCard, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { runStatusLabel } from '@/lib/runs'
import type { JobWorkspaceDetail } from '@/stores/jobs'

const props = defineProps<{
  workspace: JobWorkspaceDetail | null
  loading: boolean
}>()

const emit = defineEmits<{
  'open-run': [runId: string]
}>()

const { t } = useI18n()
const ui = useUiStore()
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const readinessType = computed<'success' | 'warning' | 'error' | 'default'>(() => {
  const state = props.workspace?.readiness.state
  if (state === 'healthy') return 'success'
  if (state === 'critical') return 'error'
  if (state === 'warning') return 'warning'
  return 'default'
})

const readinessLabel = computed(() => {
  const state = props.workspace?.readiness.state
  if (state === 'healthy') return t('jobs.workspace.support.healthHealthy')
  if (state === 'critical') return t('jobs.workspace.support.healthCritical')
  if (state === 'warning') return t('jobs.workspace.support.healthWarning')
  if (state === 'archived') return t('jobs.archived')
  return t('common.noData')
})
</script>

<template>
  <div class="space-y-3">
    <AppEmptyState v-if="loading && !workspace" :title="t('common.loading')" loading />
    <AppEmptyState v-else-if="!workspace" :title="t('common.noData')" />

    <template v-else>
      <n-card size="small" class="app-card" :bordered="false">
        <div class="flex items-center justify-between gap-3">
          <div>
            <div class="text-sm font-medium">{{ t('jobs.workspace.support.healthTitle') }}</div>
            <div class="mt-1 text-xs app-text-muted">{{ t('jobs.workspace.support.healthSubtitle') }}</div>
          </div>
          <n-tag size="small" :bordered="false" :type="readinessType">
            {{ readinessLabel }}
          </n-tag>
        </div>

        <div class="mt-3 space-y-2 text-sm">
          <div class="flex items-start justify-between gap-3">
            <span class="app-text-muted">{{ t('jobs.workspace.support.latestSuccess') }}</span>
            <span class="font-mono tabular-nums text-right">
              {{ workspace.summary.latest_success_at ? formatUnixSeconds(workspace.summary.latest_success_at) : '-' }}
            </span>
          </div>
          <div class="flex items-start justify-between gap-3">
            <span class="app-text-muted">{{ t('jobs.workspace.support.nextRun') }}</span>
            <span class="font-mono tabular-nums text-right">
              {{ workspace.summary.next_run_at ? formatUnixSeconds(workspace.summary.next_run_at) : t('jobs.scheduleMode.manual') }}
            </span>
          </div>
          <div class="flex items-start justify-between gap-3">
            <span class="app-text-muted">{{ t('jobs.workspace.support.target') }}</span>
            <span class="text-right break-all">
              {{ workspace.summary.target_label || workspace.summary.target_type || '-' }}
            </span>
          </div>
        </div>
      </n-card>

      <n-card size="small" class="app-card" :bordered="false">
        <div class="text-sm font-medium">{{ t('jobs.workspace.support.recentRuns') }}</div>
        <div class="mt-3 space-y-2">
          <AppEmptyState
            v-if="workspace.recent_runs.length === 0"
            :title="t('jobs.workspace.support.noRecentRuns')"
            variant="inset"
          />
          <button
            v-for="run in workspace.recent_runs.slice(0, 5)"
            :key="run.id"
            type="button"
            class="w-full rounded-lg app-border-subtle px-3 py-2 text-left app-motion-soft hover:bg-[var(--app-pressed)]"
            @click="emit('open-run', run.id)"
          >
            <div class="flex items-center justify-between gap-3">
              <n-tag
                size="small"
                :bordered="false"
                :type="run.status === 'success' ? 'success' : run.status === 'failed' ? 'error' : run.status === 'rejected' ? 'warning' : 'default'"
              >
                {{ runStatusLabel(t, run.status) }}
              </n-tag>
              <span class="font-mono tabular-nums text-xs app-text-muted">
                {{ formatUnixSeconds(run.started_at) }}
              </span>
            </div>
            <div class="mt-1 text-xs break-all" :class="run.error ? 'text-[var(--app-danger)]' : 'app-text-muted'">
              {{ run.error || run.id }}
            </div>
          </button>
        </div>
      </n-card>

      <n-card size="small" class="app-card" :bordered="false">
        <div class="text-sm font-medium">{{ t('jobs.workspace.support.warningsTitle') }}</div>
        <div class="mt-3 flex flex-wrap gap-2">
          <n-tag v-if="workspace.warnings.length === 0" size="small" :bordered="false" type="success">
            {{ t('jobs.workspace.support.noWarnings') }}
          </n-tag>
          <n-tag
            v-for="warning in workspace.warnings"
            :key="warning"
            size="small"
            :bordered="false"
            :type="warning === 'latest_run_failed' || warning === 'no_successful_backup' ? 'error' : 'warning'"
          >
            {{ t(`jobs.workspace.support.warningLabels.${warning}`) }}
          </n-tag>
        </div>
        <div class="mt-3">
          <n-button
            block
            size="small"
            quaternary
            :disabled="workspace.recent_runs.length === 0"
            @click="workspace.recent_runs[0] && emit('open-run', workspace.recent_runs[0].id)"
          >
            {{ t('jobs.workspace.support.openLatestRun') }}
          </n-button>
        </div>
      </n-card>
    </template>
  </div>
</template>
