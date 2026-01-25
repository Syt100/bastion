<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NButton, NIcon, NPopover, NProgress, NStep, NSteps, NTag } from 'naive-ui'
import { HelpCircleOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatBytes } from '@/lib/format'
import { useMediaQuery } from '@/lib/media'
import { useUiStore } from '@/stores/ui'
import type { RunEvent, RunStatus } from '@/stores/jobs'

type ProgressUnits = { files: number; dirs: number; bytes: number }

type BackupDetail = {
  source_total?: ProgressUnits | null
  transfer_total_bytes?: number | null
  transfer_done_bytes?: number | null
}

type ProgressDetail = {
  backup?: BackupDetail | null
}

type ProgressSnapshot = {
  stage: string
  ts: number
  done: ProgressUnits
  total?: ProgressUnits | null
  rate_bps?: number | null
  eta_seconds?: number | null
  detail?: ProgressDetail | null
}

const props = defineProps<{
  progress: unknown | null | undefined
  events?: RunEvent[] | null | undefined
  runStartedAt?: number | null | undefined
  runEndedAt?: number | null | undefined
  runStatus?: RunStatus | null | undefined
}>()

const { t } = useI18n()
const ui = useUiStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

function asUnits(value: unknown): ProgressUnits | null {
  if (!value || typeof value !== 'object') return null
  const v = value as Partial<ProgressUnits>
  const files = typeof v.files === 'number' && Number.isFinite(v.files) ? v.files : null
  const dirs = typeof v.dirs === 'number' && Number.isFinite(v.dirs) ? v.dirs : null
  const bytes = typeof v.bytes === 'number' && Number.isFinite(v.bytes) ? v.bytes : null
  if (files == null || dirs == null || bytes == null) return null
  return { files, dirs, bytes }
}

const snapshot = computed<ProgressSnapshot | null>(() => {
  const p = props.progress
  if (!p || typeof p !== 'object') return null
  const obj = p as Partial<ProgressSnapshot>
  if (typeof obj.stage !== 'string') return null
  if (typeof obj.ts !== 'number' || !Number.isFinite(obj.ts)) return null
  const done = asUnits(obj.done)
  if (!done) return null

  const total = obj.total == null ? null : asUnits(obj.total)
  const detail = obj.detail && typeof obj.detail === 'object' ? (obj.detail as ProgressDetail) : null

  return {
    stage: obj.stage,
    ts: obj.ts,
    done,
    total,
    rate_bps: typeof obj.rate_bps === 'number' ? obj.rate_bps : null,
    eta_seconds: typeof obj.eta_seconds === 'number' ? obj.eta_seconds : null,
    detail,
  }
})

const stage = computed(() => snapshot.value?.stage ?? null)
const backupDetail = computed<BackupDetail | null>(() => snapshot.value?.detail?.backup ?? null)
const showStages = computed(() => {
  const s = stage.value
  return s === 'scan' || s === 'packaging' || s === 'upload' || s === 'complete'
})

type KnownStage = 'scan' | 'packaging' | 'upload' | 'complete'
const knownStage = computed<KnownStage | null>(() => {
  const s = stage.value
  if (s === 'scan' || s === 'packaging' || s === 'upload' || s === 'complete') return s
  return null
})

const sourceTotal = computed<ProgressUnits | null>(() => {
  const fromDetail = asUnits(backupDetail.value?.source_total ?? null)
  if (fromDetail) return fromDetail

  // Back-compat: when not uploading, total usually refers to the filesystem SOURCE totals.
  if (stage.value && stage.value !== 'upload') {
    return snapshot.value?.total ?? null
  }
  return null
})

const transferTotalBytes = computed<number | null>(() => {
  const v = backupDetail.value?.transfer_total_bytes
  if (typeof v === 'number' && Number.isFinite(v) && v >= 0) return v

  if (stage.value === 'upload') {
    const total = snapshot.value?.total
    if (total && typeof total.bytes === 'number' && Number.isFinite(total.bytes)) return total.bytes
  }
  return null
})

const transferDoneBytes = computed<number | null>(() => {
  const v = backupDetail.value?.transfer_done_bytes
  if (typeof v === 'number' && Number.isFinite(v) && v >= 0) return v

  if (stage.value === 'upload') {
    const done = snapshot.value?.done
    if (done && typeof done.bytes === 'number' && Number.isFinite(done.bytes)) return done.bytes
  }
  return null
})

