<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NCheckbox, NDataTable, NDropdown, NModal, NSpace, NSwitch, NTag, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useJobsStore, type JobListItem, type OverlapPolicy } from '@/stores/jobs'
import { useAgentsStore } from '@/stores/agents'
import { useSecretsStore } from '@/stores/secrets'
import { useUiStore } from '@/stores/ui'
import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { MODAL_WIDTH } from '@/lib/modal'

import JobEditorModal, { type JobEditorModalExpose } from '@/components/jobs/JobEditorModal.vue'
import JobDeployModal, { type JobDeployModalExpose } from '@/components/jobs/JobDeployModal.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const jobs = useJobsStore()
const agents = useAgentsStore()
const secrets = useSecretsStore()
const route = useRoute()
const router = useRouter()

const isDesktop = useMediaQuery(MQ.mdUp)

const editorModal = ref<JobEditorModalExpose | null>(null)
const deployModal = ref<JobDeployModalExpose | null>(null)

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

async function archiveJob(jobId: string, opts?: { cascadeSnapshots?: boolean }): Promise<boolean> {
  try {
    await jobs.archiveJob(jobId, { cascadeSnapshots: !!opts?.cascadeSnapshots })
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

async function openDeploy(jobId: string): Promise<void> {
  await deployModal.value?.open(jobId)
}

function openJobDetail(jobId: string): void {
  void router.push(`/n/${encodeURIComponent(nodeIdOrHub.value)}/jobs/${encodeURIComponent(jobId)}`)
}

function openSnapshots(jobId: string): void {
  void router.push(`/n/${encodeURIComponent(nodeIdOrHub.value)}/jobs/${encodeURIComponent(jobId)}/snapshots`)
}

const deleteOpen = ref<boolean>(false)
const deleteTarget = ref<JobListItem | null>(null)
const deleteBusy = ref<'archive' | 'delete' | null>(null)
const archiveCascadeSnapshots = ref<boolean>(false)

function openDelete(job: JobListItem): void {
  deleteTarget.value = job
  archiveCascadeSnapshots.value = false
  deleteOpen.value = true
}

async function confirmArchive(): Promise<void> {
  const job = deleteTarget.value
  if (!job) return
  deleteBusy.value = 'archive'
  try {
    const ok = await archiveJob(job.id, { cascadeSnapshots: archiveCascadeSnapshots.value })
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
                { size: 'small', onClick: () => openJobDetail(row.id) },
                { default: () => t('common.browse') },
              ),
              h(
                NDropdown,
                {
                  trigger: 'click',
                  options: [
                    { label: t('jobs.actions.snapshots'), key: 'snapshots' },
                    { label: t('jobs.retention.title'), key: 'retention' },
                    { type: 'divider', key: '__d1' },
                    { label: t('common.edit'), key: 'edit', disabled: !!row.archived_at },
                    { label: t('jobs.actions.deploy'), key: 'deploy', disabled: !!row.archived_at },
                    row.archived_at
                      ? { label: t('jobs.actions.unarchive'), key: 'unarchive' }
                      : { label: t('jobs.actions.archive'), key: 'archive' },
                    { type: 'divider', key: '__d2' },
                    { label: t('common.delete'), key: 'delete' },
                  ],
                  onSelect: (key: string | number) => {
                    if (key === 'snapshots') return void openSnapshots(row.id)
                    if (key === 'retention') return void router.push(`/n/${encodeURIComponent(nodeIdOrHub.value)}/jobs/${encodeURIComponent(row.id)}/retention`)
                    if (key === 'edit') return void openEdit(row.id)
                    if (key === 'deploy') return void openDeploy(row.id)
                    if (key === 'unarchive') return void unarchiveJob(row.id)
                    if (key === 'archive') return void openDelete(row)
                    if (key === 'delete') return void openDelete(row)
                  },
                },
                {
                  default: () =>
                    h(
                      NButton,
                      { size: 'small', tertiary: true },
                      { default: () => t('common.more') },
                    ),
                },
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
      <template #prefix>
        <NodeContextTag :node-id="nodeIdOrHub" />
      </template>
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
            <n-button size="small" @click="openJobDetail(job.id)">{{ t('common.browse') }}</n-button>
            <n-dropdown
              trigger="click"
              :options="[
                { label: t('jobs.actions.snapshots'), key: 'snapshots' },
                { label: t('jobs.retention.title'), key: 'retention' },
                { type: 'divider', key: '__d1' },
                { label: t('common.edit'), key: 'edit', disabled: !!job.archived_at },
                { label: t('jobs.actions.deploy'), key: 'deploy', disabled: !!job.archived_at },
                job.archived_at
                  ? { label: t('jobs.actions.unarchive'), key: 'unarchive' }
                  : { label: t('jobs.actions.archive'), key: 'archive' },
                { type: 'divider', key: '__d2' },
                { label: t('common.delete'), key: 'delete' },
              ]"
              @select="
                (key) => {
                  if (key === 'snapshots') return openSnapshots(job.id)
                  if (key === 'retention') return router.push(`/n/${encodeURIComponent(nodeIdOrHub)}/jobs/${encodeURIComponent(job.id)}/retention`)
                  if (key === 'edit') return openEdit(job.id)
                  if (key === 'deploy') return openDeploy(job.id)
                  if (key === 'unarchive') return unarchiveJob(job.id)
                  if (key === 'archive') return openDelete(job)
                  if (key === 'delete') return openDelete(job)
                }
              "
            >
              <n-button size="small" tertiary>{{ t('common.more') }}</n-button>
            </n-dropdown>
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

    <JobDeployModal ref="deployModal" />

    <n-modal v-model:show="deleteOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('jobs.deleteTitle')">
      <div class="space-y-3">
        <div class="text-sm opacity-80">
          {{
            deleteTarget?.archived_at
              ? t('jobs.deletePermanentlyHelp')
              : t('jobs.deleteHelp')
          }}
        </div>

        <div v-if="deleteTarget && !deleteTarget.archived_at" class="rounded border border-slate-200/60 dark:border-slate-700/60 p-3 space-y-1">
          <n-checkbox :checked="archiveCascadeSnapshots" @update:checked="(v) => (archiveCascadeSnapshots = v)">
            {{ t('jobs.archiveCascadeLabel') }}
          </n-checkbox>
          <div class="text-xs opacity-70">{{ t('jobs.archiveCascadeHelp') }}</div>
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
