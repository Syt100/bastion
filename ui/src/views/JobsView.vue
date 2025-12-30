<script setup lang="ts">
import { computed, h, onMounted, reactive, ref } from 'vue'
import {
  NButton,
  NCard,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NPopconfirm,
  NSelect,
  NSpace,
  NSwitch,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type JobListItem, type JobType, type OverlapPolicy, type RunListItem } from '@/stores/jobs'
import { useSecretsStore } from '@/stores/secrets'
import { useUiStore } from '@/stores/ui'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const jobs = useJobsStore()
const secrets = useSecretsStore()

const editorOpen = ref<boolean>(false)
const editorMode = ref<'create' | 'edit'>('create')
const editorSaving = ref<boolean>(false)

const runsOpen = ref<boolean>(false)
const runsLoading = ref<boolean>(false)
const runsJobId = ref<string | null>(null)
const runs = ref<RunListItem[]>([])

const form = reactive<{
  id: string | null
  name: string
  schedule: string
  overlapPolicy: OverlapPolicy
  jobType: JobType
  fsRoot: string
  sqlitePath: string
  sqliteIntegrityCheck: boolean
  vaultwardenDataDir: string
  webdavBaseUrl: string
  webdavSecretName: string
  partSizeMiB: number
}>({
  id: null,
  name: '',
  schedule: '',
  overlapPolicy: 'queue',
  jobType: 'filesystem',
  fsRoot: '',
  sqlitePath: '',
  sqliteIntegrityCheck: false,
  vaultwardenDataDir: '',
  webdavBaseUrl: '',
  webdavSecretName: '',
  partSizeMiB: 256,
})

const dateFormatter = computed(
  () =>
    new Intl.DateTimeFormat(ui.locale, {
      dateStyle: 'medium',
      timeStyle: 'medium',
    }),
)

function formatUnixSeconds(ts: number | null): string {
  if (!ts) return '-'
  return dateFormatter.value.format(new Date(ts * 1000))
}

function openCreate(): void {
  editorMode.value = 'create'
  form.id = null
  form.name = ''
  form.schedule = ''
  form.overlapPolicy = 'queue'
  form.jobType = 'filesystem'
  form.fsRoot = ''
  form.sqlitePath = ''
  form.sqliteIntegrityCheck = false
  form.vaultwardenDataDir = ''
  form.webdavBaseUrl = ''
  form.webdavSecretName = ''
  form.partSizeMiB = 256
  editorOpen.value = true
}

async function openEdit(jobId: string): Promise<void> {
  editorMode.value = 'edit'
  editorOpen.value = true
  editorSaving.value = true
  try {
    const job = await jobs.getJob(jobId)
    form.id = job.id
    form.name = job.name
    form.schedule = job.schedule ?? ''
    form.overlapPolicy = job.overlap_policy
    form.jobType = job.spec.type

    const target = (job.spec as Record<string, unknown>).target as Record<string, unknown> | undefined
    form.webdavBaseUrl = typeof target?.base_url === 'string' ? target.base_url : ''
    form.webdavSecretName = typeof target?.secret_name === 'string' ? target.secret_name : ''
    form.partSizeMiB =
      typeof target?.part_size_bytes === 'number' && target.part_size_bytes > 0
        ? Math.max(1, Math.round(target.part_size_bytes / (1024 * 1024)))
        : 256

    const source = (job.spec as Record<string, unknown>).source as Record<string, unknown> | undefined
    form.fsRoot = typeof source?.root === 'string' ? source.root : ''
    form.sqlitePath = typeof source?.path === 'string' ? source.path : ''
    form.sqliteIntegrityCheck = typeof source?.integrity_check === 'boolean' ? source.integrity_check : false
    form.vaultwardenDataDir = typeof source?.data_dir === 'string' ? source.data_dir : ''
  } catch {
    message.error(t('errors.fetchJobFailed'))
    editorOpen.value = false
  } finally {
    editorSaving.value = false
  }
}

