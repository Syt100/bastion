<script setup lang="ts">
import { NAlert, NButton, NDropdown, NFormItem, NInput, NInputNumber, NSelect, NSpin, NSwitch, useMessage, type DropdownOption } from 'naive-ui'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type RetentionPreviewResponse } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'

import { useJobEditorContext } from '../context'
import { simpleScheduleToCron } from '../schedule'

type Option = { label: string; value: string }

defineProps<{
  nodeOptions: Array<Option>
  jobTypeOptions: Array<Option>
  overlapOptions: Array<Option>
}>()

const { t } = useI18n()
const message = useMessage()
const jobs = useJobsStore()
const ui = useUiStore()
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const { form, fieldErrors, lockedNodeId, clearFieldError, onJobTypeChanged } = useJobEditorContext()

const retentionPreview = ref<RetentionPreviewResponse | null>(null)
const retentionLoading = ref<boolean>(false)
const retentionApplying = ref<boolean>(false)
const retentionError = ref<string | null>(null)

function normalizeOptionalPositiveInt(value: number | null): number | null {
  if (typeof value !== 'number') return null
  const n = Math.floor(value)
  return n > 0 ? n : null
}

function buildRetentionPayload() {
  return {
    enabled: !!form.retentionEnabled,
    keep_last: normalizeOptionalPositiveInt(form.retentionKeepLast),
    keep_days: normalizeOptionalPositiveInt(form.retentionKeepDays),
    max_delete_per_tick: Math.max(1, Math.floor(form.retentionMaxDeletePerTick || 1)),
    max_delete_per_day: Math.max(1, Math.floor(form.retentionMaxDeletePerDay || 1)),
  }
}

async function previewRetention(): Promise<void> {
  if (!form.id) {
    message.info(t('jobs.retention.saveFirst'))
    return
  }
  retentionLoading.value = true
  retentionError.value = null
  try {
    retentionPreview.value = await jobs.previewJobRetention(form.id, buildRetentionPayload())
  } catch (error) {
    retentionError.value = String(error)
    message.error(formatToastError(t('errors.previewRetentionFailed'), error, t))
  } finally {
    retentionLoading.value = false
  }
}

async function applyRetention(): Promise<void> {
  if (!form.id) {
    message.info(t('jobs.retention.saveFirst'))
    return
  }
  if (!form.retentionEnabled) {
    message.warning(t('jobs.retention.enableFirst'))
    return
  }
  retentionApplying.value = true
  retentionError.value = null
  try {
    const resp = await jobs.applyJobRetention(form.id, buildRetentionPayload())
    message.success(
      t('jobs.retention.applyOk', { n: resp.enqueued.length, existing: resp.already_exists, skipped: resp.skipped_due_to_limits }),
    )
    await previewRetention()
  } catch (error) {
    retentionError.value = String(error)
    message.error(formatToastError(t('errors.applyRetentionFailed'), error, t))
  } finally {
    retentionApplying.value = false
  }
}

watch(
  () => [
    form.retentionEnabled,
    form.retentionKeepLast,
    form.retentionKeepDays,
    form.retentionMaxDeletePerTick,
    form.retentionMaxDeletePerDay,
  ],
  () => {
    retentionPreview.value = null
    retentionError.value = null
  },
)

function listTimezones(): string[] {
  const api = Intl as unknown as { supportedValuesOf?: (kind: string) => unknown }
  if (typeof api.supportedValuesOf === 'function') {
    try {
      const v = api.supportedValuesOf('timeZone')
      if (Array.isArray(v) && v.every((x) => typeof x === 'string')) {
        return v as string[]
      }
    } catch {
      // ignore
    }
  }
  return ['UTC']
}

const timezoneOptions = computed<Option[]>(() => {
  const current = form.scheduleTimezone?.trim() || 'UTC'
  const set = new Set<string>(['UTC', current, ...listTimezones()])
  return [...set].map((tz) => ({ label: tz, value: tz }))
})

const scheduleModeOptions = computed(() => [
  { label: t('jobs.scheduleMode.manual'), value: 'manual' as const },
  { label: t('jobs.scheduleMode.simple'), value: 'simple' as const },
  { label: t('jobs.scheduleMode.cron'), value: 'cron' as const },
])

