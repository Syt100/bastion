<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { NButton, NCard, NDataTable, NModal, NSpace, NSwitch, NTag, useMessage, type DataTableColumns } from 'naive-ui'
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
import { MODAL_WIDTH } from '@/lib/modal'

import JobEditorModal, { type JobEditorModalExpose } from '@/components/jobs/JobEditorModal.vue'
import JobRunsModal, { type JobRunsModalExpose } from '@/components/jobs/JobRunsModal.vue'
import RunEventsModal, { type RunEventsModalExpose } from '@/components/jobs/RunEventsModal.vue'
import RestoreWizardModal, { type RestoreWizardModalExpose } from '@/components/jobs/RestoreWizardModal.vue'
import VerifyWizardModal, { type VerifyWizardModalExpose } from '@/components/jobs/VerifyWizardModal.vue'
import OperationModal, { type OperationModalExpose } from '@/components/jobs/OperationModal.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const jobs = useJobsStore()
const agents = useAgentsStore()
const secrets = useSecretsStore()
const route = useRoute()

const isDesktop = useMediaQuery(MQ.mdUp)

const editorModal = ref<JobEditorModalExpose | null>(null)
const runsModal = ref<JobRunsModalExpose | null>(null)
const runEventsModal = ref<RunEventsModalExpose | null>(null)
const restoreModal = ref<RestoreWizardModalExpose | null>(null)
const verifyModal = ref<VerifyWizardModalExpose | null>(null)
const opModal = ref<OperationModalExpose | null>(null)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : null))
const inNodeContext = computed(() => nodeId.value !== null)
const nodeIdOrHub = computed(() => nodeId.value ?? 'hub')

const showArchived = ref<boolean>(false)

const visibleJobs = computed(() => {
  const id = nodeId.value
  if (!id) return jobs.items
  if (id === 'hub') return jobs.items.filter((j) => j.agent_id === null)
  return jobs.items.filter((j) => j.agent_id === id)
})

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
    await jobs.refresh({ includeArchived: showArchived.value })
  } catch (error) {
    message.error(formatToastError(t('errors.fetchJobsFailed'), error, t))
  }
}

async function removeJob(jobId: string): Promise<boolean> {
  try {
    await jobs.deleteJob(jobId)
    message.success(t('messages.jobDeleted'))
    await refresh()
    return true
  } catch (error) {
    message.error(formatToastError(t('errors.deleteJobFailed'), error, t))
    return false
  }
}

async function archiveJob(jobId: string): Promise<boolean> {
  try {
    await jobs.archiveJob(jobId)
    message.success(t('messages.jobArchived'))
    await refresh()
    return true
  } catch (error) {
    message.error(formatToastError(t('errors.archiveJobFailed'), error, t))
    return false
  }
}

async function unarchiveJob(jobId: string): Promise<void> {
  try {
    await jobs.unarchiveJob(jobId)
    message.success(t('messages.jobUnarchived'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.unarchiveJobFailed'), error, t))
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
  const currentNode = nodeId.value
  editorModal.value?.openCreate(currentNode ? { nodeId: currentNode } : undefined)
}

async function openEdit(jobId: string): Promise<void> {
  const currentNode = nodeId.value
  await editorModal.value?.openEdit(jobId, currentNode ? { nodeId: currentNode } : undefined)
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

const deleteOpen = ref<boolean>(false)
const deleteTarget = ref<JobListItem | null>(null)
const deleteBusy = ref<'archive' | 'delete' | null>(null)

function openDelete(job: JobListItem): void {
  deleteTarget.value = job
  deleteOpen.value = true
}

async function confirmArchive(): Promise<void> {
  const job = deleteTarget.value
  if (!job) return
  deleteBusy.value = 'archive'
  try {
    const ok = await archiveJob(job.id)
    if (ok) deleteOpen.value = false
  } finally {
    deleteBusy.value = null
  }
}

async function confirmDeletePermanently(): Promise<void> {
  const job = deleteTarget.value
  if (!job) return
  deleteBusy.value = 'delete'
  try {
    const ok = await removeJob(job.id)
    if (ok) deleteOpen.value = false
  } finally {
    deleteBusy.value = null
  }
}

const columns = computed<DataTableColumns<JobListItem>>(() => {
  const cols: DataTableColumns<JobListItem> = [
    {
      title: t('jobs.columns.name'),
      key: 'name',
      render: (row) =>
        h(
          NSpace,
          { size: 6, align: 'center' },
          {
            default: () => [
              h('span', { class: 'truncate' }, row.name),
              row.archived_at
                ? h(
                    NTag,
                    { size: 'small', bordered: false },
                    { default: () => t('jobs.archived') },
                  )
                : null,
            ],
          },
        ),
    },
  ]

  if (!inNodeContext.value) {
    cols.push({
      title: t('jobs.columns.node'),
      key: 'agent_id',
      render: (row) => {
        return formatJobNode(row.agent_id)
      },
    })
  }

  cols.push(
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
                {
                  size: 'small',
                  type: 'primary',
                  disabled: !!row.archived_at,
                  onClick: () => void runNow(row.id),
                },
                { default: () => t('jobs.actions.runNow') },
              ),
              h(
                NButton,
                { size: 'small', onClick: () => void openRuns(row.id) },
                { default: () => t('jobs.actions.runs') },
              ),
              h(
                NButton,
                { size: 'small', disabled: !!row.archived_at, onClick: () => void openEdit(row.id) },
                { default: () => t('common.edit') },
              ),
              row.archived_at
                ? h(
                    NButton,
                    { size: 'small', onClick: () => void unarchiveJob(row.id) },
                    { default: () => t('jobs.actions.unarchive') },
                  )
                : null,
              h(
                NButton,
                { size: 'small', type: 'error', tertiary: true, onClick: () => openDelete(row) },
                { default: () => t('common.delete') },
              ),
            ],
          },
        ),
    },
  )

  return cols
})

