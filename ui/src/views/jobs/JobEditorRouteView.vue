<script setup lang="ts">
import { computed, nextTick, provide, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NAlert, NButton, NCard, NForm, NStep, NSteps, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import FsPathPickerModal, { type FsPathPickerModalExpose } from '@/components/fs/FsPathPickerModal.vue'
import JobEditorStepBasicsOnly from '@/components/jobs/editor/steps/JobEditorStepBasicsOnly.vue'
import JobEditorStepNotifications from '@/components/jobs/editor/steps/JobEditorStepNotifications.vue'
import JobEditorStepReview from '@/components/jobs/editor/steps/JobEditorStepReview.vue'
import JobEditorStepScheduleRetention from '@/components/jobs/editor/steps/JobEditorStepScheduleRetention.vue'
import JobEditorStepSecurity from '@/components/jobs/editor/steps/JobEditorStepSecurity.vue'
import JobEditorStepSource from '@/components/jobs/editor/steps/JobEditorStepSource.vue'
import JobEditorStepTarget from '@/components/jobs/editor/steps/JobEditorStepTarget.vue'
import { jobEditorContextKey } from '@/components/jobs/editor/context'
import { createInitialJobEditorFieldErrors, createInitialJobEditorForm, resetJobEditorForm } from '@/components/jobs/editor/form'
import { editorFormToRequest, jobDetailToEditorForm } from '@/components/jobs/editor/mapping'
import type { JobEditorField, JobEditorForm } from '@/components/jobs/editor/types'
import { useAgentsStore } from '@/stores/agents'
import { useHubRuntimeConfigStore } from '@/stores/hubRuntimeConfig'
import { useJobsStore } from '@/stores/jobs'
import { useNotificationsStore } from '@/stores/notifications'
import { useSecretsStore } from '@/stores/secrets'
import { useSystemStore } from '@/stores/system'
import { MQ } from '@/lib/breakpoints'
import { formatToastError, resolveApiFieldErrors, toApiErrorInfo } from '@/lib/errors'
import { buildJobsCollectionLocation, readJobsCollectionState, resolveJobsScope } from '@/lib/jobsRoute'
import { useMediaQuery } from '@/lib/media'
import { scopeToNodeId } from '@/lib/scope'

type EditorMode = 'create' | 'edit'
type DraftEnvelope = {
  version: 1
  mode: EditorMode
  jobId: string | null
  baseJobUpdatedAt: number | null
  step: number
  values: JobEditorForm
  updatedAt: number
}

const ROUTE_EDITOR_TOTAL_STEPS = 7
const ROUTE_EDITOR_LAST_INPUT_STEP = 6

const { t } = useI18n()
const message = useMessage()
const route = useRoute()
const router = useRouter()
const isDesktop = useMediaQuery(MQ.mdUp)

const jobs = useJobsStore()
const agents = useAgentsStore()
const secrets = useSecretsStore()
const notifications = useNotificationsStore()
const system = useSystemStore()
const hubRuntimeConfig = useHubRuntimeConfigStore()

const routeCollectionState = computed(() => readJobsCollectionState(route.query, resolveJobsScope(route, 'all')))
const backToJobs = computed(() => router.resolve(buildJobsCollectionLocation(routeCollectionState.value)).fullPath)
const mode = computed<EditorMode>(() => (typeof route.params.jobId === 'string' ? 'edit' : 'create'))
const jobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))
const step = ref<number>(1)
const loading = ref<boolean>(true)
const saving = ref<boolean>(false)
const fsPicker = ref<FsPathPickerModalExpose | null>(null)
const fsPickerPurpose = ref<'source_paths' | 'local_base_dir'>('source_paths')
const fsPathDraft = ref<string>('')
const showJsonPreview = ref<boolean>(false)
const baseJobUpdatedAt = ref<number | null>(null)
const lockedNodeId = ref<'hub' | string | null>(null)
const pendingDraft = ref<DraftEnvelope | null>(null)
const pageBody = ref<HTMLElement | null>(null)
const mobileStepsExpanded = ref<boolean>(false)
const mobileSummaryExpanded = ref<boolean>(false)
const mobileRisksExpanded = ref<boolean>(false)

const form = reactive<JobEditorForm>(createInitialJobEditorForm())
const fieldErrors = reactive<Record<JobEditorField, string | null>>(createInitialJobEditorFieldErrors())

function clearFieldError(field: JobEditorField): void {
  fieldErrors[field] = null
}

function clearAllFieldErrors(): void {
  for (const key of Object.keys(fieldErrors) as JobEditorField[]) {
    fieldErrors[key] = null
  }
}

function resetForm(): void {
  resetJobEditorForm(form)
  Object.assign(fieldErrors, createInitialJobEditorFieldErrors())
}

function parseLines(text: string): string[] {
  return text
    .split(/\r?\n/g)
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
}