const simpleKindOptions = computed(() => [
  { label: t('jobs.simpleSchedule.everyMinutes'), value: 'every_minutes' as const },
  { label: t('jobs.simpleSchedule.hourly'), value: 'hourly' as const },
  { label: t('jobs.simpleSchedule.daily'), value: 'daily' as const },
  { label: t('jobs.simpleSchedule.weekly'), value: 'weekly' as const },
  { label: t('jobs.simpleSchedule.monthly'), value: 'monthly' as const },
])

const weekdayOptions = computed(() => [
  { label: t('jobs.weekdays.sun'), value: 0 },
  { label: t('jobs.weekdays.mon'), value: 1 },
  { label: t('jobs.weekdays.tue'), value: 2 },
  { label: t('jobs.weekdays.wed'), value: 3 },
  { label: t('jobs.weekdays.thu'), value: 4 },
  { label: t('jobs.weekdays.fri'), value: 5 },
  { label: t('jobs.weekdays.sat'), value: 6 },
])

const cronPresets = computed<DropdownOption[]>(() => [
  { label: t('jobs.cronPresets.manual'), key: '__manual__' },
  { label: `${t('jobs.cronPresets.hourly')} (0 * * * *)`, key: '0 * * * *' },
  { label: `${t('jobs.cronPresets.every15m')} (*/15 * * * *)`, key: '*/15 * * * *' },
  { label: `${t('jobs.cronPresets.daily')} (0 0 * * *)`, key: '0 0 * * *' },
  { label: `${t('jobs.cronPresets.weekly')} (0 0 * * 0)`, key: '0 0 * * 0' },
  { label: `${t('jobs.cronPresets.monthly')} (0 0 1 * *)`, key: '0 0 1 * *' },
])

function applyCronPreset(key: string | number): void {
  const k = String(key)
  form.schedule = k === '__manual__' ? '' : k
  clearFieldError('schedule')
}

function applySimpleSchedule(): void {
  const schedule = simpleScheduleToCron({
    kind: form.simpleScheduleKind,
    everyMinutes: form.simpleEveryMinutes,
    atHour: form.simpleAtHour,
    atMinute: form.simpleAtMinute,
    weekday: form.simpleWeekday,
    monthday: form.simpleMonthday,
  })
  form.schedule = schedule
  clearFieldError('schedule')
}

watch(
  () => form.scheduleMode,
  (mode) => {
    if (mode === 'manual') {
      form.schedule = ''
      clearFieldError('schedule')
      return
    }
    if (mode === 'simple') {
      applySimpleSchedule()
    }
  },
)

watch(
  () => [
    form.simpleScheduleKind,
    form.simpleEveryMinutes,
    form.simpleAtHour,
    form.simpleAtMinute,
    form.simpleWeekday,
    form.simpleMonthday,
  ],
  () => {
    if (form.scheduleMode !== 'simple') return
    applySimpleSchedule()
  },
)
</script>