function percent(done: number | null, total: number | null): number | null {
  if (done == null || total == null) return null
  if (!Number.isFinite(done) || !Number.isFinite(total) || total <= 0) return null
  return Math.max(0, Math.min(100, (done / total) * 100))
}

const scanPct = computed<number | null>(() => {
  if (!snapshot.value) return null
  if (knownStage.value !== 'scan') return null
  const total = snapshot.value.total?.bytes ?? null
  return percent(snapshot.value.done.bytes, total)
})

const packagingPct = computed<number | null>(() => {
  if (!snapshot.value) return null
  const totalBytes = snapshot.value.total?.bytes ?? null
  const doneBytes = snapshot.value.done.bytes
  if (stage.value === 'packaging') return percent(doneBytes, totalBytes)
  if (stage.value === 'upload') return totalBytes != null ? 100 : null
  if (stage.value === 'complete') return 100
  return null
})

const uploadPct = computed<number | null>(() => {
  if (stage.value === 'upload') return percent(transferDoneBytes.value, transferTotalBytes.value)
  if (stage.value === 'complete') return 100
  return null
})

const displayStage = computed<KnownStage | null>(() => {
  const s = knownStage.value
  if (!s) return null
  if (props.runStatus === 'success' && props.runEndedAt != null) return 'complete'
  if (s === 'upload' && uploadPct.value != null && uploadPct.value >= 100) return 'complete'
  return s
})

const stageForLabel = computed<string | null>(() => displayStage.value ?? stage.value)

const stepsCurrent = computed<number>(() => {
  const s = displayStage.value
  if (s === 'scan') return 1
  if (s === 'packaging') return 2
  return 3
})

const stepsStatus = computed<'process' | 'finish' | 'error' | 'wait'>(() => {
  const s = displayStage.value
  if (!s) return 'wait'
  if (props.runStatus === 'failed' || props.runStatus === 'rejected') return 'error'
  if (s === 'complete') return 'finish'
  return 'process'
})

const currentStagePct = computed<number | null>(() => {
  const s = displayStage.value
  if (s === 'scan') return scanPct.value
  if (s === 'packaging') return packagingPct.value
  if (s === 'upload') return uploadPct.value
  if (s === 'complete') return 100
  return null
})

const overallPct = computed<number | null>(() => {
  const s = stageForLabel.value
  if (!s) return null

  const wScan = 0.05
  const wPackaging = 0.45
  const wUpload = 0.5

  const stagePct = ((): number | null => {
    if (s === 'scan') {
      const total = snapshot.value?.total?.bytes ?? null
      return percent(snapshot.value?.done.bytes ?? null, total)
    }
    if (s === 'packaging') return packagingPct.value
    if (s === 'upload') return uploadPct.value
    if (s === 'complete') return 100
    return null
  })()

  if (stagePct == null) return null
  const r = stagePct / 100

  if (s === 'scan') return (wScan * r) * 100
  if (s === 'packaging') return (wScan + wPackaging * r) * 100
  if (s === 'upload') return (wScan + wPackaging + wUpload * r) * 100
  if (s === 'complete') return 100
  return null
})

function stageLabel(value: string | null): string {
  if (!value) return '-'
  if (value === 'scan') return t('runs.progress.stages.scan')
  if (value === 'packaging') return t('runs.progress.stages.packaging')
  if (value === 'upload') return t('runs.progress.stages.upload')
  if (value === 'complete') return t('runs.progress.stages.complete')
  return value
}

function stageHelp(stage: 'scan' | 'packaging' | 'upload'): { title: string; body: string } {
  if (stage === 'scan') {
    return { title: t('runs.progress.help.scanTitle'), body: t('runs.progress.help.scanBody') }
  }
  if (stage === 'packaging') {
    return {
      title: t('runs.progress.help.packagingTitle'),
      body: t('runs.progress.help.packagingBody'),
    }
  }
  return { title: t('runs.progress.help.uploadTitle'), body: t('runs.progress.help.uploadBody') }
}

const currentStageHelp = computed<{ title: string; body: string } | null>(() => {
  const s = displayStage.value
  if (s === 'scan' || s === 'packaging' || s === 'upload') return stageHelp(s)
  return null
})