function mergeUniqueStrings(target: string[], next: string[]): { merged: string[]; added: number; skipped: number } {
  const existing = new Set(target.map((v) => v.trim()).filter((v) => v.length > 0))
  const out = [...target]
  let added = 0
  let skipped = 0
  for (const raw of next) {
    const value = raw.trim()
    if (!value) continue
    if (existing.has(value)) {
      skipped += 1
      continue
    }
    existing.add(value)
    out.push(value)
    added += 1
  }
  return { merged: out, added, skipped }
}

function routeStepForField(field: JobEditorField): number {
  switch (field) {
    case 'name':
      return 1
    case 'fsPaths':
    case 'fsSnapshotMode':
    case 'fsSnapshotProvider':
    case 'fsConsistencyPolicy':
    case 'fsConsistencyFailThreshold':
    case 'fsUploadOnConsistencyFailure':
    case 'sqlitePath':
    case 'vaultwardenDataDir':
    case 'vaultwardenConsistencyPolicy':
    case 'vaultwardenConsistencyFailThreshold':
    case 'vaultwardenUploadOnConsistencyFailure':
      return 2
    case 'webdavBaseUrl':
    case 'webdavSecretName':
    case 'webdavRawTreeDirectMode':
    case 'webdavRawTreeDirectConcurrency':
    case 'webdavRawTreeDirectPutQps':
    case 'webdavRawTreeDirectHeadQps':
    case 'webdavRawTreeDirectMkcolQps':
    case 'webdavRawTreeDirectBurst':
    case 'webdavRawTreeDirectRequestTimeoutSecs':
    case 'webdavRawTreeDirectConnectTimeoutSecs':
    case 'webdavRawTreeDirectMaxPutAttempts':
    case 'localBaseDir':
    case 'partSizeMiB':
      return 3
    case 'scheduleTimezone':
    case 'schedule':
    case 'retentionKeepLast':
    case 'retentionKeepDays':
    case 'retentionMaxDeletePerTick':
    case 'retentionMaxDeletePerDay':
      return 4
    case 'encryptionKeyName':
      return 5
    default:
      return 1
  }
}

function cronLooksValid(expr: string): boolean {
  const trimmed = expr.trim()
  if (!trimmed) return true
  const parts = trimmed.split(/\s+/g).filter(Boolean)
  if (parts.length !== 5) return false
  return parts.every((part) => /^[0-9*/,\-]+$/.test(part))
}

function validateRouteEditorStep(targetStep: number): Array<{ field: JobEditorField; message: string }> {
  const issues: Array<{ field: JobEditorField; message: string }> = []

  if (targetStep >= 1 && !form.name.trim()) {
    issues.push({ field: 'name', message: t('errors.jobNameRequired') })
  }

  if (targetStep >= 2) {
    if (form.jobType === 'filesystem') {
      if (form.fsPaths.every((value) => !value.trim())) {
        issues.push({ field: 'fsPaths', message: t('errors.sourcePathsRequired') })
      }
      if (form.fsSnapshotMode !== 'off') {
        const selected = form.fsPaths.map((value) => value.trim()).filter(Boolean)
        if (selected.length !== 1) {
          issues.push({ field: 'fsSnapshotMode', message: t('errors.snapshotRequiresSingleSourcePath') })
        }
      }
      if (form.fsConsistencyPolicy === 'fail' && (!Number.isFinite(form.fsConsistencyFailThreshold) || form.fsConsistencyFailThreshold < 0)) {
        issues.push({ field: 'fsConsistencyFailThreshold', message: t('errors.consistencyThresholdInvalid') })
      }
    } else if (form.jobType === 'sqlite') {
      if (!form.sqlitePath.trim()) {
        issues.push({ field: 'sqlitePath', message: t('errors.sqlitePathRequired') })
      }
    } else {
      if (!form.vaultwardenDataDir.trim()) {
        issues.push({ field: 'vaultwardenDataDir', message: t('errors.vaultwardenDataDirRequired') })
      }
      if (
        form.vaultwardenConsistencyPolicy === 'fail' &&
        (!Number.isFinite(form.vaultwardenConsistencyFailThreshold) || form.vaultwardenConsistencyFailThreshold < 0)
      ) {
        issues.push({ field: 'vaultwardenConsistencyFailThreshold', message: t('errors.consistencyThresholdInvalid') })
      }
    }
  }

  if (targetStep >= 3) {
    if (form.targetType === 'webdav') {
      if (!form.webdavBaseUrl.trim()) issues.push({ field: 'webdavBaseUrl', message: t('errors.webdavBaseUrlRequired') })
      if (!form.webdavSecretName.trim()) issues.push({ field: 'webdavSecretName', message: t('errors.webdavSecretRequired') })
    } else if (!form.localBaseDir.trim()) {
      issues.push({ field: 'localBaseDir', message: t('errors.localBaseDirRequired') })
    }

    if (!Number.isFinite(form.partSizeMiB) || form.partSizeMiB <= 0) {
      issues.push({ field: 'partSizeMiB', message: t('errors.partSizeInvalid') })
    }
  }

  if (targetStep >= 4) {
    if (!form.scheduleTimezone.trim()) {
      issues.push({ field: 'scheduleTimezone', message: t('errors.scheduleTimezoneRequired') })
    }
    if (form.scheduleMode === 'cron' && !form.schedule.trim()) {
      issues.push({ field: 'schedule', message: t('errors.cronRequired') })
    }
    if (!cronLooksValid(form.schedule)) {
      issues.push({ field: 'schedule', message: t('errors.invalidCron') })
    }
    if (form.retentionEnabled) {
      const keepLast = typeof form.retentionKeepLast === 'number' ? Math.floor(form.retentionKeepLast) : 0
      const keepDays = typeof form.retentionKeepDays === 'number' ? Math.floor(form.retentionKeepDays) : 0
      if (keepLast <= 0 && keepDays <= 0) {
        issues.push({ field: 'retentionKeepLast', message: t('errors.retentionRuleRequired') })
      }
      if (!Number.isFinite(form.retentionMaxDeletePerTick) || form.retentionMaxDeletePerTick <= 0) {
        issues.push({ field: 'retentionMaxDeletePerTick', message: t('errors.retentionLimitInvalid') })
      }
      if (!Number.isFinite(form.retentionMaxDeletePerDay) || form.retentionMaxDeletePerDay <= 0) {
        issues.push({ field: 'retentionMaxDeletePerDay', message: t('errors.retentionLimitInvalid') })
      }
    }
  }

  if (targetStep >= 5 && form.encryptionEnabled && !form.encryptionKeyName.trim()) {
    issues.push({ field: 'encryptionKeyName', message: t('errors.encryptionKeyNameRequired') })
  }

  return issues
}

