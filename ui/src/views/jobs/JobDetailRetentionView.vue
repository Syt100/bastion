<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import {
  NAlert,
  NButton,
  NCard,
  NForm,
  NFormItem,
  NInputNumber,
  NSwitch,
  NSpin,
  useMessage,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type RetentionPolicy, type RetentionPreviewResponse } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { useJobDetailContext } from '@/lib/jobDetailContext'

const { t } = useI18n()
const message = useMessage()

const ctx = useJobDetailContext()
const jobs = useJobsStore()
const ui = useUiStore()

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const loading = ref<boolean>(false)
const saving = ref<boolean>(false)
const previewLoading = ref<boolean>(false)
const applying = ref<boolean>(false)
const error = ref<string | null>(null)
const preview = ref<RetentionPreviewResponse | null>(null)

const form = reactive<{
  enabled: boolean
  keepLast: number | null
  keepDays: number | null
  maxDeletePerTick: number
  maxDeletePerDay: number
}>({
  enabled: false,
  keepLast: null,
  keepDays: null,
  maxDeletePerTick: 50,
  maxDeletePerDay: 200,
})

function normalizeOptionalPositiveInt(value: number | null): number | null {
  if (typeof value !== 'number') return null
  const n = Math.floor(value)
  return n > 0 ? n : null
}

function buildRetentionPayload(): RetentionPolicy {
  return {
    enabled: form.enabled,
    keep_last: normalizeOptionalPositiveInt(form.keepLast),
    keep_days: normalizeOptionalPositiveInt(form.keepDays),
    max_delete_per_tick: Math.max(1, Math.floor(form.maxDeletePerTick || 1)),
    max_delete_per_day: Math.max(1, Math.floor(form.maxDeletePerDay || 1)),
  }
}

async function refresh(): Promise<void> {
  const id = ctx.jobId.value
  if (!id) return

  loading.value = true
  error.value = null
  try {
    const r = await jobs.getJobRetention(id)
    form.enabled = !!r.enabled
    form.keepLast = typeof r.keep_last === 'number' ? r.keep_last : null
    form.keepDays = typeof r.keep_days === 'number' ? r.keep_days : null
    form.maxDeletePerTick = typeof r.max_delete_per_tick === 'number' && r.max_delete_per_tick > 0 ? r.max_delete_per_tick : 50
    form.maxDeletePerDay = typeof r.max_delete_per_day === 'number' && r.max_delete_per_day > 0 ? r.max_delete_per_day : 200
    preview.value = null
  } catch (e) {
    error.value = formatToastError(t('errors.fetchJobFailed'), e, t)
  } finally {
    loading.value = false
  }
}

function validateRetention(): boolean {
  const keepLast = normalizeOptionalPositiveInt(form.keepLast)
  const keepDays = normalizeOptionalPositiveInt(form.keepDays)

  if (form.enabled && keepLast === null && keepDays === null) {
    message.error(t('errors.retentionRuleRequired'))
    return false
  }
  if (form.maxDeletePerTick <= 0 || form.maxDeletePerDay <= 0) {
    message.error(t('errors.retentionLimitInvalid'))
    return false
  }
  return true
}