const stagesHelp = computed(() => ({
  scan: stageHelp('scan'),
  packaging: stageHelp('packaging'),
  upload: stageHelp('upload'),
}))

type StageBoundaryTs = { scan: number | null; packaging: number | null; upload: number | null; end: number | null }

const stageBoundaryTs = computed<StageBoundaryTs>(() => {
  const out: StageBoundaryTs = { scan: null, packaging: null, upload: null, end: null }
  for (const e of props.events ?? []) {
    if (out.scan == null && e.kind === 'scan') out.scan = e.ts
    if (out.packaging == null && e.kind === 'packaging') out.packaging = e.ts
    if (out.upload == null && e.kind === 'upload') out.upload = e.ts
    if (out.end == null && (e.kind === 'complete' || e.kind === 'failed')) out.end = e.ts
  }
  return out
})

const runStartTs = computed<number | null>(() => {
  const v = props.runStartedAt
  return typeof v === 'number' && Number.isFinite(v) ? v : null
})

const runEndTs = computed<number | null>(() => {
  const fromEvent = stageBoundaryTs.value.end
  if (fromEvent != null) return fromEvent
  const v = props.runEndedAt
  return typeof v === 'number' && Number.isFinite(v) ? v : null
})

// Prefer a deterministic "now" aligned with snapshot updates for display/testing.
const nowTs = computed<number | null>(() => snapshot.value?.ts ?? runEndTs.value ?? null)

function durationSeconds(start: number | null, end: number | null): number | null {
  if (start == null || end == null) return null
  if (!Number.isFinite(start) || !Number.isFinite(end)) return null
  if (end < start) return null
  return end - start
}

const scanDurationSeconds = computed<number | null>(() => {
  const start = stageBoundaryTs.value.scan ?? runStartTs.value
  const end =
    stageBoundaryTs.value.packaging ??
    (displayStage.value === 'scan' ? nowTs.value : null)
  return durationSeconds(start, end)
})

const packagingDurationSeconds = computed<number | null>(() => {
  const start = stageBoundaryTs.value.packaging
  const end =
    stageBoundaryTs.value.upload ??
    (displayStage.value === 'packaging' ? nowTs.value : null)
  return durationSeconds(start, end)
})

const uploadDurationSeconds = computed<number | null>(() => {
  const start = stageBoundaryTs.value.upload
  const end =
    runEndTs.value ??
    (displayStage.value === 'upload' ? nowTs.value : null)
  return durationSeconds(start, end)
})

const totalDurationSeconds = computed<number | null>(() => {
  const start = runStartTs.value
  const end = runEndTs.value ?? (props.runStatus === 'running' ? nowTs.value : null)
  return durationSeconds(start, end)
})

const finalAvgRateBps = computed<number | null>(() => {
  const totalBytes = transferTotalBytes.value
  if (totalBytes == null || !Number.isFinite(totalBytes) || totalBytes <= 0) return null

  const end = runEndTs.value
  if (end == null) return null

  const uploadStart = stageBoundaryTs.value.upload
  if (uploadStart != null && end > uploadStart) {
    return Math.floor(totalBytes / (end - uploadStart))
  }

  const start = runStartTs.value
  if (start != null && end > start) {
    return Math.floor(totalBytes / (end - start))
  }

  return null
})

const peakRateBps = ref<number | null>(null)
watch(
  () => snapshot.value?.rate_bps ?? null,
  (v) => {
    if (typeof v !== 'number' || !Number.isFinite(v) || v <= 0) return
    if (peakRateBps.value == null || v > peakRateBps.value) peakRateBps.value = v
  },
  { immediate: true },
)

const displayRateBps = computed<number | null>(() => {
  const live = snapshot.value?.rate_bps
  if (typeof live === 'number' && Number.isFinite(live) && live > 0) return live

  const finalRate = finalAvgRateBps.value
  const isTerminal =
    props.runStatus === 'success' || props.runStatus === 'failed' || props.runStatus === 'rejected'
  if ((isTerminal || displayStage.value === 'complete') && typeof finalRate === 'number' && Number.isFinite(finalRate) && finalRate > 0) {
    return finalRate
  }

  return null
})