async function focusField(field: JobEditorField): Promise<void> {
  await nextTick()
  const root = pageBody.value
  if (!root) return
  const el = root.querySelector(`[data-field="${field}"]`) as HTMLElement | null
  if (!el) return
  el.scrollIntoView({ block: 'center' })
  const focusable = el.querySelector(
    'input, textarea, [contenteditable="true"], [role="combobox"], button, [tabindex]:not([tabindex="-1"])',
  ) as HTMLElement | null
  focusable?.focus?.()
}

async function validateUpTo(stepTarget: number): Promise<boolean> {
  clearAllFieldErrors()
  const issues = validateRouteEditorStep(stepTarget)
  if (issues.length === 0) return true
  for (const issue of issues) {
    fieldErrors[issue.field] = issue.message
  }
  const first = issues[0]!
  const stepForField = routeStepForField(first.field)
  if (step.value !== stepForField) {
    step.value = stepForField
    await nextTick()
  }
  message.error(t('errors.formInvalid'))
  await focusField(first.field)
  return false
}

async function goToStep(target: number): Promise<void> {
  const next = Math.min(ROUTE_EDITOR_TOTAL_STEPS, Math.max(1, Math.floor(target)))
  if (next <= step.value) {
    step.value = next
    return
  }
  const ok = await validateUpTo(next - 1)
  if (!ok) return
  step.value = next
}

function prevStep(): void {
  step.value = Math.max(1, step.value - 1)
}

async function nextStep(): Promise<void> {
  const ok = await validateUpTo(step.value)
  if (!ok) return
  step.value = Math.min(ROUTE_EDITOR_TOTAL_STEPS, step.value + 1)
}

function onJobTypeChanged(): void {
  clearFieldError('fsPaths')
  clearFieldError('fsConsistencyPolicy')
  clearFieldError('fsConsistencyFailThreshold')
  clearFieldError('fsUploadOnConsistencyFailure')
  clearFieldError('sqlitePath')
  clearFieldError('vaultwardenDataDir')
  clearFieldError('vaultwardenConsistencyPolicy')
  clearFieldError('vaultwardenConsistencyFailThreshold')
  clearFieldError('vaultwardenUploadOnConsistencyFailure')
  if (form.jobType !== 'filesystem') form.webdavRawTreeDirectMode = 'off'
}

function onTargetTypeChanged(): void {
  clearFieldError('webdavBaseUrl')
  clearFieldError('webdavSecretName')
  clearFieldError('localBaseDir')
  if (form.targetType !== 'webdav') form.webdavRawTreeDirectMode = 'off'
}

function onArtifactFormatChanged(): void {
  if (form.artifactFormat === 'raw_tree_v1') {
    form.encryptionEnabled = false
  } else {
    form.webdavRawTreeDirectMode = 'off'
  }
  clearFieldError('encryptionKeyName')
}

function onEncryptionEnabledChanged(): void {
  clearFieldError('encryptionKeyName')
}

function openFsPicker(): void {
  fsPickerPurpose.value = 'source_paths'
  fsPicker.value?.open(form.node)
}

