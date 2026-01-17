<script setup lang="ts">
import { computed, nextTick, provide, reactive, ref } from 'vue'
import {
  NButton,
  NForm,
  NModal,
  NStep,
  NSteps,
  useMessage,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore } from '@/stores/jobs'
import { useAgentsStore } from '@/stores/agents'
import { useSecretsStore } from '@/stores/secrets'
import { useNotificationsStore } from '@/stores/notifications'
import { useSystemStore } from '@/stores/system'
import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatToastError } from '@/lib/errors'
import FsPathPickerModal, { type FsPathPickerModalExpose } from '@/components/fs/FsPathPickerModal.vue'

import { jobEditorContextKey } from './editor/context'
import { createInitialJobEditorFieldErrors, createInitialJobEditorForm, resetJobEditorForm } from './editor/form'
import { editorFormToRequest, jobDetailToEditorForm } from './editor/mapping'
import type { JobEditorField, JobEditorForm } from './editor/types'
import { stepForJobEditorField, validateJobEditorUpToStep } from './editor/validation'
import JobEditorStepBasics from './editor/steps/JobEditorStepBasics.vue'
import JobEditorStepNotifications from './editor/steps/JobEditorStepNotifications.vue'
import JobEditorStepReview from './editor/steps/JobEditorStepReview.vue'
import JobEditorStepSecurity from './editor/steps/JobEditorStepSecurity.vue'
import JobEditorStepSource from './editor/steps/JobEditorStepSource.vue'
import JobEditorStepTarget from './editor/steps/JobEditorStepTarget.vue'

export type JobEditorModalExpose = {
  openCreate: (ctx?: { nodeId?: 'hub' | string }) => void
  openEdit: (jobId: string, ctx?: { nodeId?: 'hub' | string }) => Promise<void>
}

const emit = defineEmits<{
  (e: 'saved'): void
}>()

const { t } = useI18n()
const message = useMessage()

const jobs = useJobsStore()
const agents = useAgentsStore()
const secrets = useSecretsStore()
const notifications = useNotificationsStore()
const system = useSystemStore()

const isDesktop = useMediaQuery(MQ.mdUp)

const modalStyle = computed(() =>
  isDesktop.value
    ? { width: MODAL_WIDTH.lg, height: MODAL_HEIGHT.desktopLoose }
    : { width: MODAL_WIDTH.lg, maxHeight: MODAL_HEIGHT.max },
)

const show = ref<boolean>(false)
const modalBody = ref<HTMLElement | null>(null)
const mode = ref<'create' | 'edit'>('create')
const saving = ref<boolean>(false)
const step = ref<number>(1)
const lockedNodeId = ref<'hub' | string | null>(null)
const fsPicker = ref<FsPathPickerModalExpose | null>(null)
const fsPickerPurpose = ref<'source_paths' | 'local_base_dir'>('source_paths')
const fsPathDraft = ref<string>('')
const showJsonPreview = ref<boolean>(false)

const fieldErrors = reactive<Record<JobEditorField, string | null>>(createInitialJobEditorFieldErrors())

function clearFieldError(field: JobEditorField): void {
  fieldErrors[field] = null
}

function clearAllFieldErrors(): void {
  for (const key of Object.keys(fieldErrors) as JobEditorField[]) {
    fieldErrors[key] = null
  }
}

const EDITOR_STEPS_TOTAL = 6
const stepTitles = computed(() => [
  t('jobs.steps.basics'),
  t('jobs.steps.source'),
  t('jobs.steps.target'),
  t('jobs.steps.security'),
  t('jobs.steps.notifications'),
  t('jobs.steps.review'),
])
const stepTitle = computed(() => {
  const idx = Math.min(EDITOR_STEPS_TOTAL - 1, Math.max(0, step.value - 1))
  return stepTitles.value[idx]
})
const stepPercent = computed(() =>
  Math.round((Math.min(EDITOR_STEPS_TOTAL, Math.max(1, step.value)) / EDITOR_STEPS_TOTAL) * 100),
)

const form = reactive<JobEditorForm>(createInitialJobEditorForm())

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
    const v = raw.trim()
    if (!v) continue
    if (existing.has(v)) {
      skipped += 1
      continue
    }
    existing.add(v)
    out.push(v)
    added += 1
  }
  return { merged: out, added, skipped }
}