const displayPeakRateBps = computed<number | null>(() => {
  const peak = peakRateBps.value
  if (peak == null || peak <= 0) return null
  const base = displayRateBps.value ?? 0
  if (peak <= base) return null
  return peak
})

const failureStage = computed<KnownStage | null>(() => {
  if (props.runStatus !== 'failed' && props.runStatus !== 'rejected') return null

  const end = runEndTs.value ?? nowTs.value ?? Number.POSITIVE_INFINITY
  if (stageBoundaryTs.value.upload != null && stageBoundaryTs.value.upload <= end) return 'upload'
  if (stageBoundaryTs.value.packaging != null && stageBoundaryTs.value.packaging <= end) return 'packaging'
  if (stageBoundaryTs.value.scan != null && stageBoundaryTs.value.scan <= end) return 'scan'

  const s = knownStage.value
  if (s === 'scan' || s === 'packaging' || s === 'upload') return s
  return null
})

function formatEta(seconds: number | null | undefined): string {
  if (seconds == null || !Number.isFinite(seconds) || seconds < 0) return '-'
  const s = Math.floor(seconds)
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = s % 60
  if (h > 0) return `${h}h ${m}m`
  if (m > 0) return `${m}m ${sec}s`
  return `${sec}s`
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

function progressNumber(pct: number | null): number {
  return pct == null ? 0 : Math.round(pct)
}
</script>

<template>
  <div v-if="!snapshot" class="text-sm opacity-70">-</div>
  <div v-else class="space-y-1.5">
    <div class="flex items-start justify-between gap-3">
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <n-tag size="small" :bordered="false">{{ stageLabel(stageForLabel) }}</n-tag>
          <n-tag v-if="failureStage" size="small" type="error" :bordered="false">
            {{ t('runs.progress.failureStage') }}: {{ stageLabel(failureStage) }}
          </n-tag>
          <div class="text-xs opacity-70 truncate">
            {{ t('runs.progress.updatedAt') }}: {{ formatUnixSeconds(snapshot.ts) }}
          </div>
        </div>
      </div>
    </div>

    <div class="space-y-1">
      <div class="flex items-center justify-between gap-3 text-xs opacity-70">
        <div class="truncate">{{ t('runs.progress.overall') }}</div>
        <div v-if="overallPct != null" class="shrink-0 font-mono tabular-nums whitespace-nowrap">
          {{ Math.round(overallPct) }}%
        </div>
        <div v-else>-</div>
      </div>
      <n-progress
        type="line"
        :percentage="progressNumber(overallPct)"
        :processing="overallPct == null"
        :show-indicator="false"
      />
    </div>

    <div v-if="showStages" class="space-y-1.5">
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-1.5 font-medium">
          <span>{{ t('runs.progress.stages.title') }}</span>
          <n-popover trigger="click" placement="top-start" :show-arrow="false">
            <template #trigger>
              <n-button size="tiny" circle quaternary :aria-label="t('common.help')">
                <template #icon>
                  <n-icon :component="HelpCircleOutline" :size="14" />
                </template>
              </n-button>
            </template>
            <div class="max-w-[420px] space-y-3 whitespace-pre-wrap break-words text-sm">
              <div>
                <div class="font-medium mb-1">{{ stagesHelp.scan.title }}</div>
                <div>{{ stagesHelp.scan.body }}</div>
              </div>
              <div>
                <div class="font-medium mb-1">{{ stagesHelp.packaging.title }}</div>
                <div>{{ stagesHelp.packaging.body }}</div>
              </div>
              <div>
                <div class="font-medium mb-1">{{ stagesHelp.upload.title }}</div>
                <div>{{ stagesHelp.upload.body }}</div>
              </div>
            </div>
          </n-popover>
        </div>
      </div>

      <div :class="isDesktop ? 'max-w-[520px]' : ''">
        <n-steps
          :current="stepsCurrent"
          :status="stepsStatus"
          size="small"
          :vertical="!isDesktop"
          :content-placement="isDesktop ? 'bottom' : 'right'"
        >
          <n-step :title="t('runs.progress.stages.scan')" />
          <n-step :title="t('runs.progress.stages.packaging')" />
          <n-step :title="t('runs.progress.stages.upload')" />
        </n-steps>
      </div>

      <div v-if="currentStageHelp" class="space-y-1">
        <div class="flex items-center justify-between gap-3">
          <div class="flex items-center gap-1.5 min-w-0">
            <div class="text-sm font-medium truncate">{{ stageLabel(stageForLabel) }}</div>
            <n-popover trigger="click" placement="top-start" :show-arrow="false">
              <template #trigger>
                <n-button size="tiny" circle quaternary>
                  <template #icon>
                    <n-icon :component="HelpCircleOutline" :size="14" />
                  </template>
                </n-button>
              </template>
              <div class="max-w-[420px] whitespace-pre-wrap break-words text-sm">
                <div class="font-medium mb-1">{{ currentStageHelp.title }}</div>
                <div>{{ currentStageHelp.body }}</div>
              </div>
            </n-popover>
          </div>
          <div class="text-xs opacity-70">
            <span v-if="currentStagePct != null">{{ Math.round(currentStagePct) }}%</span>
            <span v-else>-</span>
          </div>
        </div>
        <n-progress
          type="line"
          :percentage="progressNumber(currentStagePct)"
          :processing="currentStagePct == null"
          :show-indicator="false"
        />
      </div>
    </div>
    <div v-else class="text-sm opacity-70">{{ stageLabel(stageForLabel) }}</div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
      <div class="rounded border border-black/5 dark:border-white/10 p-2.5 md:col-span-2">
        <div class="text-sm font-medium mb-1">{{ t('runs.progress.timeline.title') }}</div>
        <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
          <dt class="opacity-70">{{ t('runs.progress.stages.scan') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatDuration(scanDurationSeconds) }}</dd>
          <dt class="opacity-70">{{ t('runs.progress.stages.packaging') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatDuration(packagingDurationSeconds) }}</dd>
          <dt class="opacity-70">{{ t('runs.progress.stages.upload') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatDuration(uploadDurationSeconds) }}</dd>
          <dt class="opacity-70">{{ t('runs.progress.timeline.total') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatDuration(totalDurationSeconds) }}</dd>
        </dl>
      </div>

      <div class="rounded border border-black/5 dark:border-white/10 p-2.5">
        <div class="text-sm font-medium mb-1">{{ t('runs.progress.source.title') }}</div>
        <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
          <dt class="opacity-70">{{ t('runs.progress.source.files') }}</dt>
          <dd class="font-mono tabular-nums">{{ sourceTotal?.files ?? '-' }}</dd>
          <dt class="opacity-70">{{ t('runs.progress.source.dirs') }}</dt>
          <dd class="font-mono tabular-nums">{{ sourceTotal?.dirs ?? '-' }}</dd>
          <dt class="opacity-70">{{ t('runs.progress.source.bytes') }}</dt>
          <dd class="font-mono tabular-nums">{{ sourceTotal ? formatBytes(sourceTotal.bytes) : '-' }}</dd>
        </dl>
      </div>

      <div class="rounded border border-black/5 dark:border-white/10 p-2.5">
        <div class="text-sm font-medium mb-1">{{ t('runs.progress.transfer.title') }}</div>
        <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
          <dt class="opacity-70">{{ t('runs.progress.transfer.done') }}</dt>
          <dd class="font-mono tabular-nums">{{ transferDoneBytes != null ? formatBytes(transferDoneBytes) : '-' }}</dd>
          <dt class="opacity-70">{{ t('runs.progress.transfer.total') }}</dt>
          <dd class="font-mono tabular-nums">{{ transferTotalBytes != null ? formatBytes(transferTotalBytes) : '-' }}</dd>
          <dt class="opacity-70">{{ t('runs.progress.transfer.rate') }}</dt>
          <dd class="font-mono tabular-nums">{{ displayRateBps != null ? `${formatBytes(displayRateBps)}/s` : '-' }}</dd>
          <template v-if="displayPeakRateBps != null">
            <dt class="opacity-70">{{ t('runs.progress.transfer.peak') }}</dt>
            <dd class="font-mono tabular-nums">{{ `${formatBytes(displayPeakRateBps)}/s` }}</dd>
          </template>
          <dt class="opacity-70">{{ t('runs.progress.transfer.eta') }}</dt>
          <dd class="font-mono tabular-nums">{{ formatEta(snapshot.eta_seconds) }}</dd>
        </dl>
      </div>
    </div>
  </div>
</template>
