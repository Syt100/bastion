<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import {
  NAlert,
  NButton,
  NCode,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NSelect,
  NSpace,
  NStep,
  NSteps,
  NSwitch,
  NTag,
  useMessage,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type JobType, type OverlapPolicy } from '@/stores/jobs'
import { useAgentsStore } from '@/stores/agents'
import { useSecretsStore } from '@/stores/secrets'
import { useNotificationsStore } from '@/stores/notifications'
import { MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatToastError } from '@/lib/errors'
import FsPathPickerModal, { type FsPathPickerModalExpose } from '@/components/fs/FsPathPickerModal.vue'

type FsSymlinkPolicy = 'keep' | 'follow' | 'skip'
type FsHardlinkPolicy = 'copy' | 'keep'
type FsErrorPolicy = 'fail_fast' | 'skip_fail' | 'skip_ok'

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

const isDesktop = useMediaQuery(MQ.mdUp)

const show = ref<boolean>(false)
const mode = ref<'create' | 'edit'>('create')
const saving = ref<boolean>(false)
const step = ref<number>(1)
const lockedNodeId = ref<'hub' | string | null>(null)
const fsPicker = ref<FsPathPickerModalExpose | null>(null)
const fsPathDraft = ref<string>('')

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

const form = reactive<{
  id: string | null
  name: string
  node: 'hub' | string
  schedule: string
  overlapPolicy: OverlapPolicy
  jobType: JobType
  encryptionEnabled: boolean
  encryptionKeyName: string
  fsPaths: string[]
  fsInclude: string
  fsExclude: string
  fsSymlinkPolicy: FsSymlinkPolicy
  fsHardlinkPolicy: FsHardlinkPolicy
  fsErrorPolicy: FsErrorPolicy
  sqlitePath: string
  sqliteIntegrityCheck: boolean
  vaultwardenDataDir: string
  targetType: 'webdav' | 'local_dir'
  webdavBaseUrl: string
  webdavSecretName: string
  localBaseDir: string
  partSizeMiB: number
  notifyMode: 'inherit' | 'custom'
  notifyWecomBots: string[]
  notifyEmails: string[]
}>({
  id: null,
  name: '',
  node: 'hub',
  schedule: '',
  overlapPolicy: 'queue',
  jobType: 'filesystem',
  encryptionEnabled: false,
  encryptionKeyName: 'default',
  fsPaths: [],
  fsInclude: '',
  fsExclude: '',
  fsSymlinkPolicy: 'keep',
  fsHardlinkPolicy: 'copy',
  fsErrorPolicy: 'fail_fast',
  sqlitePath: '',
  sqliteIntegrityCheck: false,
  vaultwardenDataDir: '',
  targetType: 'webdav',
  webdavBaseUrl: '',
  webdavSecretName: '',
  localBaseDir: '',
  partSizeMiB: 256,
  notifyMode: 'inherit',
  notifyWecomBots: [],
  notifyEmails: [],
})

function resetForm(): void {
  form.id = null
  form.name = ''
  form.node = 'hub'
  form.schedule = ''
  form.overlapPolicy = 'queue'
  form.jobType = 'filesystem'
  form.encryptionEnabled = false
  form.encryptionKeyName = 'default'
  form.fsPaths = []
  form.fsInclude = ''
  form.fsExclude = ''
  form.fsSymlinkPolicy = 'keep'
  form.fsHardlinkPolicy = 'copy'
  form.fsErrorPolicy = 'fail_fast'
  form.sqlitePath = ''
  form.sqliteIntegrityCheck = false
  form.vaultwardenDataDir = ''
  form.targetType = 'webdav'
  form.webdavBaseUrl = ''
  form.webdavSecretName = ''
  form.localBaseDir = ''
  form.partSizeMiB = 256
  form.notifyMode = 'inherit'
  form.notifyWecomBots = []
  form.notifyEmails = []
}

function parseLines(text: string): string[] {
  return text
    .split(/\r?\n/g)
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
}

function parseStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value.filter((v): v is string => typeof v === 'string')
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