function openLocalBaseDirPicker(): void {
  fsPickerPurpose.value = 'local_base_dir'
  const path = form.localBaseDir.trim() || undefined
  fsPicker.value?.open(form.node, { mode: 'single_dir', path })
}

function addFsPathsFromList(paths: string[]): void {
  const { merged, added, skipped } = mergeUniqueStrings(form.fsPaths, paths)
  form.fsPaths = merged
  if (added > 0) clearFieldError('fsPaths')
  if (added > 0) message.success(t('messages.sourcePathsAdded', { count: added }))
  if (skipped > 0) message.warning(t('messages.sourcePathsSkipped', { count: skipped }))
}

function onFsPickerPicked(paths: string[]): void {
  if (fsPickerPurpose.value === 'local_base_dir') {
    form.localBaseDir = paths[0] || ''
    clearFieldError('localBaseDir')
    return
  }
  addFsPathsFromList(paths)
}

function addFsPathsFromDraft(): void {
  const lines = parseLines(fsPathDraft.value)
  if (lines.length === 0) return
  fsPathDraft.value = ''
  addFsPathsFromList(lines)
}

function removeFsPath(path: string): void {
  form.fsPaths = form.fsPaths.filter((item) => item !== path)
}

function clearFsPaths(): void {
  form.fsPaths = []
}

const previewPayload = computed(() => editorFormToRequest(form))

provide(jobEditorContextKey, {
  form,
  fieldErrors,
  lockedNodeId,
  fsPathDraft,
  showJsonPreview,
  previewPayload,
  clearFieldError,
  onJobTypeChanged,
  onTargetTypeChanged,
  onArtifactFormatChanged,
  onEncryptionEnabledChanged,
  openFsPicker,
  openLocalBaseDirPicker,
  addFsPathsFromDraft,
  removeFsPath,
  clearFsPaths,
})

function draftStorageKey(): string {
  return mode.value === 'create'
    ? 'bastion.jobs.editor.createDraft'
    : `bastion.jobs.editor.editDraft.${jobId.value ?? 'unknown'}`
}

function serializeDraft(): DraftEnvelope {
  return {
    version: 1,
    mode: mode.value,
    jobId: jobId.value,
    baseJobUpdatedAt: baseJobUpdatedAt.value,
    step: step.value,
    values: JSON.parse(JSON.stringify(form)) as JobEditorForm,
    updatedAt: Date.now(),
  }
}

function persistDraft(): void {
  localStorage.setItem(draftStorageKey(), JSON.stringify(serializeDraft()))
}

function clearDraft(): void {
  localStorage.removeItem(draftStorageKey())
}

function discardPendingDraft(): void {
  pendingDraft.value = null
  clearDraft()
}

function resumePendingDraft(): void {
  const draft = pendingDraft.value
  if (!draft) return
  Object.assign(form, draft.values)
  step.value = Math.min(ROUTE_EDITOR_TOTAL_STEPS, Math.max(1, draft.step))
  pendingDraft.value = null
}

function loadPendingDraft(): void {
  const raw = localStorage.getItem(draftStorageKey())
  if (!raw) {
    pendingDraft.value = null
    return
  }
  try {
    const parsed = JSON.parse(raw) as DraftEnvelope
    if (!parsed || parsed.version !== 1) {
      pendingDraft.value = null
      return
    }
    pendingDraft.value = parsed
  } catch {
    pendingDraft.value = null
  }
}

async function prepareCreateDefaults(): Promise<void> {
  resetForm()
  step.value = 1
  showJsonPreview.value = false
  lockedNodeId.value = scopeToNodeId(routeCollectionState.value.scope)
  if (lockedNodeId.value) {
    form.node = lockedNodeId.value
  }
  try {
    await system.refresh()
  } catch {
    // ignore
  }
  form.scheduleTimezone = system.hubTimezone || 'UTC'
  try {
    const cfg = await hubRuntimeConfig.get()
    const retention = cfg.saved.default_backup_retention
    if (retention) {
      form.retentionEnabled = !!retention.enabled
      form.retentionKeepLast = typeof retention.keep_last === 'number' ? retention.keep_last : null
      form.retentionKeepDays = typeof retention.keep_days === 'number' ? retention.keep_days : null
      form.retentionMaxDeletePerTick =
        typeof retention.max_delete_per_tick === 'number' && retention.max_delete_per_tick > 0 ? retention.max_delete_per_tick : 50
      form.retentionMaxDeletePerDay =
        typeof retention.max_delete_per_day === 'number' && retention.max_delete_per_day > 0 ? retention.max_delete_per_day : 200
    }
  } catch {
    // ignore
  }
  baseJobUpdatedAt.value = null
}

