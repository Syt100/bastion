<script setup lang="ts">
import { computed, h, onMounted, reactive, ref } from 'vue'
import {
  NButton,
  NCard,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NModal,
  NPopconfirm,
  NSelect,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type JobListItem, type JobType, type OverlapPolicy, type RunListItem } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const jobs = useJobsStore()

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
}>({
  id: null,
  name: '',
  schedule: '',
  overlapPolicy: 'queue',
  jobType: 'filesystem',
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

  editorSaving.value = true
  try {
    const payload = {
      name,
      schedule: form.schedule.trim() ? form.schedule.trim() : null,
      overlap_policy: form.overlapPolicy,
      spec: { v: 1 as const, type: form.jobType },
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

onMounted(refresh)
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