function normalizeSymlinkPolicy(value: unknown): FsSymlinkPolicy {
  if (value === 'follow') return 'follow'
  if (value === 'skip') return 'skip'
  return 'keep'
}

function normalizeHardlinkPolicy(value: unknown): FsHardlinkPolicy {
  if (value === 'keep') return 'keep'
  return 'copy'
}

function normalizeErrorPolicy(value: unknown): FsErrorPolicy {
  if (value === 'skip_fail') return 'skip_fail'
  if (value === 'skip_ok') return 'skip_ok'
  return 'fail_fast'
}

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function openCreateWithContext(ctx?: { nodeId?: 'hub' | string }): void {
  mode.value = 'create'
  step.value = 1
  resetForm()
  lockedNodeId.value = ctx?.nodeId ?? null
  if (lockedNodeId.value) {
    form.node = lockedNodeId.value
  }
  void notifications.refreshDestinations()
  show.value = true
}

async function openEdit(jobId: string, ctx?: { nodeId?: 'hub' | string }): Promise<void> {
  mode.value = 'edit'
  step.value = 1
  show.value = true
  saving.value = true
  lockedNodeId.value = ctx?.nodeId ?? null
  void notifications.refreshDestinations()
  try {
    const job = await jobs.getJob(jobId)
    form.id = job.id
    form.name = job.name
    form.node = job.agent_id ? job.agent_id : 'hub'
    if (lockedNodeId.value) {
      form.node = lockedNodeId.value
    }
    form.schedule = job.schedule ?? ''
    form.overlapPolicy = job.overlap_policy
    form.jobType = job.spec.type

    const pipeline = (job.spec as Record<string, unknown>).pipeline as Record<string, unknown> | undefined
    const enc = pipeline?.encryption as Record<string, unknown> | undefined
    const encType = typeof enc?.type === 'string' ? enc.type : 'none'
    if (encType === 'age_x25519') {
      form.encryptionEnabled = true
      form.encryptionKeyName = typeof enc?.key_name === 'string' ? enc.key_name : 'default'
    } else {
      form.encryptionEnabled = false
      form.encryptionKeyName = 'default'
    }

    const target = (job.spec as Record<string, unknown>).target as Record<string, unknown> | undefined
    const targetType = target?.type === 'local_dir' ? 'local_dir' : 'webdav'
    form.targetType = targetType
    form.webdavBaseUrl = typeof target?.base_url === 'string' ? target.base_url : ''
    form.webdavSecretName = typeof target?.secret_name === 'string' ? target.secret_name : ''
    form.localBaseDir = typeof target?.base_dir === 'string' ? target.base_dir : ''
    form.partSizeMiB =
      typeof target?.part_size_bytes === 'number' && target.part_size_bytes > 0
        ? Math.max(1, Math.round(target.part_size_bytes / (1024 * 1024)))
        : 256

    const source = (job.spec as Record<string, unknown>).source as Record<string, unknown> | undefined
    const paths = parseStringArray(source?.paths)
    if (paths.length > 0) {
      form.fsPaths = paths
    } else {
      const legacyRoot = typeof source?.root === 'string' ? source.root : ''
      form.fsPaths = legacyRoot.trim() ? [legacyRoot] : []
    }
    form.fsInclude = parseStringArray(source?.include).join('\n')
    form.fsExclude = parseStringArray(source?.exclude).join('\n')
    form.fsSymlinkPolicy = normalizeSymlinkPolicy(source?.symlink_policy)
    form.fsHardlinkPolicy = normalizeHardlinkPolicy(source?.hardlink_policy)
    form.fsErrorPolicy = normalizeErrorPolicy(source?.error_policy)
    form.sqlitePath = typeof source?.path === 'string' ? source.path : ''
    form.sqliteIntegrityCheck = typeof source?.integrity_check === 'boolean' ? source.integrity_check : false
    form.vaultwardenDataDir = typeof source?.data_dir === 'string' ? source.data_dir : ''

    const notif = (job.spec as Record<string, unknown>).notifications as Record<string, unknown> | undefined
    const mode = typeof notif?.mode === 'string' && notif.mode === 'custom' ? 'custom' : 'inherit'
    form.notifyMode = mode
    form.notifyWecomBots = parseStringArray(notif?.['wecom_bot'])
    form.notifyEmails = parseStringArray(notif?.['email'])
  } catch (error) {
    message.error(formatToastError(t('errors.fetchJobFailed'), error, t))
    show.value = false
  } finally {
    saving.value = false
  }
}

