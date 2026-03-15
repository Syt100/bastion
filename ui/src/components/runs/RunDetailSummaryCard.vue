<script setup lang="ts">
import { computed } from 'vue'
import { NAlert, NCard, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import type { RunEvent, RunWorkspaceDetail } from '@/stores/runs'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { runFailureStageLabel, runTargetTypeLabel } from '@/lib/runs'
import { parseRunSummary } from '@/lib/run_summary'
import RunProgressPanel from '@/components/runs/RunProgressPanel.vue'

const props = defineProps<{
  detail: RunWorkspaceDetail | null
  events: RunEvent[]
}>()

const { t } = useI18n()
const ui = useUiStore()
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const parsedSummary = computed(() => parseRunSummary(props.detail?.summary))
const targetTypeLabel = computed(() => runTargetTypeLabel(t, parsedSummary.value.targetType))
const failureStageLabel = computed(() => runFailureStageLabel(t, props.detail?.diagnostics.failure_stage))
const primaryFailureBody = computed(() => {
  const runError = props.detail?.run.error
  if (shouldShowRunError(runError)) return runError
  if (props.detail?.diagnostics.failure_hint) return null
  return t('runs.detail.noFailureHint')
})

function shouldShowRunError(value: string | null | undefined): boolean {
  if (!value) return false
  return !/^[a-z0-9_]+$/i.test(value)
}

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
  const run = props.detail?.run
  if (!run?.started_at || !run.ended_at) return null
  return Math.max(0, run.ended_at - run.started_at)
})
</script>

<template>
  <n-card
    :title="t('runs.detail.overviewTitle')"
    size="small"
    class="app-card"
    :bordered="false"
    data-testid="run-detail-summary"
  >
    <div v-if="!detail" class="text-sm app-text-muted">-</div>
    <div v-else class="space-y-3">
      <n-alert
        v-if="detail.run.status === 'failed' || detail.run.status === 'rejected' || detail.run.status === 'canceled'"
        type="error"
        :title="detail.diagnostics.failure_title"
        :bordered="false"
      >
        <div v-if="primaryFailureBody">{{ primaryFailureBody }}</div>
        <div v-if="detail.diagnostics.failure_hint" class="mt-1">
          {{ detail.diagnostics.failure_hint }}
        </div>
      </n-alert>

      <div class="space-y-2" data-testid="run-detail-overview">
        <div class="flex flex-wrap items-center gap-2">
          <n-tag v-if="detail.diagnostics.failure_kind" size="small" type="error" :bordered="false">
            {{ t('runs.detail.failureKind') }}: {{ detail.diagnostics.failure_kind }}
          </n-tag>
          <n-tag v-if="detail.diagnostics.failure_stage" size="small" type="warning" :bordered="false">
            {{ t('runs.detail.failureStage') }}: {{ failureStageLabel }}
          </n-tag>
          <n-tag v-if="parsedSummary.errorsTotal != null && parsedSummary.errorsTotal > 0" size="small" type="error" :bordered="false">
            {{ t('runs.detail.errors', { count: parsedSummary.errorsTotal }) }}
          </n-tag>
          <n-tag v-if="parsedSummary.warningsTotal != null && parsedSummary.warningsTotal > 0" size="small" type="warning" :bordered="false">
            {{ t('runs.detail.warnings', { count: parsedSummary.warningsTotal }) }}
          </n-tag>
          <n-tag
            v-if="detail.diagnostics.first_error_event_seq != null"
            size="small"
            :bordered="false"
          >
            {{ t('runs.detail.firstErrorSeq', { seq: detail.diagnostics.first_error_event_seq }) }}
          </n-tag>
        </div>

        <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
          <dt class="app-text-muted">{{ t('runs.columns.startedAt') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatUnixSeconds(detail.run.started_at) }}</dd>

          <dt class="app-text-muted">{{ t('runs.columns.endedAt') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatUnixSeconds(detail.run.ended_at) }}</dd>

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

          <dt class="app-text-muted">{{ t('runs.detail.relatedOps') }}</dt>
          <dd>{{ detail.related.operations_total }}</dd>

          <dt class="app-text-muted">{{ t('runs.detail.relatedArtifacts') }}</dt>
          <dd>{{ detail.related.artifacts_total }}</dd>
        </dl>
      </div>

      <div class="border-t border-[color:var(--app-border)]" />

      <div class="space-y-1.5" data-testid="run-detail-progress">
        <div class="text-sm font-medium">{{ t('runs.progress.title') }}</div>
        <RunProgressPanel
          :progress="detail.progress"
          :events="events"
          :run-started-at="detail.run.started_at"
          :run-ended-at="detail.run.ended_at"
          :run-status="detail.run.status"
        />
      </div>
    </div>
  </n-card>
</template>