async function loadEditor(): Promise<void> {
  loading.value = true
  clearAllFieldErrors()
  try {
    await Promise.allSettled([agents.refresh(), notifications.refreshDestinations()])
    if (mode.value === 'create') {
      await prepareCreateDefaults()
      loadPendingDraft()
      return
    }

    const liveJob = await jobs.getJob(jobId.value || '')
    Object.assign(form, jobDetailToEditorForm(liveJob))
    baseJobUpdatedAt.value = liveJob.updated_at
    lockedNodeId.value = null
    loadPendingDraft()
    if (pendingDraft.value && pendingDraft.value.baseJobUpdatedAt !== null && pendingDraft.value.baseJobUpdatedAt !== liveJob.updated_at) {
      // Keep the live job loaded until the operator explicitly resumes the stale draft.
      return
    }
  } catch (error) {
    message.error(formatToastError(t('errors.fetchJobFailed'), error, t))
    await router.replace(backToJobs.value)
  } finally {
    loading.value = false
  }
}

const summaryRows = computed(() => [
  { label: t('jobs.editor.summary.node'), value: form.node === 'hub' ? t('jobs.nodes.hub') : form.node },
  { label: t('jobs.editor.summary.type'), value: t(`jobs.types.${form.jobType}`) },
  { label: t('jobs.editor.summary.target'), value: form.targetType === 'webdav' ? form.webdavBaseUrl.trim() || t('jobs.targets.webdav') : form.localBaseDir.trim() || t('jobs.targets.localDir') },
  { label: t('jobs.editor.summary.schedule'), value: form.schedule.trim() || t('jobs.scheduleMode.manual') },
  { label: t('jobs.editor.summary.notifications'), value: form.notifyMode === 'inherit' ? t('jobs.notifications.inherit') : t('jobs.notifications.custom') },
])
const editorSteps = computed(() => [
  { index: 1, label: t('jobs.steps.basics'), help: '' },
  { index: 2, label: t('jobs.steps.source'), help: t('jobs.steps.sourceHelp') },
  { index: 3, label: t('jobs.steps.target'), help: '' },
  { index: 4, label: t('jobs.steps.scheduleRetention'), help: '' },
  { index: 5, label: t('jobs.steps.security'), help: '' },
  { index: 6, label: t('jobs.steps.notifications'), help: t('jobs.steps.notificationsHelp') },
  { index: 7, label: t('jobs.steps.review'), help: t('jobs.steps.reviewHelp') },
])
const currentEditorStep = computed(() => editorSteps.value.find((item) => item.index === step.value) ?? editorSteps.value[0]!)
const stepProgressPercent = computed(() => Math.round((step.value / ROUTE_EDITOR_TOTAL_STEPS) * 100))
const summaryPeekText = computed(() => (
  [summaryRows.value[0]?.value, summaryRows.value[1]?.value, summaryRows.value[2]?.value]
    .map((value) => String(value ?? '').trim())
    .filter((value) => value.length > 0)
    .slice(0, 2)
    .join(' · ')
))

const riskRows = computed(() => {
  const rows: Array<{ key: string; type: 'warning' | 'error' | 'info' }> = []
  if (!form.schedule.trim()) rows.push({ key: 'manual_only', type: 'warning' })
  if (!form.retentionEnabled) rows.push({ key: 'retention_disabled', type: 'warning' })
  if (form.artifactFormat !== 'raw_tree_v1' && !form.encryptionEnabled) rows.push({ key: 'unencrypted', type: 'warning' })
  if (form.notifyMode === 'custom' && form.notifyWecomBots.length === 0 && form.notifyEmails.length === 0) {
    rows.push({ key: 'notifications_disabled', type: 'warning' })
  }
  if (mode.value === 'edit' && pendingDraft.value && pendingDraft.value.baseJobUpdatedAt !== baseJobUpdatedAt.value) {
    rows.push({ key: 'stale_draft', type: 'error' })
  }
  if (rows.length === 0) rows.push({ key: 'none', type: 'info' })
  return rows
})
const riskRowsNeedingAttention = computed(() => riskRows.value.filter((risk) => risk.key !== 'none'))
const mobileActionWarnings = computed(() => riskRows.value.filter((risk) => risk.type !== 'info').slice(0, 2))
const riskPeekText = computed(() => (
  riskRowsNeedingAttention.value.length > 0
    ? t('jobs.editor.risksPeek', { count: riskRowsNeedingAttention.value.length })
    : t('jobs.editor.risksPeekNone')
))

async function save(): Promise<void> {
  const ok = await validateUpTo(ROUTE_EDITOR_LAST_INPUT_STEP)
  if (!ok) return

  saving.value = true
  try {
    const payload = previewPayload.value
    if (mode.value === 'create') {
      await jobs.createJob(payload)
      message.success(t('messages.jobCreated'))
    } else if (jobId.value) {
      await jobs.updateJob(jobId.value, payload)
      message.success(t('messages.jobUpdated'))
    }
    clearDraft()
    await router.push(backToJobs.value)
  } catch (error) {
    const info = toApiErrorInfo(error, t)
    const mapped = resolveApiFieldErrors(info, {
      t,
      fieldMap: {
        name: 'name',
        schedule_timezone: 'scheduleTimezone',
        'spec.target.secret_name': 'webdavSecretName',
      },
    })
    for (const [field, messageText] of Object.entries(mapped)) {
      fieldErrors[field as JobEditorField] = messageText
    }
    const firstField = Object.keys(mapped)[0] as JobEditorField | undefined
    if (firstField) {
      step.value = routeStepForField(firstField)
      await focusField(firstField)
    }
    message.error(formatToastError(t('errors.saveJobFailed'), error, t))
  } finally {
    saving.value = false
  }
}