function validateEditorStep(targetStep: number): boolean {
  if (targetStep >= 1) {
    const name = form.name.trim()
    if (!name) {
      message.error(t('errors.jobNameRequired'))
      return false
    }
  }

  if (targetStep >= 2) {
    if (form.jobType === 'filesystem' && form.fsPaths.every((p) => !p.trim())) {
      message.error(t('errors.sourcePathsRequired'))
      return false
    }
    if (form.jobType === 'sqlite' && !form.sqlitePath.trim()) {
      message.error(t('errors.sqlitePathRequired'))
      return false
    }
    if (form.jobType === 'vaultwarden' && !form.vaultwardenDataDir.trim()) {
      message.error(t('errors.vaultwardenDataDirRequired'))
      return false
    }
  }

  if (targetStep >= 3) {
    if (form.targetType === 'webdav') {
      if (!form.webdavBaseUrl.trim()) {
        message.error(t('errors.webdavBaseUrlRequired'))
        return false
      }
      if (!form.webdavSecretName.trim()) {
        message.error(t('errors.webdavSecretRequired'))
        return false
      }
    } else {
      if (!form.localBaseDir.trim()) {
        message.error(t('errors.localBaseDirRequired'))
        return false
      }
    }

    if (!Number.isFinite(form.partSizeMiB) || form.partSizeMiB <= 0) {
      message.error(t('errors.partSizeInvalid'))
      return false
    }
  }

  if (targetStep >= 4) {
    const encryptionKeyName = form.encryptionKeyName.trim()
    if (form.encryptionEnabled && !encryptionKeyName) {
      message.error(t('errors.encryptionKeyNameRequired'))
      return false
    }
  }

  return true
}

function prevStep(): void {
  step.value = Math.max(1, step.value - 1)
}

function nextStep(): void {
  if (!validateEditorStep(step.value)) return
  step.value = Math.min(EDITOR_STEPS_TOTAL, step.value + 1)
}

function openFsPicker(): void {
  const initial = form.fsPaths.find((p) => p.trim()) ?? '/'
  fsPicker.value?.open(form.node, initial)
}

function addFsPathsFromList(paths: string[]): void {
  const { merged, added, skipped } = mergeUniqueStrings(form.fsPaths, paths)
  form.fsPaths = merged
  if (added > 0) {
    message.success(t('messages.sourcePathsAdded', { count: added }))
  }
  if (skipped > 0) {
    message.warning(t('messages.sourcePathsSkipped', { count: skipped }))
  }
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
  const partSizeMiB = Math.max(1, Math.floor(form.partSizeMiB || 1))
  const partSizeBytes = partSizeMiB * 1024 * 1024

  const pipeline = {
    encryption: form.encryptionEnabled
      ? ({ type: 'age_x25519' as const, key_name: form.encryptionKeyName.trim() || 'default' } as const)
      : ({ type: 'none' as const } as const),
  }

  const notifications =
    form.notifyMode === 'custom'
      ? {
          mode: 'custom' as const,
          wecom_bot: form.notifyWecomBots,
          email: form.notifyEmails,
        }
      : ({ mode: 'inherit' as const } as const)

  const source =
    form.jobType === 'filesystem'
      ? {
          paths: form.fsPaths.map((p) => p.trim()).filter((p) => p.length > 0),
          include: parseLines(form.fsInclude),
          exclude: parseLines(form.fsExclude),
          symlink_policy: form.fsSymlinkPolicy,
          hardlink_policy: form.fsHardlinkPolicy,
          error_policy: form.fsErrorPolicy,
        }
      : form.jobType === 'sqlite'
        ? { path: form.sqlitePath.trim(), integrity_check: form.sqliteIntegrityCheck }
        : { data_dir: form.vaultwardenDataDir.trim() }

  const target =
    form.targetType === 'webdav'
      ? ({
          type: 'webdav' as const,
          base_url: form.webdavBaseUrl.trim(),
          secret_name: form.webdavSecretName.trim(),
          part_size_bytes: partSizeBytes,
        } as const)
      : ({
          type: 'local_dir' as const,
          base_dir: form.localBaseDir.trim(),
          part_size_bytes: partSizeBytes,
        } as const)

  return {
    name: form.name.trim(),
    agent_id: form.node === 'hub' ? null : form.node,
    schedule: form.schedule.trim() ? form.schedule.trim() : null,
    overlap_policy: form.overlapPolicy,
    spec: {
      v: 1 as const,
      type: form.jobType,
      pipeline,
      notifications,
      source,
      target,
    },
  }
})

