<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import {
  NAlert,
  NButton,
  NCard,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NSelect,
  NSpace,
  NSpin,
  NSwitch,
  useMessage,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import {
  useHubRuntimeConfigStore,
  type ConfigValueSource,
  type HubRuntimeConfigFieldMeta,
  type HubRuntimeConfigGetResponse,
} from '@/stores/hubRuntimeConfig'
import { formatToastError, toApiErrorInfo } from '@/lib/errors'

const { t } = useI18n()
const message = useMessage()
const hubRuntimeConfig = useHubRuntimeConfigStore()

const loading = ref(false)
const saving = ref(false)
const loadError = ref<string | null>(null)
const saveError = ref<string | null>(null)
const data = ref<HubRuntimeConfigGetResponse | null>(null)

const fieldErrors = reactive<{
  hub_timezone?: string
  run_retention_days?: string
  incomplete_cleanup_days?: string
  log_rotation?: string
}>({})

const form = reactive<{
  hub_timezone: string
  run_retention_days: number | null
  incomplete_cleanup_days: number | null
  log_filter: string
  log_file: string
  log_rotation: string | null
  log_keep_files: number | null
  default_backup_retention_enabled: boolean
  default_backup_retention_keep_last: number | null
  default_backup_retention_keep_days: number | null
  default_backup_retention_max_delete_per_tick: number
  default_backup_retention_max_delete_per_day: number
}>({
  hub_timezone: '',
  run_retention_days: null,
  incomplete_cleanup_days: null,
  log_filter: '',
  log_file: '',
  log_rotation: null,
  log_keep_files: null,
  default_backup_retention_enabled: false,
  default_backup_retention_keep_last: null,
  default_backup_retention_keep_days: null,
  default_backup_retention_max_delete_per_tick: 50,
  default_backup_retention_max_delete_per_day: 200,
})

function isOverridden(source: ConfigValueSource): boolean {
  return source === 'cli' || source === 'env' || source === 'env_rust_log'
}

function formatSource(source: ConfigValueSource): string {
  const map: Record<ConfigValueSource, string> = {
    cli: t('settings.hubRuntimeConfig.source.cli'),
    env: t('settings.hubRuntimeConfig.source.env'),
    env_rust_log: t('settings.hubRuntimeConfig.source.envRustLog'),
    db: t('settings.hubRuntimeConfig.source.db'),
    default: t('settings.hubRuntimeConfig.source.default'),
  }
  return map[source] ?? source
}

function formatBool(value: boolean): string {
  return value ? t('common.yes') : t('common.no')
}

function normalizeString(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const v = value.trim()
  return v ? v : null
}

function isPendingString(saved: unknown, effective: string): boolean {
  const s = normalizeString(saved)
  return s !== null && s !== effective
}

function isPendingNumber(saved: unknown, effective: number): boolean {
  return typeof saved === 'number' && saved !== effective
}

function isPendingOptionalString(saved: unknown, effective: unknown): boolean {
  const s = normalizeString(saved)
  const e = normalizeString(effective)
  return s !== null && s !== e
}

function clearErrors(): void {
  loadError.value = null
  saveError.value = null
  fieldErrors.hub_timezone = undefined
  fieldErrors.run_retention_days = undefined
  fieldErrors.incomplete_cleanup_days = undefined
  fieldErrors.log_rotation = undefined
}

function applySavedToForm(saved: HubRuntimeConfigGetResponse['saved']): void {
  form.hub_timezone = saved.hub_timezone ?? ''
  form.run_retention_days = typeof saved.run_retention_days === 'number' ? saved.run_retention_days : null
  form.incomplete_cleanup_days =
    typeof saved.incomplete_cleanup_days === 'number' ? saved.incomplete_cleanup_days : null
  form.log_filter = saved.log_filter ?? ''
  form.log_file = saved.log_file ?? ''
  form.log_rotation = saved.log_rotation ?? null
  form.log_keep_files = typeof saved.log_keep_files === 'number' ? saved.log_keep_files : null

  const r = saved.default_backup_retention
  if (r) {
    form.default_backup_retention_enabled = !!r.enabled
    form.default_backup_retention_keep_last = typeof r.keep_last === 'number' ? r.keep_last : null
    form.default_backup_retention_keep_days = typeof r.keep_days === 'number' ? r.keep_days : null
    form.default_backup_retention_max_delete_per_tick =
      typeof r.max_delete_per_tick === 'number' && r.max_delete_per_tick > 0 ? r.max_delete_per_tick : 50
    form.default_backup_retention_max_delete_per_day =
      typeof r.max_delete_per_day === 'number' && r.max_delete_per_day > 0 ? r.max_delete_per_day : 200
  } else {
    form.default_backup_retention_enabled = false
    form.default_backup_retention_keep_last = null
    form.default_backup_retention_keep_days = null
    form.default_backup_retention_max_delete_per_tick = 50
    form.default_backup_retention_max_delete_per_day = 200
  }
}

async function refresh(): Promise<void> {
  loading.value = true
  clearErrors()
  try {
    const resp = await hubRuntimeConfig.get()
    data.value = resp
    applySavedToForm(resp.saved)
  } catch (error) {
    loadError.value = String(error)
    message.error(formatToastError(t('errors.fetchHubRuntimeConfigFailed'), error, t))
  } finally {
    loading.value = false
  }
}

const rotationOptions = computed(() => [
  { label: t('settings.hubRuntimeConfig.rotation.daily'), value: 'daily' },
  { label: t('settings.hubRuntimeConfig.rotation.hourly'), value: 'hourly' },
  { label: t('settings.hubRuntimeConfig.rotation.never'), value: 'never' },
])

function renderMeta(meta: HubRuntimeConfigFieldMeta, pending: boolean): string {
  const chunks: string[] = []
  chunks.push(`${t('settings.hubRuntimeConfig.meta.env')}: ${meta.env}`)
  chunks.push(`${t('settings.hubRuntimeConfig.meta.source')}: ${formatSource(meta.source)}`)
  if (pending) chunks.push(t('settings.hubRuntimeConfig.tags.pending'))
  if (isOverridden(meta.source)) chunks.push(t('settings.hubRuntimeConfig.tags.overridden'))
  return chunks.join(' · ')
}

function normalizeOptionalPositiveInt(value: number | null): number | null {
  if (typeof value !== 'number') return null
  const n = Math.floor(value)
  return n > 0 ? n : null
}

async function save(): Promise<void> {
  if (!data.value) return

  saving.value = true
  clearErrors()
  try {
    const payload = {
      hub_timezone: form.hub_timezone.trim() ? form.hub_timezone.trim() : null,
      run_retention_days: form.run_retention_days,
      incomplete_cleanup_days: form.incomplete_cleanup_days,
      log_filter: form.log_filter.trim() ? form.log_filter.trim() : null,
      log_file: form.log_file.trim() ? form.log_file.trim() : null,
      log_rotation: form.log_rotation,
      log_keep_files: form.log_keep_files,
      default_backup_retention: {
        enabled: form.default_backup_retention_enabled,
        keep_last: normalizeOptionalPositiveInt(form.default_backup_retention_keep_last),
        keep_days: normalizeOptionalPositiveInt(form.default_backup_retention_keep_days),
        max_delete_per_tick: Math.max(1, Math.floor(form.default_backup_retention_max_delete_per_tick || 1)),
        max_delete_per_day: Math.max(1, Math.floor(form.default_backup_retention_max_delete_per_day || 1)),
      },
    }
    await hubRuntimeConfig.save(payload)
    message.success(t('messages.hubRuntimeConfigSaved'))
    await refresh()
  } catch (error) {
    const info = toApiErrorInfo(error)
    saveError.value = info?.message ?? String(error)

    if (info?.code === 'invalid_timezone') fieldErrors.hub_timezone = t('apiErrors.invalid_timezone')
    if (info?.code === 'invalid_run_retention_days') fieldErrors.run_retention_days = t('apiErrors.invalid_run_retention_days')
    if (info?.code === 'invalid_incomplete_cleanup_days') {
      fieldErrors.incomplete_cleanup_days = t('apiErrors.invalid_incomplete_cleanup_days')
    }
    if (info?.code === 'invalid_log_rotation') fieldErrors.log_rotation = t('apiErrors.invalid_log_rotation')

    message.error(formatToastError(t('errors.saveHubRuntimeConfigFailed'), error, t))
  } finally {
    saving.value = false
  }
}

onMounted(refresh)
</script>

<template>
  <n-card class="app-card" :title="t('settings.hubRuntimeConfig.title')">
    <template #header-extra>
      <n-space size="small">
        <n-button size="small" :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
        <n-button size="small" type="primary" :loading="saving" :disabled="!data" @click="save">{{ t('common.save') }}</n-button>
      </n-space>
    </template>

    <div class="space-y-6">
      <n-alert type="warning" :bordered="false">
        {{ t('settings.hubRuntimeConfig.restartRequired') }}
      </n-alert>

      <n-alert v-if="loadError" type="error" :bordered="false">
        {{ loadError }}
      </n-alert>
      <n-alert v-if="saveError" type="error" :bordered="false">
        {{ saveError }}
      </n-alert>

      <div v-if="loading && !data" class="py-10 flex justify-center">
        <n-spin size="small" />
      </div>

      <div v-else-if="data" class="space-y-10">
        <div class="space-y-3">
          <div class="flex items-baseline justify-between gap-3 flex-wrap">
            <h3 class="text-base font-semibold">{{ t('settings.hubRuntimeConfig.sections.startup.title') }}</h3>
            <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.sections.startup.subtitle') }}</div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div class="p-3 rounded-lg app-border-subtle app-glass-soft">
              <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.fields.bindHost') }}</div>
              <div class="font-medium mt-1">{{ data.effective.bind_host }}</div>
              <div class="text-xs opacity-70 mt-1">
                {{ renderMeta(data.fields.bind_host, false) }}
              </div>
            </div>
            <div class="p-3 rounded-lg app-border-subtle app-glass-soft">
              <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.fields.bindPort') }}</div>
              <div class="font-medium mt-1">{{ data.effective.bind_port }}</div>
              <div class="text-xs opacity-70 mt-1">
                {{ renderMeta(data.fields.bind_port, false) }}
              </div>
            </div>
            <div class="p-3 rounded-lg app-border-subtle app-glass-soft">
              <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.fields.dataDir') }}</div>
              <div class="font-medium mt-1 break-all">{{ data.effective.data_dir }}</div>
              <div class="text-xs opacity-70 mt-1">
                {{ renderMeta(data.fields.data_dir, false) }}
              </div>
            </div>
            <div class="p-3 rounded-lg app-border-subtle app-glass-soft">
              <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.fields.insecureHttp') }}</div>
              <div class="font-medium mt-1">{{ formatBool(data.effective.insecure_http) }}</div>
              <div class="text-xs opacity-70 mt-1">
                {{ renderMeta(data.fields.insecure_http, false) }}
              </div>
            </div>
            <div class="p-3 rounded-lg app-border-subtle app-glass-soft">
              <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.fields.trustedProxies') }}</div>
              <div class="font-medium mt-1 break-all">
                {{ data.effective.trusted_proxies.length ? data.effective.trusted_proxies.join(', ') : '-' }}
              </div>
              <div class="text-xs opacity-70 mt-1">
                {{ renderMeta(data.fields.trusted_proxies, false) }}
              </div>
            </div>
            <div class="p-3 rounded-lg app-border-subtle app-glass-soft">
              <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.fields.debugErrors') }}</div>
              <div class="font-medium mt-1">{{ formatBool(data.effective.debug_errors) }}</div>
              <div class="text-xs opacity-70 mt-1">
                {{ renderMeta(data.fields.debug_errors, false) }}
              </div>
            </div>
          </div>
        </div>

        <div class="space-y-3">
          <div class="flex items-baseline justify-between gap-3 flex-wrap">
            <h3 class="text-base font-semibold">{{ t('settings.hubRuntimeConfig.sections.policy.title') }}</h3>
            <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.sections.policy.subtitle') }}</div>
          </div>

          <n-form label-placement="top">
            <n-form-item
              :label="t('settings.hubRuntimeConfig.fields.hubTimezone')"
              :validation-status="fieldErrors.hub_timezone ? 'error' : undefined"
              :feedback="fieldErrors.hub_timezone"
            >
              <div class="space-y-1 w-full">
                <n-input v-model:value="form.hub_timezone" :disabled="!data.fields.hub_timezone.editable" />
                <div class="text-xs opacity-70">
                  {{ renderMeta(data.fields.hub_timezone, isPendingString(data.saved.hub_timezone, data.effective.hub_timezone)) }}
                  <span class="ml-1">·</span>
                  <span class="ml-1">{{ t('settings.hubRuntimeConfig.meta.effective') }}: {{ data.effective.hub_timezone }}</span>
                </div>
              </div>
            </n-form-item>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <n-form-item
                :label="t('settings.hubRuntimeConfig.fields.runRetentionDays')"
                :validation-status="fieldErrors.run_retention_days ? 'error' : undefined"
                :feedback="fieldErrors.run_retention_days"
              >
                <div class="space-y-1 w-full">
                  <n-input-number
                    v-model:value="form.run_retention_days"
                    :min="1"
                    :disabled="!data.fields.run_retention_days.editable"
                  />
                  <div class="text-xs opacity-70">
                    {{
                      renderMeta(
                        data.fields.run_retention_days,
                        isPendingNumber(data.saved.run_retention_days, data.effective.run_retention_days),
                      )
                    }}
                    <span class="ml-1">·</span>
                    <span class="ml-1">{{ t('settings.hubRuntimeConfig.meta.effective') }}: {{ data.effective.run_retention_days }}</span>
                  </div>
                </div>
              </n-form-item>

              <n-form-item
                :label="t('settings.hubRuntimeConfig.fields.incompleteCleanupDays')"
                :validation-status="fieldErrors.incomplete_cleanup_days ? 'error' : undefined"
                :feedback="fieldErrors.incomplete_cleanup_days"
              >
                <div class="space-y-1 w-full">
                  <n-input-number
                    v-model:value="form.incomplete_cleanup_days"
                    :min="0"
                    :disabled="!data.fields.incomplete_cleanup_days.editable"
                  />
                  <div class="text-xs opacity-70">
                    {{
                      renderMeta(
                        data.fields.incomplete_cleanup_days,
                        isPendingNumber(data.saved.incomplete_cleanup_days, data.effective.incomplete_cleanup_days),
                      )
                    }}
                    <span class="ml-1">·</span>
                    <span class="ml-1">{{ t('settings.hubRuntimeConfig.meta.effective') }}: {{ data.effective.incomplete_cleanup_days }}</span>
                  </div>
                </div>
              </n-form-item>
            </div>
          </n-form>
        </div>

        <div class="space-y-3">
          <div class="flex items-baseline justify-between gap-3 flex-wrap">
            <h3 class="text-base font-semibold">{{ t('settings.hubRuntimeConfig.sections.backupRetention.title') }}</h3>
            <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.sections.backupRetention.subtitle') }}</div>
          </div>

          <n-form label-placement="top">
            <n-form-item :label="t('settings.hubRuntimeConfig.fields.backupRetentionEnabled')">
              <div class="flex items-center justify-between gap-3 w-full">
                <div class="text-xs opacity-70">
                  {{ t('settings.hubRuntimeConfig.fields.backupRetentionEnabledHelp') }}
                </div>
                <n-switch v-model:value="form.default_backup_retention_enabled" />
              </div>
            </n-form-item>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <n-form-item :label="t('settings.hubRuntimeConfig.fields.backupRetentionKeepLast')">
                <n-input-number v-model:value="form.default_backup_retention_keep_last" :min="0" />
              </n-form-item>
              <n-form-item :label="t('settings.hubRuntimeConfig.fields.backupRetentionKeepDays')">
                <n-input-number v-model:value="form.default_backup_retention_keep_days" :min="0" />
              </n-form-item>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <n-form-item :label="t('settings.hubRuntimeConfig.fields.backupRetentionMaxDeletePerTick')">
                <n-input-number v-model:value="form.default_backup_retention_max_delete_per_tick" :min="1" />
              </n-form-item>
              <n-form-item :label="t('settings.hubRuntimeConfig.fields.backupRetentionMaxDeletePerDay')">
                <n-input-number v-model:value="form.default_backup_retention_max_delete_per_day" :min="1" />
              </n-form-item>
            </div>

            <n-alert type="info" :bordered="false" class="mt-2">
              {{ t('settings.hubRuntimeConfig.fields.backupRetentionTips') }}
            </n-alert>
          </n-form>
        </div>

        <div class="space-y-3">
          <div class="flex items-baseline justify-between gap-3 flex-wrap">
            <h3 class="text-base font-semibold">{{ t('settings.hubRuntimeConfig.sections.logging.title') }}</h3>
            <div class="text-xs opacity-70">{{ t('settings.hubRuntimeConfig.sections.logging.subtitle') }}</div>
          </div>

          <n-form label-placement="top">
            <n-form-item :label="t('settings.hubRuntimeConfig.fields.logFilter')">
              <div class="space-y-1 w-full">
                <n-input v-model:value="form.log_filter" :disabled="!data.fields.log_filter.editable" />
                <div class="text-xs opacity-70">
                  {{ renderMeta(data.fields.log_filter, isPendingString(data.saved.log_filter, data.effective.log_filter)) }}
                  <span class="ml-1">·</span>
                  <span class="ml-1">{{ t('settings.hubRuntimeConfig.meta.effective') }}: {{ data.effective.log_filter }}</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item :label="t('settings.hubRuntimeConfig.fields.logFile')">
              <div class="space-y-1 w-full">
                <n-input v-model:value="form.log_file" :disabled="!data.fields.log_file.editable" />
                <div class="text-xs opacity-70">
                  {{ renderMeta(data.fields.log_file, isPendingOptionalString(data.saved.log_file, data.effective.log_file)) }}
                  <span class="ml-1">·</span>
                  <span class="ml-1">{{ t('settings.hubRuntimeConfig.meta.effective') }}: {{ data.effective.log_file ?? '-' }}</span>
                </div>
              </div>
            </n-form-item>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <n-form-item
                :label="t('settings.hubRuntimeConfig.fields.logRotation')"
                :validation-status="fieldErrors.log_rotation ? 'error' : undefined"
                :feedback="fieldErrors.log_rotation"
              >
                <div class="space-y-1 w-full">
                  <n-select
                    v-model:value="form.log_rotation"
                    :clearable="true"
                    :options="rotationOptions"
                    :disabled="!data.fields.log_rotation.editable"
                  />
                  <div class="text-xs opacity-70">
                    {{ renderMeta(data.fields.log_rotation, isPendingString(data.saved.log_rotation, data.effective.log_rotation)) }}
                    <span class="ml-1">·</span>
                    <span class="ml-1">{{ t('settings.hubRuntimeConfig.meta.effective') }}: {{ data.effective.log_rotation }}</span>
                  </div>
                </div>
              </n-form-item>

              <n-form-item :label="t('settings.hubRuntimeConfig.fields.logKeepFiles')">
                <div class="space-y-1 w-full">
                  <n-input-number
                    v-model:value="form.log_keep_files"
                    :min="0"
                    :disabled="!data.fields.log_keep_files.editable"
                  />
                  <div class="text-xs opacity-70">
                    {{
                      renderMeta(
                        data.fields.log_keep_files,
                        isPendingNumber(data.saved.log_keep_files, data.effective.log_keep_files),
                      )
                    }}
                    <span class="ml-1">·</span>
                    <span class="ml-1">{{ t('settings.hubRuntimeConfig.meta.effective') }}: {{ data.effective.log_keep_files }}</span>
                  </div>
                </div>
              </n-form-item>
            </div>
          </n-form>
        </div>
      </div>
    </div>
  </n-card>
</template>