function openCreateWithContext(ctx?: { nodeId?: 'hub' | string }): void {
  void (async () => {
    mode.value = 'create'
    step.value = 1
    resetForm()
    showJsonPreview.value = false
    lockedNodeId.value = ctx?.nodeId ?? null
    if (lockedNodeId.value) {
      form.node = lockedNodeId.value
    }
    try {
      await system.refresh()
    } catch {
      // ignore
    }
    form.scheduleTimezone = system.hubTimezone || 'UTC'
    void notifications.refreshDestinations()
    show.value = true
  })()
}

async function openEdit(jobId: string, ctx?: { nodeId?: 'hub' | string }): Promise<void> {
  mode.value = 'edit'
  step.value = 1
  show.value = true
  saving.value = true
  clearAllFieldErrors()
  showJsonPreview.value = false
  lockedNodeId.value = ctx?.nodeId ?? null
  void notifications.refreshDestinations()
  try {
    const job = await jobs.getJob(jobId)
    Object.assign(form, jobDetailToEditorForm(job))
    if (lockedNodeId.value) {
      form.node = lockedNodeId.value
    }
  } catch (error) {
    message.error(formatToastError(t('errors.fetchJobFailed'), error, t))
    show.value = false
  } finally {
    saving.value = false
  }
}

async function focusField(field: JobEditorField): Promise<void> {
  await nextTick()
  const root = modalBody.value
  if (!root) return

  const el = root.querySelector(`[data-field="${field}"]`) as HTMLElement | null
  if (!el) return

  el.scrollIntoView({ block: 'center' })

  const focusable = el.querySelector(
    'input, textarea, [contenteditable="true"], [role="combobox"], button, [tabindex]:not([tabindex="-1"])',
  ) as HTMLElement | null
  focusable?.focus?.()
}

async function validateEditorUpTo(targetStep: number): Promise<boolean> {
  clearAllFieldErrors()

  const issues = validateJobEditorUpToStep(targetStep, form, t)
  if (issues.length === 0) return true

  for (const issue of issues) {
    fieldErrors[issue.field] = issue.message
  }

  message.error(t('errors.formInvalid'))

  const first = issues[0]!
  const fieldStep = stepForJobEditorField(first.field)
  if (step.value !== fieldStep) {
    step.value = fieldStep
    await nextTick()
  }
  await focusField(first.field)
  return false
}

async function goToStep(targetStep: number): Promise<void> {
  const clamped = Math.min(EDITOR_STEPS_TOTAL, Math.max(1, Math.floor(targetStep)))
  if (clamped <= step.value) {
    step.value = clamped
    return
  }
  const ok = await validateEditorUpTo(clamped - 1)
  if (!ok) return
  step.value = clamped
}

function onJobTypeChanged(): void {
  clearFieldError('fsPaths')
  clearFieldError('sqlitePath')
  clearFieldError('vaultwardenDataDir')
}

function onTargetTypeChanged(): void {
  clearFieldError('webdavBaseUrl')
  clearFieldError('webdavSecretName')
  clearFieldError('localBaseDir')
}

function onEncryptionEnabledChanged(): void {
  clearFieldError('encryptionKeyName')
}

function prevStep(): void {
  step.value = Math.max(1, step.value - 1)
}

async function nextStep(): Promise<void> {
  const ok = await validateEditorUpTo(step.value)
  if (!ok) return
  step.value = Math.min(EDITOR_STEPS_TOTAL, step.value + 1)
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
  if (added > 0) {
    message.success(t('messages.sourcePathsAdded', { count: added }))
  }
  if (skipped > 0) {
    message.warning(t('messages.sourcePathsSkipped', { count: skipped }))
  }
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
  form.fsPaths = form.fsPaths.filter((p) => p !== path)
}

function clearFsPaths(): void {
  form.fsPaths = []
}

const previewPayload = computed(() => {
  return editorFormToRequest(form)
})

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
  onEncryptionEnabledChanged,
  openFsPicker,
  openLocalBaseDirPicker,
  addFsPathsFromDraft,
  removeFsPath,
  clearFsPaths,
})

function getOptionLabel<T extends string>(options: ReadonlyArray<{ label: string; value: T }>, value: T): string {
  const found = options.find((o) => o.value === value)
  return found?.label ?? value
}