async function save(): Promise<void> {
  const id = ctx.jobId.value
  if (!id) return
  if (!validateRetention()) return

  saving.value = true
  try {
    await jobs.putJobRetention(id, buildRetentionPayload())
    message.success(t('messages.retentionSaved'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.saveJobFailed'), e, t))
  } finally {
    saving.value = false
  }
}

async function doPreview(): Promise<void> {
  const id = ctx.jobId.value
  if (!id) return
  if (!validateRetention()) return

  previewLoading.value = true
  try {
    preview.value = await jobs.previewJobRetention(id, buildRetentionPayload())
  } catch (e) {
    message.error(formatToastError(t('errors.previewRetentionFailed'), e, t))
  } finally {
    previewLoading.value = false
  }
}

async function applyNow(): Promise<void> {
  const id = ctx.jobId.value
  if (!id) return
  if (!validateRetention()) return
  if (!form.enabled) {
    message.error(t('jobs.retention.enableFirst'))
    return
  }

  applying.value = true
  try {
    const resp = await jobs.applyJobRetention(id, buildRetentionPayload())
    message.success(t('jobs.retention.applyOk', { n: resp.enqueued.length, existing: resp.already_exists, skipped: resp.skipped_due_to_limits }))
    await doPreview()
  } catch (e) {
    message.error(formatToastError(t('errors.applyRetentionFailed'), e, t))
  } finally {
    applying.value = false
  }
}

watch(
  () => ctx.jobId.value,
  (id) => {
    if (id) void refresh()
  },
  { immediate: true },
)
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-end gap-2">
      <n-button :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button type="primary" :loading="saving" @click="save">{{ t('common.save') }}</n-button>
    </div>

    <n-card class="app-card">
      <div class="space-y-3">
        <div class="flex items-center justify-between gap-3">
          <div class="text-sm font-medium">{{ t('jobs.retention.title') }}</div>
          <n-switch v-model:value="form.enabled" />
        </div>

        <div class="text-sm opacity-70">{{ t('jobs.retention.help') }}</div>

        <n-form label-placement="top">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
            <n-form-item :label="t('jobs.retention.keepLast')">
              <n-input-number v-model:value="form.keepLast" :min="0" class="w-full" />
            </n-form-item>
            <n-form-item :label="t('jobs.retention.keepDays')">
              <n-input-number v-model:value="form.keepDays" :min="0" class="w-full" />
            </n-form-item>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
            <n-form-item :label="t('jobs.retention.maxDeletePerTick')">
              <n-input-number v-model:value="form.maxDeletePerTick" :min="1" class="w-full" />
            </n-form-item>
            <n-form-item :label="t('jobs.retention.maxDeletePerDay')">
              <n-input-number v-model:value="form.maxDeletePerDay" :min="1" class="w-full" />
            </n-form-item>
          </div>
        </n-form>

        <div class="flex flex-wrap items-center gap-2">
          <n-button size="small" secondary :loading="previewLoading" @click="doPreview">{{ t('jobs.retention.preview') }}</n-button>
          <n-button
            size="small"
            type="primary"
            :loading="applying"
            :disabled="!form.enabled"
            @click="applyNow"
          >
            {{ t('jobs.retention.applyNow') }}
          </n-button>
          <div v-if="!form.enabled" class="text-xs opacity-70">{{ t('jobs.retention.enableFirst') }}</div>
        </div>

        <n-alert v-if="error" type="error" :bordered="false">
          {{ error }}
        </n-alert>

        <div v-if="previewLoading" class="flex justify-center py-2">
          <n-spin size="small" />
        </div>

        <div v-else-if="preview" class="space-y-2">
          <div class="text-sm">
            {{ t('jobs.retention.previewSummary', { keep: preview.keep_total, del: preview.delete_total }) }}
            <span v-if="preview.result_truncated" class="text-xs opacity-70 ml-2">
              {{ t('jobs.retention.previewTruncated') }}
            </span>
          </div>

          <div v-if="preview.delete_total === 0" class="text-xs opacity-70">
            {{ t('jobs.retention.previewNoDeletes') }}
          </div>
          <div v-else class="app-border-subtle rounded-md p-2">
            <div class="text-xs opacity-70 mb-1">{{ t('jobs.retention.deleteList') }}</div>
            <div class="space-y-1">
              <div
                v-for="it in preview.delete.slice(0, 8)"
                :key="it.run_id"
                class="text-xs flex items-start justify-between gap-2"
              >
                <div class="break-all">{{ it.run_id }}</div>
                <div class="opacity-70 shrink-0">{{ formatUnixSeconds(it.ended_at) }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </n-card>
  </div>
</template>
