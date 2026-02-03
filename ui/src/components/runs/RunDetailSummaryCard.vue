<script setup lang="ts">
import { computed } from 'vue'
import { NAlert, NCard, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import type { RunDetail, RunEvent } from '@/stores/jobs'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { runTargetTypeLabel } from '@/lib/runs'
import { parseRunSummary } from '@/lib/run_summary'
import RunProgressPanel from '@/components/runs/RunProgressPanel.vue'

const props = defineProps<{
  run: RunDetail | null
  events: RunEvent[]
}>()

const { t } = useI18n()
const ui = useUiStore()
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const parsedSummary = computed(() => parseRunSummary(props.run?.summary))
const targetTypeLabel = computed(() => runTargetTypeLabel(t, parsedSummary.value.targetType))

function formatDuration(seconds: number | null): string {
  if (seconds == null || !Number.isFinite(seconds) || seconds < 0) return '-'
  const s = Math.floor(seconds)
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = s % 60
  if (h > 0) return `${h}h ${m}m ${sec}s`
  if (m > 0) return `${m}m ${sec}s`
  return `${sec}s`
}

const durationSeconds = computed<number | null>(() => {
  const r = props.run
  if (!r) return null
  if (!r.started_at || !r.ended_at) return null
  return Math.max(0, r.ended_at - r.started_at)
})
</script>

<template>
  <n-card :title="t('runs.detail.overviewTitle')" size="small" class="app-card" :bordered="false" data-testid="run-detail-summary">
    <div v-if="!run" class="text-sm app-text-muted">-</div>
    <div v-else class="space-y-2">
      <n-alert v-if="run.error" type="error" :title="t('runs.columns.error')" :bordered="false">
        {{ run.error }}
      </n-alert>

      <div class="space-y-1.5" data-testid="run-detail-overview">
        <div class="flex flex-wrap items-center gap-2">
          <n-tag v-if="parsedSummary.errorsTotal != null && parsedSummary.errorsTotal > 0" size="small" type="error" :bordered="false">
            {{ t('runs.detail.errors', { count: parsedSummary.errorsTotal }) }}
          </n-tag>
          <n-tag v-if="parsedSummary.warningsTotal != null && parsedSummary.warningsTotal > 0" size="small" type="warning" :bordered="false">
            {{ t('runs.detail.warnings', { count: parsedSummary.warningsTotal }) }}
          </n-tag>
          <n-tag v-if="parsedSummary.entriesCount != null" size="small" :bordered="false">
            {{ t('runs.detail.entries', { count: parsedSummary.entriesCount }) }}
          </n-tag>
          <n-tag v-if="parsedSummary.partsCount != null" size="small" :bordered="false">
            {{ t('runs.detail.parts', { count: parsedSummary.partsCount }) }}
          </n-tag>
        </div>

        <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
          <dt class="app-text-muted">{{ t('runs.columns.startedAt') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatUnixSeconds(run.started_at) }}</dd>

          <dt class="app-text-muted">{{ t('runs.columns.endedAt') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatUnixSeconds(run.ended_at) }}</dd>

          <dt class="app-text-muted">{{ t('runs.detail.duration') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatDuration(durationSeconds) }}</dd>

          <dt class="app-text-muted">{{ t('runs.detail.target') }}</dt>
          <dd class="min-w-0">
            <div class="flex items-start gap-2 min-w-0">
              <n-tag size="small" :bordered="false" class="shrink-0">{{ targetTypeLabel }}</n-tag>
              <span class="flex-1 min-w-0 font-mono tabular-nums break-all whitespace-normal">
                {{ parsedSummary.targetLocation ?? '-' }}
              </span>
            </div>
          </dd>
        </dl>
      </div>

      <div class="border-t border-[color:var(--app-border)]" />

      <div class="space-y-1.5" data-testid="run-detail-progress">
        <div class="flex items-center justify-between gap-3">
          <div class="text-sm font-medium">{{ t('runs.progress.title') }}</div>
        </div>
        <run-progress-panel
          :progress="run.progress"
          :events="events"
          :run-started-at="run.started_at"
          :run-ended-at="run.ended_at"
          :run-status="run.status"
        />
      </div>
    </div>
  </n-card>
</template>