const nodeLabel = computed(() => getOptionLabel(nodeOptions.value, form.node))
const overlapLabel = computed(() => getOptionLabel(overlapOptions.value, form.overlapPolicy))
const jobTypeLabel = computed(() => getOptionLabel(jobTypeOptions.value, form.jobType))
const targetTypeLabel = computed(() => getOptionLabel(targetTypeOptions.value, form.targetType))
const notifyModeLabel = computed(() => getOptionLabel(notifyModeOptions.value, form.notifyMode))
const fsSymlinkPolicyLabel = computed(() => getOptionLabel(fsSymlinkPolicyOptions.value, form.fsSymlinkPolicy))
const fsHardlinkPolicyLabel = computed(() => getOptionLabel(fsHardlinkPolicyOptions.value, form.fsHardlinkPolicy))
const fsErrorPolicyLabel = computed(() => getOptionLabel(fsErrorPolicyOptions.value, form.fsErrorPolicy))

async function save(): Promise<void> {
  const ok = await validateEditorUpTo(5)
  if (!ok) return

  saving.value = true
  try {
    const payload = previewPayload.value

    if (mode.value === 'create') {
      await jobs.createJob(payload)
      message.success(t('messages.jobCreated'))
    } else if (form.id) {
      await jobs.updateJob(form.id, payload)
      message.success(t('messages.jobUpdated'))
    }

    show.value = false
    emit('saved')
  } catch (error) {
    message.error(formatToastError(t('errors.saveJobFailed'), error, t))
  } finally {
    saving.value = false
  }
}

const overlapOptions = computed(() => [
  { label: t('jobs.overlap.queue'), value: 'queue' },
  { label: t('jobs.overlap.reject'), value: 'reject' },
])

const nodeOptions = computed(() => [
  { label: t('jobs.nodes.hub'), value: 'hub' },
  ...agents.items.map((a) => ({
    label: a.name ? `${a.name} (${a.id.slice(0, 8)}…)` : a.id,
    value: a.id,
  })),
])

const targetTypeOptions = computed(() => [
  { label: t('jobs.targets.webdav'), value: 'webdav' },
  { label: t('jobs.targets.localDir'), value: 'local_dir' },
])

const jobTypeOptions = computed(() => [
  { label: t('jobs.types.filesystem'), value: 'filesystem' },
  { label: t('jobs.types.sqlite'), value: 'sqlite' },
  { label: t('jobs.types.vaultwarden'), value: 'vaultwarden' },
])

const fsSymlinkPolicyOptions = computed(() => [
  { label: t('jobs.fs.symlink.keep'), value: 'keep' },
  { label: t('jobs.fs.symlink.follow'), value: 'follow' },
  { label: t('jobs.fs.symlink.skip'), value: 'skip' },
])

const fsHardlinkPolicyOptions = computed(() => [
  { label: t('jobs.fs.hardlink.copy'), value: 'copy' },
  { label: t('jobs.fs.hardlink.keep'), value: 'keep' },
])

const fsErrorPolicyOptions = computed(() => [
  { label: t('jobs.fs.error.failFast'), value: 'fail_fast' },
  { label: t('jobs.fs.error.skipFail'), value: 'skip_fail' },
  { label: t('jobs.fs.error.skipOk'), value: 'skip_ok' },
])

const webdavSecretOptions = computed(() => secrets.webdav.map((s) => ({ label: s.name, value: s.name })))

const notifyModeOptions = computed(() => [
  { label: t('jobs.notifications.inherit'), value: 'inherit' as const },
  { label: t('jobs.notifications.custom'), value: 'custom' as const },
])

const wecomDestinationOptions = computed(() =>
  notifications.destinations
    .filter((d) => d.channel === 'wecom_bot')
    .map((d) => ({
      label: d.enabled ? d.name : `${d.name} (${t('settings.notifications.destinationDisabled')})`,
      value: d.name,
    })),
)

const emailDestinationOptions = computed(() =>
  notifications.destinations
    .filter((d) => d.channel === 'email')
    .map((d) => ({
      label: d.enabled ? d.name : `${d.name} (${t('settings.notifications.destinationDisabled')})`,
      value: d.name,
    })),
)

const disabledWecomSelected = computed(() => {
  const enabled = new Map(
    notifications.destinations
      .filter((d) => d.channel === 'wecom_bot')
      .map((d) => [d.name, d.enabled] as const),
  )
  return form.notifyWecomBots.filter((name) => enabled.get(name) === false)
})