async function save(): Promise<void> {
  const name = form.name.trim()
  if (!name) {
    message.error(t('errors.jobNameRequired'))
    return
  }

  const webdavBaseUrl = form.webdavBaseUrl.trim()
  if (!webdavBaseUrl) {
    message.error(t('errors.webdavBaseUrlRequired'))
    return
  }
  const webdavSecretName = form.webdavSecretName.trim()
  if (!webdavSecretName) {
    message.error(t('errors.webdavSecretRequired'))
    return
  }

  const partSizeMiB = Math.max(1, Math.floor(form.partSizeMiB))
  const partSizeBytes = partSizeMiB * 1024 * 1024

  const source =
    form.jobType === 'filesystem'
      ? { root: form.fsRoot.trim(), include: [], exclude: [] }
      : form.jobType === 'sqlite'
        ? { path: form.sqlitePath.trim(), integrity_check: form.sqliteIntegrityCheck }
        : { data_dir: form.vaultwardenDataDir.trim() }

  if (form.jobType === 'filesystem' && !source.root) {
    message.error(t('errors.sourceRootRequired'))
    return
  }
  if (form.jobType === 'sqlite' && !source.path) {
    message.error(t('errors.sqlitePathRequired'))
    return
  }
  if (form.jobType === 'vaultwarden' && !source.data_dir) {
    message.error(t('errors.vaultwardenDataDirRequired'))
    return
  }

  editorSaving.value = true
  try {
    const payload = {
      name,
      schedule: form.schedule.trim() ? form.schedule.trim() : null,
      overlap_policy: form.overlapPolicy,
      spec: {
        v: 1 as const,
        type: form.jobType,
        source,
        target: {
          type: 'webdav' as const,
          base_url: webdavBaseUrl,
          secret_name: webdavSecretName,
          part_size_bytes: partSizeBytes,
        },
      },
    }

    if (editorMode.value === 'create') {
      await jobs.createJob(payload)
      message.success(t('messages.jobCreated'))
    } else if (form.id) {
      await jobs.updateJob(form.id, payload)
      message.success(t('messages.jobUpdated'))
    }

    editorOpen.value = false
    await refresh()
  } catch (error) {
    const msg =
      error && typeof error === 'object' && 'message' in error
        ? String((error as { message: unknown }).message)
        : t('errors.saveJobFailed')
    message.error(msg)
  } finally {
    editorSaving.value = false
  }
}

async function refresh(): Promise<void> {
  try {
    await jobs.refresh()
  } catch {
    message.error(t('errors.fetchJobsFailed'))
  }
}

async function removeJob(jobId: string): Promise<void> {
  try {
    await jobs.deleteJob(jobId)
    message.success(t('messages.jobDeleted'))
    await refresh()
  } catch {
    message.error(t('errors.deleteJobFailed'))
  }
}

async function runNow(jobId: string): Promise<void> {
  try {
    const res = await jobs.runNow(jobId)
    if (res.status === 'rejected') {
      message.warning(t('messages.runRejected'))
    } else {
      message.success(t('messages.runQueued'))
    }
  } catch {
    message.error(t('errors.runNowFailed'))
  }
}

async function openRuns(jobId: string): Promise<void> {
  runsOpen.value = true
  runsJobId.value = jobId
  runsLoading.value = true
  runs.value = []
  try {
    runs.value = await jobs.listRuns(jobId)
  } catch {
    message.error(t('errors.fetchRunsFailed'))
  } finally {
    runsLoading.value = false
  }
}

const overlapOptions = computed(() => [
  { label: t('jobs.overlap.queue'), value: 'queue' },
  { label: t('jobs.overlap.reject'), value: 'reject' },
])

const jobTypeOptions = computed(() => [
  { label: t('jobs.types.filesystem'), value: 'filesystem' },
  { label: t('jobs.types.sqlite'), value: 'sqlite' },
  { label: t('jobs.types.vaultwarden'), value: 'vaultwarden' },
])

function statusTagType(status: RunListItem['status']): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  return 'default'
}

const columns = computed<DataTableColumns<JobListItem>>(() => [
  { title: t('jobs.columns.name'), key: 'name' },
  {
    title: t('jobs.columns.schedule'),
    key: 'schedule',
    render: (row) => row.schedule ?? '-',
  },
  {
    title: t('jobs.columns.overlap'),
    key: 'overlap_policy',
    render: (row) => (row.overlap_policy === 'queue' ? t('jobs.overlap.queue') : t('jobs.overlap.reject')),
  },
  {
    title: t('jobs.columns.updatedAt'),
    key: 'updated_at',
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('jobs.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NButton,
              { size: 'small', type: 'primary', onClick: () => runNow(row.id) },
              { default: () => t('jobs.actions.runNow') },
            ),
            h(
              NButton,
              { size: 'small', onClick: () => openRuns(row.id) },
              { default: () => t('jobs.actions.runs') },
            ),
            h(NButton, { size: 'small', onClick: () => openEdit(row.id) }, { default: () => t('common.edit') }),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => removeJob(row.id),
                positiveText: t('common.delete'),
                negativeText: t('common.cancel'),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { size: 'small', type: 'error', tertiary: true },
                    { default: () => t('common.delete') },
                  ),
                default: () => t('jobs.deleteConfirm'),
              },
            ),
          ],
        },
      ),
  },
])