async function jumpToStepFromMobile(target: number): Promise<void> {
  await goToStep(target)
  mobileStepsExpanded.value = false
}

watch(
  form,
  () => {
    if (loading.value) return
    persistDraft()
  },
  { deep: true },
)

watch(step, () => {
  if (!isDesktop.value) {
    mobileStepsExpanded.value = false
  }
  if (loading.value) return
  persistDraft()
})

watch(isDesktop, (desktop) => {
  if (desktop) {
    mobileStepsExpanded.value = false
    mobileSummaryExpanded.value = false
    mobileRisksExpanded.value = false
  }
})

watch(
  () => route.fullPath,
  () => {
    void loadEditor()
  },
  { immediate: true },
)
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      :title="mode === 'create' ? t('jobs.editor.createPageTitle') : t('jobs.editor.editPageTitle')"
      :subtitle="mode === 'create' ? t('jobs.editor.createPageSubtitle') : t('jobs.editor.editPageSubtitle')"
    >
      <n-button quaternary @click="void router.push(backToJobs)">
        {{ t('jobs.editor.backToJobs') }}
      </n-button>
      <n-button v-if="!loading" quaternary @click="discardPendingDraft">
        {{ t('jobs.editor.discardDraft') }}
      </n-button>
    </PageHeader>

    <n-alert
      v-if="pendingDraft"
      :type="pendingDraft.baseJobUpdatedAt !== baseJobUpdatedAt ? 'warning' : 'info'"
      :bordered="false"
    >
      <div class="space-y-3">
        <div>
          {{
            pendingDraft.baseJobUpdatedAt !== baseJobUpdatedAt
              ? t('jobs.editor.draftStaleNotice')
              : t('jobs.editor.draftResumeNotice')
          }}
        </div>
        <div class="flex flex-wrap gap-2">
          <n-button size="small" type="primary" @click="resumePendingDraft">
            {{ t('jobs.editor.resumeDraft') }}
          </n-button>
          <n-button size="small" @click="discardPendingDraft">
            {{ t('jobs.editor.discardDraft') }}
          </n-button>
          <n-button size="small" tertiary @click="pendingDraft = null">
            {{ t('jobs.editor.keepLive') }}
          </n-button>
        </div>
      </div>
    </n-alert>

    <div class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_320px]">
      <n-card class="app-card" :bordered="false">
        <div ref="pageBody" class="space-y-5">
          <div v-if="loading" class="py-10 text-center app-text-muted">
            {{ t('common.loading') }}
          </div>

          <template v-else>
            <n-steps v-if="isDesktop" :current="step" size="small" @update:current="(value) => void goToStep(value)">
              <n-step :title="t('jobs.steps.basics')" />
              <n-step :title="t('jobs.steps.source')" />
              <n-step :title="t('jobs.steps.target')" />
              <n-step :title="t('jobs.steps.scheduleRetention')" />
              <n-step :title="t('jobs.steps.security')" />
              <n-step :title="t('jobs.steps.notifications')" />
              <n-step :title="t('jobs.steps.review')" />
            </n-steps>

            <div v-else class="space-y-3" data-testid="job-editor-mobile-progress">
              <button
                type="button"
                class="w-full rounded-2xl border border-[color:var(--app-border)] bg-[color:var(--app-surface-muted)] px-4 py-3 text-left"
                data-testid="job-editor-mobile-progress-toggle"
                @click="mobileStepsExpanded = !mobileStepsExpanded"
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <div class="text-xs uppercase tracking-[0.14em] app-text-muted">
                      {{ t('jobs.editor.progressLabel') }}
                    </div>
                    <div class="mt-1 text-base font-semibold">
                      {{ currentEditorStep.label }}
                    </div>
                  </div>
                  <n-tag size="small" :bordered="false">
                    {{ t('common.stepOf', { current: step, total: ROUTE_EDITOR_TOTAL_STEPS }) }}
                  </n-tag>
                </div>

                <div class="mt-3 h-2 overflow-hidden rounded-full bg-[var(--app-border)]">
                  <div
                    class="h-full rounded-full bg-[var(--app-primary)]"
                    :style="{ width: `${stepProgressPercent}%` }"
                  />
                </div>
              </button>

              <div class="flex items-center justify-between gap-3">
                <div class="text-sm app-text-muted">
                  {{ currentEditorStep.help }}
                </div>
                <n-button
                  size="small"
                  quaternary
                  data-testid="job-editor-mobile-step-toggle"
                  @click="mobileStepsExpanded = !mobileStepsExpanded"
                >
                  {{ mobileStepsExpanded ? t('jobs.editor.hideSteps') : t('jobs.editor.changeStep') }}
                </n-button>
              </div>

              <div v-if="mobileStepsExpanded" class="grid gap-2" data-testid="job-editor-mobile-step-list">
                <n-button
                  v-for="item in editorSteps"
                  :key="item.index"
                  size="small"
                  :type="item.index === step ? 'primary' : 'default'"
                  :secondary="item.index !== step"
                  @click="void jumpToStepFromMobile(item.index)"
                >
                  {{ t('common.stepOf', { current: item.index, total: ROUTE_EDITOR_TOTAL_STEPS }) }} · {{ item.label }}
                </n-button>
              </div>
            </div>

            <n-form label-placement="top">
              <JobEditorStepBasicsOnly
                v-if="step === 1"
                :node-options="[
                  { label: t('jobs.nodes.hub'), value: 'hub' },
                  ...agents.items.map((item) => ({
                    label: item.name ? `${item.name} (${item.id.slice(0, 8)}…)` : item.id,
                    value: item.id,
                  })),
                ]"
                :job-type-options="[
                  { label: t('jobs.types.filesystem'), value: 'filesystem' },
                  { label: t('jobs.types.sqlite'), value: 'sqlite' },
                  { label: t('jobs.types.vaultwarden'), value: 'vaultwarden' },
                ]"
                :overlap-options="[
                  { label: t('jobs.overlap.queue'), value: 'queue' },
                  { label: t('jobs.overlap.reject'), value: 'reject' },
                ]"
              />

              <JobEditorStepSource
                v-else-if="step === 2"
                :fs-symlink-policy-options="[
                  { label: t('jobs.fs.symlink.keep'), value: 'keep' },
                  { label: t('jobs.fs.symlink.follow'), value: 'follow' },
                  { label: t('jobs.fs.symlink.skip'), value: 'skip' },
                ]"
                :fs-hardlink-policy-options="[
                  { label: t('jobs.fs.hardlink.copy'), value: 'copy' },
                  { label: t('jobs.fs.hardlink.keep'), value: 'keep' },
                ]"
                :fs-error-policy-options="[
                  { label: t('jobs.fs.error.failFast'), value: 'fail_fast' },
                  { label: t('jobs.fs.error.skipFail'), value: 'skip_fail' },
                  { label: t('jobs.fs.error.skipOk'), value: 'skip_ok' },
                ]"
              />

              <JobEditorStepTarget
                v-else-if="step === 3"
                :target-type-options="[
                  { label: t('jobs.targets.webdav'), value: 'webdav' },
                  { label: t('jobs.targets.localDir'), value: 'local_dir' },
                ]"
                :webdav-secret-options="secrets.webdav.map((item) => ({ label: item.name, value: item.name }))"
              />

              <JobEditorStepScheduleRetention v-else-if="step === 4" />

              <JobEditorStepSecurity v-else-if="step === 5" />

              <JobEditorStepNotifications
                v-else-if="step === 6"
                :notify-mode-options="[
                  { label: t('jobs.notifications.inherit'), value: 'inherit' },
                  { label: t('jobs.notifications.custom'), value: 'custom' },
                ]"
                :wecom-destination-options="notifications.destinations.filter((item) => item.channel === 'wecom_bot').map((item) => ({ label: item.enabled ? item.name : `${item.name} (${t('settings.notifications.destinationDisabled')})`, value: item.name }))"
                :email-destination-options="notifications.destinations.filter((item) => item.channel === 'email').map((item) => ({ label: item.enabled ? item.name : `${item.name} (${t('settings.notifications.destinationDisabled')})`, value: item.name }))"
                :disabled-wecom-selected="form.notifyWecomBots.filter((name) => notifications.destinations.find((item) => item.channel === 'wecom_bot' && item.name === name)?.enabled === false)"
                :disabled-email-selected="form.notifyEmails.filter((name) => notifications.destinations.find((item) => item.channel === 'email' && item.name === name)?.enabled === false)"
              />

              <JobEditorStepReview
                v-else
                :node-label="form.node === 'hub' ? t('jobs.nodes.hub') : form.node"
                :overlap-label="form.overlapPolicy === 'queue' ? t('jobs.overlap.queue') : t('jobs.overlap.reject')"
                :job-type-label="t(`jobs.types.${form.jobType}`)"
                :target-type-label="form.targetType === 'webdav' ? t('jobs.targets.webdav') : t('jobs.targets.localDir')"
                :notify-mode-label="form.notifyMode === 'inherit' ? t('jobs.notifications.inherit') : t('jobs.notifications.custom')"
                :fs-symlink-policy-label="t(`jobs.fs.symlink.${form.fsSymlinkPolicy}`)"
                :fs-hardlink-policy-label="t(`jobs.fs.hardlink.${form.fsHardlinkPolicy}`)"
                :fs-error-policy-label="t(`jobs.fs.error.${form.fsErrorPolicy === 'fail_fast' ? 'failFast' : form.fsErrorPolicy === 'skip_fail' ? 'skipFail' : 'skipOk'}`)"
                :disabled-wecom-selected="form.notifyWecomBots.filter((name) => notifications.destinations.find((item) => item.channel === 'wecom_bot' && item.name === name)?.enabled === false)"
                :disabled-email-selected="form.notifyEmails.filter((name) => notifications.destinations.find((item) => item.channel === 'email' && item.name === name)?.enabled === false)"
              />
            </n-form>

            <div class="space-y-3 border-t border-[color:var(--app-border)] pt-4">
              <n-alert
                v-if="!isDesktop && mobileActionWarnings.length"
                :type="mobileActionWarnings.some((risk) => risk.type === 'error') ? 'error' : 'warning'"
                :bordered="false"
                data-testid="job-editor-mobile-action-warning"
              >
                <div class="space-y-2">
                  <div class="font-medium">{{ t('jobs.editor.actionWarningTitle') }}</div>
                  <div class="flex flex-wrap gap-2">
                    <n-tag
                      v-for="risk in mobileActionWarnings"
                      :key="risk.key"
                      size="small"
                      :bordered="false"
                      :type="risk.type === 'error' ? 'error' : 'warning'"
                    >
                      {{ t(`jobs.editor.riskLabels.${risk.key}`) }}
                    </n-tag>
                  </div>
                </div>
              </n-alert>

              <div class="flex flex-wrap items-center justify-between gap-2">
                <div class="text-sm app-text-muted">{{ t('common.stepOf', { current: step, total: ROUTE_EDITOR_TOTAL_STEPS }) }}</div>
                <div class="flex flex-wrap items-center gap-2">
                  <n-button @click="void router.push(backToJobs)">
                    {{ t('common.cancel') }}
                  </n-button>
                  <n-button v-if="step > 1" @click="prevStep">
                    {{ t('common.back') }}
                  </n-button>
                  <n-button v-if="step < ROUTE_EDITOR_TOTAL_STEPS" type="primary" :disabled="saving" @click="void nextStep()">
                    {{ t('common.next') }}
                  </n-button>
                  <n-button v-else type="primary" :loading="saving" @click="void save()">
                    {{ t('common.save') }}
                  </n-button>
                </div>
              </div>
            </div>
          </template>
        </div>
      </n-card>

      <div class="space-y-4">
        <n-card class="app-card" :bordered="false">
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="text-sm font-medium">{{ t('jobs.editor.summaryTitle') }}</div>
              <div v-if="!isDesktop && summaryPeekText" class="mt-1 text-xs app-text-muted">
                {{ summaryPeekText }}
              </div>
            </div>
            <n-button
              v-if="!isDesktop"
              size="small"
              quaternary
              data-testid="job-editor-toggle-summary"
              @click="mobileSummaryExpanded = !mobileSummaryExpanded"
            >
              {{ mobileSummaryExpanded ? t('jobs.editor.hideSummary') : t('jobs.editor.showSummary') }}
            </n-button>
          </div>
          <div v-if="isDesktop || mobileSummaryExpanded" class="mt-3 space-y-2 text-sm" data-testid="job-editor-summary-body">
            <div v-for="row in summaryRows" :key="row.label" class="flex items-start justify-between gap-3">
              <span class="app-text-muted">{{ row.label }}</span>
              <span class="text-right break-all">{{ row.value || '-' }}</span>
            </div>
          </div>
        </n-card>

        <n-card class="app-card" :bordered="false">
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="text-sm font-medium">{{ t('jobs.editor.risksTitle') }}</div>
              <div v-if="!isDesktop" class="mt-1 text-xs app-text-muted">
                {{ riskPeekText }}
              </div>
            </div>
            <n-button
              v-if="!isDesktop"
              size="small"
              quaternary
              data-testid="job-editor-toggle-risks"
              @click="mobileRisksExpanded = !mobileRisksExpanded"
            >
              {{ mobileRisksExpanded ? t('jobs.editor.hideRisks') : t('jobs.editor.showRisks') }}
            </n-button>
          </div>
          <div v-if="isDesktop || mobileRisksExpanded" class="mt-3 flex flex-wrap gap-2" data-testid="job-editor-risks-body">
            <n-tag
              v-for="risk in riskRows"
              :key="risk.key"
              size="small"
              :bordered="false"
              :type="risk.type === 'error' ? 'error' : risk.type === 'warning' ? 'warning' : 'info'"
            >
              {{ t(`jobs.editor.riskLabels.${risk.key}`) }}
            </n-tag>
          </div>
        </n-card>
      </div>
    </div>

    <FsPathPickerModal ref="fsPicker" @picked="onFsPickerPicked" />
  </div>
</template>