const disabledEmailSelected = computed(() => {
  const enabled = new Map(
    notifications.destinations
      .filter((d) => d.channel === 'email')
      .map((d) => [d.name, d.enabled] as const),
  )
  return form.notifyEmails.filter((name) => enabled.get(name) === false)
})

defineExpose<JobEditorModalExpose>({ openCreate: openCreateWithContext, openEdit })
</script>

<template>
  <n-modal
    v-model:show="show"
    preset="card"
    :style="modalStyle"
    :content-style="{ overflow: 'auto', minHeight: 0 }"
    :title="mode === 'create' ? t('jobs.createTitle') : t('jobs.editTitle')"
  >
    <div ref="modalBody" class="space-y-4">
      <div v-if="isDesktop">
        <n-steps :current="step" size="small" @update:current="(v) => void goToStep(v)">
          <n-step :title="t('jobs.steps.basics')" />
          <n-step :title="t('jobs.steps.source')" />
          <n-step :title="t('jobs.steps.target')" />
          <n-step :title="t('jobs.steps.security')" />
          <n-step :title="t('jobs.steps.notifications')" />
          <n-step :title="t('jobs.steps.review')" />
        </n-steps>
      </div>
      <div v-else class="space-y-2">
        <div class="flex items-center justify-between gap-3">
          <div class="text-sm font-medium">{{ stepTitle }}</div>
          <div class="text-xs opacity-70">
            {{ t('common.stepOf', { current: step, total: EDITOR_STEPS_TOTAL }) }}
          </div>
        </div>
        <div class="h-1.5 rounded bg-black/5 dark:bg-white/10 overflow-hidden">
          <div class="h-full bg-[var(--n-primary-color)]" :style="{ width: `${stepPercent}%` }" />
        </div>
      </div>

      <n-form label-placement="top">
        <JobEditorStepBasics
          v-if="step === 1"
          :node-options="nodeOptions"
          :job-type-options="jobTypeOptions"
          :overlap-options="overlapOptions"
        />

        <JobEditorStepSource
          v-else-if="step === 2"
          :fs-symlink-policy-options="fsSymlinkPolicyOptions"
          :fs-hardlink-policy-options="fsHardlinkPolicyOptions"
          :fs-error-policy-options="fsErrorPolicyOptions"
        />

        <JobEditorStepTarget
          v-else-if="step === 3"
          :target-type-options="targetTypeOptions"
          :webdav-secret-options="webdavSecretOptions"
        />

        <JobEditorStepSecurity v-else-if="step === 4" />

        <JobEditorStepNotifications
          v-else-if="step === 5"
          :notify-mode-options="notifyModeOptions"
          :wecom-destination-options="wecomDestinationOptions"
          :email-destination-options="emailDestinationOptions"
          :disabled-wecom-selected="disabledWecomSelected"
          :disabled-email-selected="disabledEmailSelected"
        />

        <JobEditorStepReview
          v-else
          :node-label="nodeLabel"
          :overlap-label="overlapLabel"
          :job-type-label="jobTypeLabel"
          :target-type-label="targetTypeLabel"
          :notify-mode-label="notifyModeLabel"
          :fs-symlink-policy-label="fsSymlinkPolicyLabel"
          :fs-hardlink-policy-label="fsHardlinkPolicyLabel"
          :fs-error-policy-label="fsErrorPolicyLabel"
          :disabled-wecom-selected="disabledWecomSelected"
          :disabled-email-selected="disabledEmailSelected"
        />
      </n-form>
    </div>

    <template #footer>
      <div class="flex flex-wrap items-center justify-between gap-2">
        <n-button :disabled="saving" @click="show = false">{{ t('common.cancel') }}</n-button>
        <div class="flex items-center gap-2">
          <n-button v-if="step > 1" :disabled="saving" @click="prevStep">{{ t('common.back') }}</n-button>
          <n-button v-if="step < EDITOR_STEPS_TOTAL" type="primary" :disabled="saving" @click="nextStep">
            {{ t('common.next') }}
          </n-button>
          <n-button v-else type="primary" :loading="saving" @click="save">
            {{ t('common.save') }}
          </n-button>
        </div>
      </div>
    </template>
  </n-modal>
  <FsPathPickerModal ref="fsPicker" @picked="onFsPickerPicked" />
</template>