async function save(): Promise<void> {
  if (!validateEditorStep(5)) return

  const name = form.name.trim()
  if (!name) {
    message.error(t('errors.jobNameRequired'))
    return
  }

  const agentId = form.node === 'hub' ? null : form.node

  const partSizeMiB = Math.max(1, Math.floor(form.partSizeMiB))
  const partSizeBytes = partSizeMiB * 1024 * 1024

  const targetType = form.targetType
  const webdavBaseUrl = form.webdavBaseUrl.trim()
  const webdavSecretName = form.webdavSecretName.trim()
  const localBaseDir = form.localBaseDir.trim()

  if (targetType === 'webdav') {
    if (!webdavBaseUrl) {
      message.error(t('errors.webdavBaseUrlRequired'))
      return
    }
    if (!webdavSecretName) {
      message.error(t('errors.webdavSecretRequired'))
      return
    }
  } else {
    if (!localBaseDir) {
      message.error(t('errors.localBaseDirRequired'))
      return
    }
  }

  const encryptionKeyName = form.encryptionKeyName.trim()
  if (form.encryptionEnabled && !encryptionKeyName) {
    message.error(t('errors.encryptionKeyNameRequired'))
    return
  }

  const pipeline = {
    encryption: form.encryptionEnabled
      ? ({ type: 'age_x25519' as const, key_name: encryptionKeyName } as const)
      : ({ type: 'none' as const } as const),
  }

  const notifications =
    form.notifyMode === 'custom'
      ? {
          mode: 'custom' as const,
          wecom_bot: form.notifyWecomBots,
          email: form.notifyEmails,
        }
      : ({ mode: 'inherit' as const } as const)

  type FsSource = {
    paths: string[]
    include: string[]
    exclude: string[]
    symlink_policy: FsSymlinkPolicy
    hardlink_policy: FsHardlinkPolicy
    error_policy: FsErrorPolicy
  }
  type SqliteSource = { path: string; integrity_check: boolean }
  type VaultwardenSource = { data_dir: string }

  let source: FsSource | SqliteSource | VaultwardenSource
  if (form.jobType === 'filesystem') {
    const fsSource: FsSource = {
      paths: form.fsPaths.map((p) => p.trim()).filter((p) => p.length > 0),
      include: parseLines(form.fsInclude),
      exclude: parseLines(form.fsExclude),
      symlink_policy: form.fsSymlinkPolicy,
      hardlink_policy: form.fsHardlinkPolicy,
      error_policy: form.fsErrorPolicy,
    }
    if (fsSource.paths.length === 0) {
      message.error(t('errors.sourcePathsRequired'))
      return
    }
    source = fsSource
  } else if (form.jobType === 'sqlite') {
    const sqliteSource: SqliteSource = { path: form.sqlitePath.trim(), integrity_check: form.sqliteIntegrityCheck }
    if (!sqliteSource.path) {
      message.error(t('errors.sqlitePathRequired'))
      return
    }
    source = sqliteSource
  } else {
    const vaultwardenSource: VaultwardenSource = { data_dir: form.vaultwardenDataDir.trim() }
    if (!vaultwardenSource.data_dir) {
      message.error(t('errors.vaultwardenDataDirRequired'))
      return
    }
    source = vaultwardenSource
  }

  saving.value = true
  try {
    const target =
      targetType === 'webdav'
        ? ({
            type: 'webdav' as const,
            base_url: webdavBaseUrl,
            secret_name: webdavSecretName,
            part_size_bytes: partSizeBytes,
          } as const)
        : ({
            type: 'local_dir' as const,
            base_dir: localBaseDir,
            part_size_bytes: partSizeBytes,
          } as const)

    const payload = {
      name,
      agent_id: agentId,
      schedule: form.schedule.trim() ? form.schedule.trim() : null,
      overlap_policy: form.overlapPolicy,
      spec: {
        v: 1 as const,
        type: form.jobType,
        pipeline,
        notifications,
        source,
        target,
      },
    }

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
    :style="{ width: MODAL_WIDTH.lg }"
    :title="mode === 'create' ? t('jobs.createTitle') : t('jobs.editTitle')"
  >
    <div class="space-y-4">
      <div v-if="isDesktop">
        <n-steps :current="step" size="small">
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
        <template v-if="step === 1">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
            <n-form-item :label="t('jobs.fields.name')">
              <n-input v-model:value="form.name" />
            </n-form-item>
            <n-form-item :label="t('jobs.fields.node')">
              <n-select v-model:value="form.node" :options="nodeOptions" filterable :disabled="lockedNodeId !== null" />
            </n-form-item>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
            <n-form-item :label="t('jobs.fields.type')">
              <n-select v-model:value="form.jobType" :options="jobTypeOptions" />
            </n-form-item>
            <n-form-item :label="t('jobs.fields.overlap')">
              <n-select v-model:value="form.overlapPolicy" :options="overlapOptions" />
            </n-form-item>
          </div>

          <n-form-item :label="t('jobs.fields.schedule')">
            <n-input v-model:value="form.schedule" :placeholder="t('jobs.fields.schedulePlaceholder')" />
            <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.scheduleHelp') }}</div>
          </n-form-item>
        </template>

        <template v-else-if="step === 2">
          <n-alert type="info" :bordered="false">
            {{ t('jobs.steps.sourceHelp') }}
          </n-alert>

          <template v-if="form.jobType === 'filesystem'">
            <n-form-item :label="t('jobs.fields.sourcePaths')">
              <div class="space-y-2 w-full">
                <div class="flex flex-wrap items-center gap-2 justify-between">
                  <div class="text-xs opacity-70">{{ t('jobs.fields.sourcePathsHelp') }}</div>
                  <div class="flex items-center gap-2">
                    <n-button size="small" @click="openFsPicker">{{ t('jobs.actions.browseFs') }}</n-button>
                    <n-button size="small" :disabled="form.fsPaths.length === 0" @click="clearFsPaths">
                      {{ t('common.clear') }}
                    </n-button>
                  </div>
                </div>

                <div v-if="form.fsPaths.length === 0" class="text-sm opacity-60">
                  {{ t('jobs.fields.sourcePathsEmpty') }}
                </div>
                <div v-else class="flex flex-wrap gap-2">
                  <n-tag v-for="p in form.fsPaths" :key="p" closable @close="removeFsPath(p)">{{ p }}</n-tag>
                </div>

                <div class="flex gap-2">
                  <n-input
                    v-model:value="fsPathDraft"
                    :placeholder="t('jobs.fields.sourcePathsPlaceholder')"
                    @keyup.enter="addFsPathsFromDraft"
                  />
                  <n-button @click="addFsPathsFromDraft">{{ t('common.add') }}</n-button>
                </div>
              </div>
            </n-form-item>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
              <n-form-item :label="t('jobs.fields.fsSymlinkPolicy')">
                <n-select v-model:value="form.fsSymlinkPolicy" :options="fsSymlinkPolicyOptions" />
              </n-form-item>
              <n-form-item :label="t('jobs.fields.fsHardlinkPolicy')">
                <n-select v-model:value="form.fsHardlinkPolicy" :options="fsHardlinkPolicyOptions" />
              </n-form-item>
            </div>
            <n-form-item :label="t('jobs.fields.fsErrorPolicy')">
              <n-select v-model:value="form.fsErrorPolicy" :options="fsErrorPolicyOptions" />
              <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.fsErrorPolicyHelp') }}</div>
            </n-form-item>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
              <n-form-item :label="t('jobs.fields.fsInclude')">
                <div class="space-y-1 w-full">
                  <n-input
                    v-model:value="form.fsInclude"
                    type="textarea"
                    :autosize="{ minRows: 2, maxRows: 6 }"
                    :placeholder="t('jobs.fields.fsIncludePlaceholder')"
                  />
                  <div class="text-xs opacity-70">{{ t('jobs.fields.fsIncludeHelp') }}</div>
                </div>
              </n-form-item>
              <n-form-item :label="t('jobs.fields.fsExclude')">
                <div class="space-y-1 w-full">
                  <n-input
                    v-model:value="form.fsExclude"
                    type="textarea"
                    :autosize="{ minRows: 2, maxRows: 6 }"
                    :placeholder="t('jobs.fields.fsExcludePlaceholder')"
                  />
                  <div class="text-xs opacity-70">{{ t('jobs.fields.fsExcludeHelp') }}</div>
                </div>
              </n-form-item>
            </div>
          </template>

          <template v-else-if="form.jobType === 'sqlite'">
            <n-form-item :label="t('jobs.fields.sqlitePath')">
              <div class="space-y-1">
                <n-input v-model:value="form.sqlitePath" :placeholder="t('jobs.fields.sqlitePathPlaceholder')" />
                <div class="text-xs opacity-70">{{ t('jobs.fields.sqlitePathHelp') }}</div>
              </div>
            </n-form-item>
            <n-form-item :label="t('jobs.fields.sqliteIntegrityCheck')">
              <div class="space-y-1">
                <n-switch v-model:value="form.sqliteIntegrityCheck" />
                <div class="text-xs opacity-70">{{ t('jobs.fields.sqliteIntegrityCheckHelp') }}</div>
              </div>
            </n-form-item>
          </template>

          <template v-else>
            <n-form-item :label="t('jobs.fields.vaultwardenDataDir')">
              <div class="space-y-1">
                <n-input
                  v-model:value="form.vaultwardenDataDir"
                  :placeholder="t('jobs.fields.vaultwardenDataDirPlaceholder')"
                />
                <div class="text-xs opacity-70">{{ t('jobs.fields.vaultwardenDataDirHelp') }}</div>
              </div>
            </n-form-item>
          </template>
        </template>

        <template v-else-if="step === 3">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-4">
            <n-form-item :label="t('jobs.fields.targetType')">
              <n-select v-model:value="form.targetType" :options="targetTypeOptions" />
            </n-form-item>

            <n-form-item :label="t('jobs.fields.partSizeMiB')">
              <n-input-number v-model:value="form.partSizeMiB" :min="1" class="w-full" />
              <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.partSizeMiBHelp') }}</div>
            </n-form-item>
          </div>

          <template v-if="form.targetType === 'webdav'">
            <n-form-item :label="t('jobs.fields.webdavBaseUrl')">
              <n-input v-model:value="form.webdavBaseUrl" :placeholder="t('jobs.fields.webdavBaseUrlPlaceholder')" />
            </n-form-item>
            <n-form-item :label="t('jobs.fields.webdavSecret')">
              <n-select v-model:value="form.webdavSecretName" :options="webdavSecretOptions" filterable />
            </n-form-item>
          </template>
          <template v-else>
            <n-form-item :label="t('jobs.fields.localBaseDir')">
              <div class="space-y-1">
                <n-input v-model:value="form.localBaseDir" :placeholder="t('jobs.fields.localBaseDirPlaceholder')" />
                <div class="text-xs opacity-70">{{ t('jobs.fields.localBaseDirHelp') }}</div>
              </div>
            </n-form-item>
          </template>
        </template>

        <template v-else-if="step === 4">
          <n-form-item :label="t('jobs.fields.encryptionEnabled')">
            <div class="space-y-1">
              <n-switch v-model:value="form.encryptionEnabled" />
              <div class="text-xs opacity-70">{{ t('jobs.fields.encryptionHelp') }}</div>
            </div>
          </n-form-item>
          <n-form-item v-if="form.encryptionEnabled" :label="t('jobs.fields.encryptionKeyName')">
            <div class="space-y-1 w-full">
              <n-input
                v-model:value="form.encryptionKeyName"
                :placeholder="t('jobs.fields.encryptionKeyNamePlaceholder')"
              />
              <div class="text-xs opacity-70">{{ t('jobs.fields.encryptionKeyNameHelp') }}</div>
            </div>
          </n-form-item>
        </template>

        <template v-else-if="step === 5">
          <n-alert type="info" :bordered="false">
            {{ t('jobs.steps.notificationsHelp') }}
          </n-alert>

          <n-form-item :label="t('jobs.fields.notificationsMode')">
            <n-select v-model:value="form.notifyMode" :options="notifyModeOptions" />
            <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.notificationsModeHelp') }}</div>
          </n-form-item>

          <template v-if="form.notifyMode === 'custom'">
            <n-form-item :label="t('jobs.fields.notifyWecomBots')">
              <div class="space-y-2 w-full">
                <n-select
                  v-model:value="form.notifyWecomBots"
                  multiple
                  filterable
                  :options="wecomDestinationOptions"
                  :placeholder="t('jobs.fields.notifySelectPlaceholder')"
                />
                <div class="text-xs opacity-70">{{ t('jobs.fields.notifyEmptyMeansDisable') }}</div>
                <n-alert
                  v-if="disabledWecomSelected.length > 0"
                  type="warning"
                  :bordered="false"
                >
                  {{ t('jobs.fields.notifyDisabledSelected', { names: disabledWecomSelected.join(', ') }) }}
                </n-alert>
              </div>
            </n-form-item>

            <n-form-item :label="t('jobs.fields.notifyEmails')">
              <div class="space-y-2 w-full">
                <n-select
                  v-model:value="form.notifyEmails"
                  multiple
                  filterable
                  :options="emailDestinationOptions"
                  :placeholder="t('jobs.fields.notifySelectPlaceholder')"
                />
                <div class="text-xs opacity-70">{{ t('jobs.fields.notifyEmptyMeansDisable') }}</div>
                <n-alert
                  v-if="disabledEmailSelected.length > 0"
                  type="warning"
                  :bordered="false"
                >
                  {{ t('jobs.fields.notifyDisabledSelected', { names: disabledEmailSelected.join(', ') }) }}
                </n-alert>
              </div>
            </n-form-item>
          </template>
          <template v-else>
            <div class="text-xs opacity-70">{{ t('jobs.fields.notificationsInheritHelp') }}</div>
          </template>
        </template>

        <template v-else>
          <n-alert type="info" :bordered="false">
            {{ t('jobs.steps.reviewHelp') }}
          </n-alert>
          <n-code class="mt-2" :code="formatJson(previewPayload)" language="json" />
        </template>
      </n-form>

      <n-space justify="space-between">
        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
        <n-space>
          <n-button v-if="step > 1" @click="prevStep">{{ t('common.back') }}</n-button>
          <n-button v-if="step < EDITOR_STEPS_TOTAL" type="primary" @click="nextStep">
            {{ t('common.next') }}
          </n-button>
          <n-button v-else type="primary" :loading="saving" @click="save">
            {{ t('common.save') }}
          </n-button>
        </n-space>
      </n-space>
    </div>
  </n-modal>
  <FsPathPickerModal ref="fsPicker" @picked="addFsPathsFromList" />
</template>