<template>
  <div class="space-y-4 app-border-subtle rounded-lg p-3 app-glass-soft">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <div data-field="name">
        <n-form-item
          :label="t('jobs.fields.name')"
          required
          :validation-status="fieldErrors.name ? 'error' : undefined"
          :feedback="fieldErrors.name || undefined"
        >
          <n-input v-model:value="form.name" @update:value="clearFieldError('name')" />
        </n-form-item>
      </div>
      <n-form-item :label="t('jobs.fields.node')">
        <n-select
          v-model:value="form.node"
          :options="nodeOptions"
          filterable
          :disabled="lockedNodeId !== null"
        />
      </n-form-item>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
      <n-form-item :label="t('jobs.fields.type')">
        <n-select v-model:value="form.jobType" :options="jobTypeOptions" @update:value="onJobTypeChanged" />
      </n-form-item>
      <n-form-item :label="t('jobs.fields.overlap')">
        <n-select v-model:value="form.overlapPolicy" :options="overlapOptions" />
      </n-form-item>
    </div>

    <div class="space-y-3">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
        <div data-field="scheduleTimezone">
          <n-form-item
            :label="t('jobs.fields.scheduleTimezone')"
            required
            :validation-status="fieldErrors.scheduleTimezone ? 'error' : undefined"
            :feedback="fieldErrors.scheduleTimezone || undefined"
          >
            <n-select v-model:value="form.scheduleTimezone" filterable :options="timezoneOptions" />
          </n-form-item>
        </div>
        <n-form-item :label="t('jobs.fields.scheduleMode')">
          <n-select v-model:value="form.scheduleMode" :options="scheduleModeOptions" />
        </n-form-item>
      </div>

      <div class="text-xs opacity-70">
        {{ t('jobs.fields.scheduleTimezoneHelp') }}
      </div>
      <div class="text-xs opacity-70">
        {{ t('jobs.fields.scheduleDstHelp') }}
      </div>

      <template v-if="form.scheduleMode === 'manual'">
        <div class="text-sm opacity-70">{{ t('jobs.fields.scheduleManualHelp') }}</div>
      </template>

      <template v-else-if="form.scheduleMode === 'simple'">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
          <n-form-item :label="t('jobs.fields.simpleSchedule')">
            <n-select v-model:value="form.simpleScheduleKind" :options="simpleKindOptions" />
          </n-form-item>

          <template v-if="form.simpleScheduleKind === 'every_minutes'">
            <n-form-item :label="t('jobs.fields.everyMinutes')">
              <n-input-number v-model:value="form.simpleEveryMinutes" :min="1" :max="59" class="w-full" />
            </n-form-item>
          </template>

          <template v-else-if="form.simpleScheduleKind === 'hourly'">
            <n-form-item :label="t('jobs.fields.atMinute')">
              <n-input-number v-model:value="form.simpleAtMinute" :min="0" :max="59" class="w-full" />
            </n-form-item>
          </template>

          <template v-else-if="form.simpleScheduleKind === 'weekly'">
            <n-form-item :label="t('jobs.fields.weekday')">
              <n-select v-model:value="form.simpleWeekday" :options="weekdayOptions" />
            </n-form-item>
          </template>

          <template v-else-if="form.simpleScheduleKind === 'monthly'">
            <n-form-item :label="t('jobs.fields.monthday')">
              <n-input-number v-model:value="form.simpleMonthday" :min="1" :max="28" class="w-full" />
            </n-form-item>
          </template>
        </div>

        <template v-if="form.simpleScheduleKind !== 'every_minutes' && form.simpleScheduleKind !== 'hourly'">
          <div class="grid grid-cols-2 gap-2">
            <n-form-item :label="t('jobs.fields.atHour')">
              <n-input-number v-model:value="form.simpleAtHour" :min="0" :max="23" class="w-full" />
            </n-form-item>
            <n-form-item :label="t('jobs.fields.atMinute')">
              <n-input-number v-model:value="form.simpleAtMinute" :min="0" :max="59" class="w-full" />
            </n-form-item>
          </div>
        </template>

        <n-form-item
          :label="t('jobs.fields.generatedCron')"
          :validation-status="fieldErrors.schedule ? 'error' : undefined"
          :feedback="fieldErrors.schedule || undefined"
        >
          <n-input :value="form.schedule" disabled />
        </n-form-item>
      </template>

      <template v-else>
        <div data-field="schedule">
          <n-form-item
            :label="t('jobs.fields.schedule')"
            :validation-status="fieldErrors.schedule ? 'error' : undefined"
            :feedback="fieldErrors.schedule || undefined"
          >
            <div class="space-y-2 w-full">
              <n-input
                v-model:value="form.schedule"
                :placeholder="t('jobs.fields.schedulePlaceholder')"
                @update:value="clearFieldError('schedule')"
              />
              <div class="flex flex-wrap items-center justify-between gap-2">
                <div v-if="!fieldErrors.schedule" class="text-xs opacity-70">{{ t('jobs.fields.scheduleHelp') }}</div>
                <n-dropdown :options="cronPresets" @select="applyCronPreset">
                  <n-button size="tiny" secondary>{{ t('jobs.actions.cronPresets') }}</n-button>
                </n-dropdown>
              </div>
            </div>
          </n-form-item>
        </div>
      </template>
    </div>

    <div class="space-y-3 app-border-subtle rounded-lg p-3 app-glass-soft">
      <div class="flex items-center justify-between gap-3">
        <div class="text-sm font-medium">{{ t('jobs.retention.title') }}</div>
        <n-switch v-model:value="form.retentionEnabled" />
      </div>

      <div class="text-xs opacity-70">
        {{ t('jobs.retention.help') }}
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
        <div data-field="retentionKeepLast">
          <n-form-item
            :label="t('jobs.retention.keepLast')"
            :validation-status="fieldErrors.retentionKeepLast ? 'error' : undefined"
            :feedback="fieldErrors.retentionKeepLast || undefined"
          >
            <n-input-number
              v-model:value="form.retentionKeepLast"
              :min="0"
              class="w-full"
              @update:value="clearFieldError('retentionKeepLast')"
            />
          </n-form-item>
        </div>
        <div data-field="retentionKeepDays">
          <n-form-item
            :label="t('jobs.retention.keepDays')"
            :validation-status="fieldErrors.retentionKeepDays ? 'error' : undefined"
            :feedback="fieldErrors.retentionKeepDays || undefined"
          >
            <n-input-number
              v-model:value="form.retentionKeepDays"
              :min="0"
              class="w-full"
              @update:value="clearFieldError('retentionKeepDays')"
            />
          </n-form-item>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
        <div data-field="retentionMaxDeletePerTick">
          <n-form-item
            :label="t('jobs.retention.maxDeletePerTick')"
            :validation-status="fieldErrors.retentionMaxDeletePerTick ? 'error' : undefined"
            :feedback="fieldErrors.retentionMaxDeletePerTick || undefined"
          >
            <n-input-number
              v-model:value="form.retentionMaxDeletePerTick"
              :min="1"
              class="w-full"
              @update:value="clearFieldError('retentionMaxDeletePerTick')"
            />
          </n-form-item>
        </div>
        <div data-field="retentionMaxDeletePerDay">
          <n-form-item
            :label="t('jobs.retention.maxDeletePerDay')"
            :validation-status="fieldErrors.retentionMaxDeletePerDay ? 'error' : undefined"
            :feedback="fieldErrors.retentionMaxDeletePerDay || undefined"
          >
            <n-input-number
              v-model:value="form.retentionMaxDeletePerDay"
              :min="1"
              class="w-full"
              @update:value="clearFieldError('retentionMaxDeletePerDay')"
            />
          </n-form-item>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <n-button size="small" secondary :loading="retentionLoading" :disabled="!form.id" @click="previewRetention">
          {{ t('jobs.retention.preview') }}
        </n-button>
        <n-button
          size="small"
          type="primary"
          :loading="retentionApplying"
          :disabled="!form.id || !form.retentionEnabled"
          @click="applyRetention"
        >
          {{ t('jobs.retention.applyNow') }}
        </n-button>
        <div v-if="!form.id" class="text-xs opacity-70">{{ t('jobs.retention.previewDisabledUntilSaved') }}</div>
      </div>

      <n-alert v-if="retentionError" type="error" :bordered="false">
        {{ retentionError }}
      </n-alert>

      <div v-if="retentionLoading" class="flex justify-center py-2">
        <n-spin size="small" />
      </div>

      <div v-else-if="retentionPreview" class="space-y-2">
        <div class="text-sm">
          {{ t('jobs.retention.previewSummary', { keep: retentionPreview.keep_total, del: retentionPreview.delete_total }) }}
          <span v-if="retentionPreview.result_truncated" class="text-xs opacity-70 ml-2">
            {{ t('jobs.retention.previewTruncated') }}
          </span>
        </div>

        <div v-if="retentionPreview.delete_total === 0" class="text-xs opacity-70">
          {{ t('jobs.retention.previewNoDeletes') }}
        </div>
        <div v-else class="app-border-subtle rounded-md p-2">
          <div class="text-xs opacity-70 mb-1">{{ t('jobs.retention.deleteList') }}</div>
          <div class="space-y-1">
            <div
              v-for="it in retentionPreview.delete.slice(0, 8)"
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
  </div>
</template>
