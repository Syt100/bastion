<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NCard, NPopover, NProgress, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatBytes } from '@/lib/format'
import { useMediaQuery } from '@/lib/media'
import { useUiStore } from '@/stores/ui'

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

const overallPct = computed<number | null>(() => {
  const s = stage.value
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

function progressNumber(pct: number | null): number {
  return pct == null ? 0 : Math.round(pct)
}
</script>

<template>
  <n-card class="app-card" :bordered="false" :title="t('runs.progress.title')" size="small">
    <div v-if="!snapshot" class="text-sm opacity-70">-</div>
    <div v-else class="space-y-4">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <n-tag size="small" :bordered="false">{{ stageLabel(stage) }}</n-tag>
            <div class="text-xs opacity-70 truncate">
              {{ t('runs.progress.updatedAt') }}: {{ formatUnixSeconds(snapshot.ts) }}
            </div>
          </div>
        </div>
        <div v-if="overallPct != null" class="text-xs opacity-70">{{ Math.round(overallPct) }}%</div>
      </div>

      <div class="space-y-1">
        <div class="flex items-center justify-between gap-3 text-xs opacity-70">
          <div class="truncate">{{ t('runs.progress.overall') }}</div>
          <div v-if="overallPct != null">{{ Math.round(overallPct) }}%</div>
          <div v-else>-</div>
        </div>
        <n-progress
          type="line"
          :percentage="progressNumber(overallPct)"
          :processing="overallPct == null"
          :show-indicator="overallPct != null"
        />
      </div>

      <div v-if="isDesktop && showStages" class="space-y-2">
        <div class="flex items-center gap-2">
          <div class="font-medium">{{ t('runs.progress.stages.title') }}</div>
        </div>

        <div class="space-y-3">
          <div class="space-y-1">
            <div class="flex items-center justify-between gap-3">
              <div class="flex items-center gap-2 min-w-0">
                <div class="text-sm truncate">{{ t('runs.progress.stages.scan') }}</div>
                <n-popover trigger="click" placement="top-start" :show-arrow="false">
                  <template #trigger>
                    <n-button size="small" circle>?</n-button>
                  </template>
                  <div class="max-w-[420px] whitespace-pre-wrap break-words text-sm">
                    <div class="font-medium mb-1">{{ stageHelp('scan').title }}</div>
                    <div>{{ stageHelp('scan').body }}</div>
                  </div>
                </n-popover>
              </div>
              <div class="text-xs opacity-70">{{ stage === 'scan' ? '-' : t('common.done') }}</div>
            </div>
            <n-progress type="line" :percentage="stage === 'scan' ? 0 : 100" :processing="stage === 'scan'" :show-indicator="stage !== 'scan'" />
          </div>

          <div class="space-y-1">
            <div class="flex items-center justify-between gap-3">
              <div class="flex items-center gap-2 min-w-0">
                <div class="text-sm truncate">{{ t('runs.progress.stages.packaging') }}</div>
                <n-popover trigger="click" placement="top-start" :show-arrow="false">
                  <template #trigger>
                    <n-button size="small" circle>?</n-button>
                  </template>
                  <div class="max-w-[420px] whitespace-pre-wrap break-words text-sm">
                    <div class="font-medium mb-1">{{ stageHelp('packaging').title }}</div>
                    <div>{{ stageHelp('packaging').body }}</div>
                  </div>
                </n-popover>
              </div>
              <div class="text-xs opacity-70">
                <span v-if="packagingPct != null">{{ Math.round(packagingPct) }}%</span>
                <span v-else>-</span>
              </div>
            </div>
            <n-progress
              type="line"
              :percentage="progressNumber(packagingPct)"
              :processing="stage === 'packaging' && packagingPct == null"
              :show-indicator="packagingPct != null"
            />
          </div>

          <div class="space-y-1">
            <div class="flex items-center justify-between gap-3">
              <div class="flex items-center gap-2 min-w-0">
                <div class="text-sm truncate">{{ t('runs.progress.stages.upload') }}</div>
                <n-popover trigger="click" placement="top-start" :show-arrow="false">
                  <template #trigger>
                    <n-button size="small" circle>?</n-button>
                  </template>
                  <div class="max-w-[420px] whitespace-pre-wrap break-words text-sm">
                    <div class="font-medium mb-1">{{ stageHelp('upload').title }}</div>
                    <div>{{ stageHelp('upload').body }}</div>
                  </div>
                </n-popover>
              </div>
              <div class="text-xs opacity-70">
                <span v-if="uploadPct != null">{{ Math.round(uploadPct) }}%</span>
                <span v-else>-</span>
              </div>
            </div>
            <n-progress
              type="line"
              :percentage="progressNumber(uploadPct)"
              :processing="stage === 'upload' && uploadPct == null"
              :show-indicator="uploadPct != null"
            />
          </div>
        </div>
      </div>

      <details v-else-if="showStages" class="rounded border border-black/5 dark:border-white/10 p-3">
        <summary class="cursor-pointer select-none text-sm font-medium">
          {{ t('runs.progress.details') }}
        </summary>
        <div class="mt-3 space-y-3">
          <div class="space-y-3">
            <div class="space-y-1">
              <div class="flex items-center justify-between gap-3">
                <div class="flex items-center gap-2 min-w-0">
                  <div class="text-sm truncate">{{ t('runs.progress.stages.scan') }}</div>
                  <n-popover trigger="click" placement="top-start" :show-arrow="false">
                    <template #trigger>
                      <n-button size="small" circle>?</n-button>
                    </template>
                    <div class="max-w-[420px] whitespace-pre-wrap break-words text-sm">
                      <div class="font-medium mb-1">{{ stageHelp('scan').title }}</div>
                      <div>{{ stageHelp('scan').body }}</div>
                    </div>
                  </n-popover>
                </div>
                <div class="text-xs opacity-70">{{ stage === 'scan' ? '-' : t('common.done') }}</div>
              </div>
              <n-progress
                type="line"
                :percentage="stage === 'scan' ? 0 : 100"
                :processing="stage === 'scan'"
                :show-indicator="stage !== 'scan'"
              />
            </div>

            <div class="space-y-1">
              <div class="flex items-center justify-between gap-3">
                <div class="flex items-center gap-2 min-w-0">
                  <div class="text-sm truncate">{{ t('runs.progress.stages.packaging') }}</div>
                  <n-popover trigger="click" placement="top-start" :show-arrow="false">
                    <template #trigger>
                      <n-button size="small" circle>?</n-button>
                    </template>
                    <div class="max-w-[420px] whitespace-pre-wrap break-words text-sm">
                      <div class="font-medium mb-1">{{ stageHelp('packaging').title }}</div>
                      <div>{{ stageHelp('packaging').body }}</div>
                    </div>
                  </n-popover>
                </div>
                <div class="text-xs opacity-70">
                  <span v-if="packagingPct != null">{{ Math.round(packagingPct) }}%</span>
                  <span v-else>-</span>
                </div>
              </div>
              <n-progress
                type="line"
                :percentage="progressNumber(packagingPct)"
                :processing="stage === 'packaging' && packagingPct == null"
                :show-indicator="packagingPct != null"
              />
            </div>

            <div class="space-y-1">
              <div class="flex items-center justify-between gap-3">
                <div class="flex items-center gap-2 min-w-0">
                  <div class="text-sm truncate">{{ t('runs.progress.stages.upload') }}</div>
                  <n-popover trigger="click" placement="top-start" :show-arrow="false">
                    <template #trigger>
                      <n-button size="small" circle>?</n-button>
                    </template>
                    <div class="max-w-[420px] whitespace-pre-wrap break-words text-sm">
                      <div class="font-medium mb-1">{{ stageHelp('upload').title }}</div>
                      <div>{{ stageHelp('upload').body }}</div>
                    </div>
                  </n-popover>
                </div>
                <div class="text-xs opacity-70">
                  <span v-if="uploadPct != null">{{ Math.round(uploadPct) }}%</span>
                  <span v-else>-</span>
                </div>
              </div>
              <n-progress
                type="line"
                :percentage="progressNumber(uploadPct)"
                :processing="stage === 'upload' && uploadPct == null"
                :show-indicator="uploadPct != null"
              />
            </div>
          </div>
        </div>
      </details>
      <div v-else class="text-sm opacity-70">{{ stageLabel(stage) }}</div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div class="rounded border border-black/5 dark:border-white/10 p-3">
          <div class="text-sm font-medium mb-2">{{ t('runs.progress.source.title') }}</div>
          <div class="text-xs opacity-70 space-y-1">
            <div>{{ t('runs.progress.source.files') }}: {{ sourceTotal?.files ?? '-' }}</div>
            <div>{{ t('runs.progress.source.dirs') }}: {{ sourceTotal?.dirs ?? '-' }}</div>
            <div>{{ t('runs.progress.source.bytes') }}: {{ sourceTotal ? formatBytes(sourceTotal.bytes) : '-' }}</div>
          </div>
        </div>

        <div class="rounded border border-black/5 dark:border-white/10 p-3">
          <div class="text-sm font-medium mb-2">{{ t('runs.progress.transfer.title') }}</div>
          <div class="text-xs opacity-70 space-y-1">
            <div>
              {{ t('runs.progress.transfer.done') }}:
              {{ transferDoneBytes != null ? formatBytes(transferDoneBytes) : '-' }}
            </div>
            <div>
              {{ t('runs.progress.transfer.total') }}:
              {{ transferTotalBytes != null ? formatBytes(transferTotalBytes) : '-' }}
            </div>
            <div>
              {{ t('runs.progress.transfer.rate') }}:
              {{ snapshot.rate_bps && snapshot.rate_bps > 0 ? `${formatBytes(snapshot.rate_bps)}/s` : '-' }}
            </div>
            <div>
              {{ t('runs.progress.transfer.eta') }}:
              {{ formatEta(snapshot.eta_seconds) }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </n-card>
</template>
