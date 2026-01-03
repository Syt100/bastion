<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { NButton, NCard, NDataTable, NPopconfirm, NSpace, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type JobListItem, type OverlapPolicy } from '@/stores/jobs'
import { useAgentsStore } from '@/stores/agents'
import { useSecretsStore } from '@/stores/secrets'
import { useUiStore } from '@/stores/ui'
import PageHeader from '@/components/PageHeader.vue'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'

import JobEditorModal, { type JobEditorModalExpose } from '@/components/jobs/JobEditorModal.vue'
import JobRunsModal, { type JobRunsModalExpose } from '@/components/jobs/JobRunsModal.vue'
import RunEventsModal, { type RunEventsModalExpose } from '@/components/jobs/RunEventsModal.vue'
import RestoreWizardModal, { type RestoreWizardModalExpose } from '@/components/jobs/RestoreWizardModal.vue'
import VerifyWizardModal, { type VerifyWizardModalExpose } from '@/components/jobs/VerifyWizardModal.vue'
import OperationModal, { type OperationModalExpose } from '@/components/jobs/OperationModal.vue'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const jobs = useJobsStore()
const agents = useAgentsStore()
const secrets = useSecretsStore()

const isDesktop = useMediaQuery(MQ.mdUp)

const editorModal = ref<JobEditorModalExpose | null>(null)
const runsModal = ref<JobRunsModalExpose | null>(null)
const runEventsModal = ref<RunEventsModalExpose | null>(null)
const restoreModal = ref<RestoreWizardModalExpose | null>(null)
const verifyModal = ref<VerifyWizardModalExpose | null>(null)
const opModal = ref<OperationModalExpose | null>(null)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

function formatJobNode(agentId: string | null): string {
  if (!agentId) return t('jobs.nodes.hub')
  const agent = agents.items.find((a) => a.id === agentId)
  return agent?.name ?? agentId
}

function formatOverlap(policy: OverlapPolicy): string {
  return policy === 'queue' ? t('jobs.overlap.queue') : t('jobs.overlap.reject')
}

async function refresh(): Promise<void> {
  try {
    await jobs.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchJobsFailed'), error, t))
  }
}

async function removeJob(jobId: string): Promise<void> {
  try {
    await jobs.deleteJob(jobId)
    message.success(t('messages.jobDeleted'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.deleteJobFailed'), error, t))
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
  } catch (error) {
    message.error(formatToastError(t('errors.runNowFailed'), error, t))
  }
}

function openCreate(): void {
  editorModal.value?.openCreate()
}

async function openEdit(jobId: string): Promise<void> {
  await editorModal.value?.openEdit(jobId)
}

async function openRuns(jobId: string): Promise<void> {
  await runsModal.value?.open(jobId)
}

async function openRunEvents(runId: string): Promise<void> {
  await runEventsModal.value?.open(runId)
}

function openRestoreWizard(runId: string): void {
  restoreModal.value?.open(runId)
}

function openVerifyWizard(runId: string): void {
  verifyModal.value?.open(runId)
}

async function openOperation(opId: string): Promise<void> {
  await opModal.value?.open(opId)
}

const columns = computed<DataTableColumns<JobListItem>>(() => [
  { title: t('jobs.columns.name'), key: 'name' },
  {
    title: t('jobs.columns.node'),
    key: 'agent_id',
    render: (row) => {
      return formatJobNode(row.agent_id)
    },
  },
  {
    title: t('jobs.columns.schedule'),
    key: 'schedule',
    render: (row) => row.schedule ?? '-',
  },
  {
    title: t('jobs.columns.overlap'),
    key: 'overlap_policy',
    render: (row) => formatOverlap(row.overlap_policy),
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
              { size: 'small', type: 'primary', onClick: () => void runNow(row.id) },
              { default: () => t('jobs.actions.runNow') },
            ),
            h(
              NButton,
              { size: 'small', onClick: () => void openRuns(row.id) },
              { default: () => t('jobs.actions.runs') },
            ),
            h(
              NButton,
              { size: 'small', onClick: () => void openEdit(row.id) },
              { default: () => t('common.edit') },
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => void removeJob(row.id),
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

onMounted(async () => {
  await refresh()
  try {
    await agents.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  }
  try {
    await secrets.refreshWebdav()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWebdavSecretsFailed'), error, t))
  }
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('jobs.title')" :subtitle="t('jobs.subtitle')">
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button type="primary" @click="openCreate">{{ t('jobs.actions.create') }}</n-button>
    </PageHeader>

    <div v-if="!isDesktop" class="space-y-3" data-testid="jobs-cards">
      <n-card
        v-if="!jobs.loading && jobs.items.length === 0"
        class="app-card"
      >
        <div class="text-sm opacity-70">{{ t('common.noData') }}</div>
      </n-card>

      <n-card
        v-for="job in jobs.items"
        :key="job.id"
        size="small"
        class="app-card"
      >
        <template #header>
          <div class="flex items-center justify-between gap-3">
            <div class="font-medium truncate">{{ job.name }}</div>
          </div>
        </template>

        <div class="text-sm">
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('jobs.columns.node') }}</div>
            <div class="text-right">{{ formatJobNode(job.agent_id) }}</div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('jobs.columns.schedule') }}</div>
            <div class="text-right">{{ job.schedule ?? '-' }}</div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('jobs.columns.overlap') }}</div>
            <div class="text-right">{{ formatOverlap(job.overlap_policy) }}</div>
          </div>
          <div class="flex items-start justify-between gap-4 py-1">
            <div class="opacity-70">{{ t('jobs.columns.updatedAt') }}</div>
            <div class="text-right">{{ formatUnixSeconds(job.updated_at) }}</div>
          </div>
        </div>

        <template #footer>
          <div class="flex flex-wrap justify-end gap-2">
            <n-button size="small" type="primary" @click="runNow(job.id)">{{ t('jobs.actions.runNow') }}</n-button>
            <n-button size="small" @click="openRuns(job.id)">{{ t('jobs.actions.runs') }}</n-button>
            <n-button size="small" @click="openEdit(job.id)">{{ t('common.edit') }}</n-button>
            <n-popconfirm
              :positive-text="t('common.delete')"
              :negative-text="t('common.cancel')"
              @positive-click="removeJob(job.id)"
            >
              <template #trigger>
                <n-button size="small" type="error" tertiary>{{ t('common.delete') }}</n-button>
              </template>
              {{ t('jobs.deleteConfirm') }}
            </n-popconfirm>
          </div>
        </template>
      </n-card>
    </div>

    <div v-else data-testid="jobs-table">
      <n-card class="app-card">
        <div class="overflow-x-auto">
          <n-data-table :loading="jobs.loading" :columns="columns" :data="jobs.items" />
        </div>
      </n-card>
    </div>

    <JobEditorModal ref="editorModal" @saved="refresh" />

    <JobRunsModal
      ref="runsModal"
      @open-events="(id) => void openRunEvents(id)"
      @open-restore="openRestoreWizard"
      @open-verify="openVerifyWizard"
    />

    <RunEventsModal ref="runEventsModal" />

    <RestoreWizardModal ref="restoreModal" @started="(id) => void openOperation(id)" />

    <VerifyWizardModal ref="verifyModal" @started="(id) => void openOperation(id)" />

    <OperationModal ref="opModal" />
  </div>
</template>