const runColumns = computed<DataTableColumns<RunListItem>>(() => [
  {
    title: t('runs.columns.status'),
    key: 'status',
    render: (row) =>
      h(NTag, { type: statusTagType(row.status) }, { default: () => row.status }),
  },
  { title: t('runs.columns.startedAt'), key: 'started_at', render: (row) => formatUnixSeconds(row.started_at) },
  { title: t('runs.columns.endedAt'), key: 'ended_at', render: (row) => formatUnixSeconds(row.ended_at) },
  { title: t('runs.columns.error'), key: 'error', render: (row) => row.error ?? '-' },
])

const webdavSecretOptions = computed(() =>
  secrets.webdav.map((s) => ({ label: s.name, value: s.name })),
)

onMounted(async () => {
  await refresh()
  try {
    await secrets.refreshWebdav()
  } catch {
    message.error(t('errors.fetchWebdavSecretsFailed'))
  }
})
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
      <div>
        <h1 class="text-xl font-semibold">{{ t('jobs.title') }}</h1>
        <p class="text-sm opacity-70">{{ t('jobs.subtitle') }}</p>
      </div>
      <n-space>
        <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
        <n-button type="primary" @click="openCreate">{{ t('jobs.actions.create') }}</n-button>
      </n-space>
    </div>

    <n-card>
      <n-data-table :loading="jobs.loading" :columns="columns" :data="jobs.items" />
    </n-card>

    <n-modal v-model:show="editorOpen" preset="card" :title="editorMode === 'create' ? t('jobs.createTitle') : t('jobs.editTitle')">
      <div class="space-y-4">
        <n-form label-placement="top">
          <n-form-item :label="t('jobs.fields.name')">
            <n-input v-model:value="form.name" />
          </n-form-item>
          <n-form-item :label="t('jobs.fields.schedule')">
            <n-input v-model:value="form.schedule" :placeholder="t('jobs.fields.schedulePlaceholder')" />
            <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.scheduleHelp') }}</div>
          </n-form-item>
          <n-form-item :label="t('jobs.fields.overlap')">
            <n-select v-model:value="form.overlapPolicy" :options="overlapOptions" />
          </n-form-item>
          <n-form-item :label="t('jobs.fields.type')">
            <n-select v-model:value="form.jobType" :options="jobTypeOptions" />
          </n-form-item>

          <n-form-item v-if="form.jobType === 'filesystem'" :label="t('jobs.fields.sourceRoot')">
            <n-input v-model:value="form.fsRoot" :placeholder="t('jobs.fields.sourceRootPlaceholder')" />
          </n-form-item>
          <n-form-item v-if="form.jobType === 'sqlite'" :label="t('jobs.fields.sqlitePath')">
            <n-input v-model:value="form.sqlitePath" :placeholder="t('jobs.fields.sqlitePathPlaceholder')" />
          </n-form-item>
          <n-form-item v-if="form.jobType === 'sqlite'" :label="t('jobs.fields.sqliteIntegrityCheck')">
            <div class="space-y-1">
              <n-switch v-model:value="form.sqliteIntegrityCheck" />
              <div class="text-xs opacity-70">{{ t('jobs.fields.sqliteIntegrityCheckHelp') }}</div>
            </div>
          </n-form-item>
          <n-form-item v-if="form.jobType === 'vaultwarden'" :label="t('jobs.fields.vaultwardenDataDir')">
            <div class="space-y-1">
              <n-input v-model:value="form.vaultwardenDataDir" :placeholder="t('jobs.fields.vaultwardenDataDirPlaceholder')" />
              <div class="text-xs opacity-70">{{ t('jobs.fields.vaultwardenDataDirHelp') }}</div>
            </div>
          </n-form-item>

          <n-form-item :label="t('jobs.fields.webdavBaseUrl')">
            <n-input v-model:value="form.webdavBaseUrl" :placeholder="t('jobs.fields.webdavBaseUrlPlaceholder')" />
          </n-form-item>
          <n-form-item :label="t('jobs.fields.webdavSecret')">
            <n-select v-model:value="form.webdavSecretName" :options="webdavSecretOptions" filterable />
          </n-form-item>
          <n-form-item :label="t('jobs.fields.partSizeMiB')">
            <n-input-number v-model:value="form.partSizeMiB" :min="1" class="w-full" />
            <div class="text-xs opacity-70 mt-1">{{ t('jobs.fields.partSizeMiBHelp') }}</div>
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="editorOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="editorSaving" @click="save">{{ t('common.save') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="runsOpen" preset="card" :title="t('runs.title')">
      <div class="space-y-3">
        <div class="text-sm opacity-70">{{ runsJobId }}</div>
        <n-data-table :loading="runsLoading" :columns="runColumns" :data="runs" />
        <n-space justify="end">
          <n-button @click="runsOpen = false">{{ t('common.close') }}</n-button>
        </n-space>
      </div>
    </n-modal>
  </div>
</template>