onMounted(async () => {
  await refresh()
  try {
    await agents.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  }
  try {
    await secrets.refreshWebdav(nodeIdOrHub.value)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWebdavSecretsFailed'), error, t))
  }
})

watch(nodeIdOrHub, async () => {
  try {
    await secrets.refreshWebdav(nodeIdOrHub.value)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWebdavSecretsFailed'), error, t))
  }
})

watch(showArchived, () => {
  void refresh()
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('jobs.title')" :subtitle="t('jobs.subtitle')">
      <div class="flex items-center gap-2">
        <span class="text-sm opacity-70">{{ t('jobs.showArchived') }}</span>
        <n-switch v-model:value="showArchived" />
      </div>
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button type="primary" @click="openCreate">{{ t('jobs.actions.create') }}</n-button>
    </PageHeader>

    <div v-if="!isDesktop" class="space-y-3" data-testid="jobs-cards">
      <AppEmptyState v-if="jobs.loading && visibleJobs.length === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!jobs.loading && visibleJobs.length === 0" :title="t('common.noData')" />

      <n-card
        v-for="job in visibleJobs"
        :key="job.id"
        size="small"
        class="app-card"
      >
        <template #header>
          <div class="flex items-center justify-between gap-3">
            <div class="min-w-0">
              <div class="font-medium truncate">{{ job.name }}</div>
              <div v-if="job.archived_at" class="text-xs opacity-70 mt-0.5">
                {{ t('jobs.archived') }}
              </div>
            </div>
          </div>
        </template>

        <div class="text-sm">
          <div v-if="!inNodeContext" class="flex items-start justify-between gap-4 py-1">
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
            <n-button size="small" type="primary" :disabled="!!job.archived_at" @click="runNow(job.id)">{{ t('jobs.actions.runNow') }}</n-button>
            <n-button size="small" @click="openRuns(job.id)">{{ t('jobs.actions.runs') }}</n-button>
            <n-button size="small" :disabled="!!job.archived_at" @click="openEdit(job.id)">{{ t('common.edit') }}</n-button>
            <n-button v-if="job.archived_at" size="small" @click="unarchiveJob(job.id)">{{ t('jobs.actions.unarchive') }}</n-button>
            <n-button size="small" type="error" tertiary @click="openDelete(job)">{{ t('common.delete') }}</n-button>
          </div>
        </template>
      </n-card>
    </div>

    <div v-else data-testid="jobs-table">
      <n-card class="app-card">
        <div class="overflow-x-auto">
          <n-data-table :loading="jobs.loading" :columns="columns" :data="visibleJobs" />
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

    <n-modal v-model:show="deleteOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('jobs.deleteTitle')">
      <div class="space-y-3">
        <div class="text-sm opacity-80">
          {{
            deleteTarget?.archived_at
              ? t('jobs.deletePermanentlyHelp')
              : t('jobs.deleteHelp')
          }}
        </div>

        <div class="flex items-center justify-end gap-2">
          <n-button :disabled="deleteBusy !== null" @click="deleteOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button
            v-if="deleteTarget && !deleteTarget.archived_at"
            type="warning"
            :loading="deleteBusy === 'archive'"
            @click="confirmArchive"
          >
            {{ t('jobs.actions.archive') }}
          </n-button>
          <n-button type="error" :loading="deleteBusy === 'delete'" @click="confirmDeletePermanently">
            {{ t('jobs.actions.deletePermanently') }}
          </n-button>
        </div>
      </div>
    </n-modal>
  </div>
</template>
