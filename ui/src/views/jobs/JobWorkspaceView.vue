<script setup lang="ts">
import { computed, provide, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NCheckbox, NCode, NDrawer, NDrawerContent, NDropdown, NModal, NTabPane, NTabs, NTag, useMessage } from 'naive-ui'
import { EllipsisHorizontal } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import NodeContextTag from '@/components/NodeContextTag.vue'
import MobileTopBar from '@/components/MobileTopBar.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import JobEditorModal, { type JobEditorModalExpose } from '@/components/jobs/JobEditorModal.vue'
import JobDeployModal, { type JobDeployModalExpose } from '@/components/jobs/JobDeployModal.vue'
import RunDetailPanel from '@/components/runs/RunDetailPanel.vue'
import ScrollShadowPane from '@/components/scroll/ScrollShadowPane.vue'
import { useJobsStore, type JobDetail } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { MODAL_WIDTH } from '@/lib/modal'
import { MQ } from '@/lib/breakpoints'
import { useMediaQuery } from '@/lib/media'
import { JOB_DETAIL_CONTEXT } from '@/lib/jobDetailContext'

type SectionKey = 'overview' | 'history' | 'data'

const { t } = useI18n()
const message = useMessage()
const route = useRoute()
const router = useRouter()
const ui = useUiStore()
const jobs = useJobsStore()

const isDesktop = useMediaQuery(MQ.mdUp)
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : 'hub'))
const jobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))
const runId = computed(() => (typeof route.params.runId === 'string' ? route.params.runId : null))

const loading = ref<boolean>(false)
const job = ref<JobDetail | null>(null)

async function refresh(): Promise<void> {
  const id = jobId.value
  if (!id) return
  loading.value = true
  try {
    job.value = await jobs.getJob(id)
  } catch (error) {
    job.value = null
    message.error(formatToastError(t('errors.fetchJobFailed'), error, t))
  } finally {
    loading.value = false
  }
}

watch(jobId, () => void refresh(), { immediate: true })

provide(JOB_DETAIL_CONTEXT, { nodeId, jobId, job, loading, refresh })

const activeSection = computed<SectionKey>(() => {
  const p = route.path
  if (p.includes('/data')) return 'data'
  if (p.includes('/history')) return 'history'
  return 'overview'
})

function goSection(key: unknown): void {
  if (typeof key !== 'string') return
  if (key !== 'overview' && key !== 'history' && key !== 'data') return
  const id = jobId.value
  if (!id) return
  void router.push(`/n/${encodeURIComponent(nodeId.value)}/jobs/${encodeURIComponent(id)}/${key}`)
}

function closeRunDrawer(): void {
  if (!runId.value) return
  const basePath = route.path.replace(/\/runs\/[^/]+$/, '')
  if (basePath === route.path) return
  void router.push({ path: basePath, query: route.query, hash: route.hash })
}

const runDrawerOpen = computed<boolean>({
  get: () => !!runId.value,
  set: (value) => {
    if (value) return
    closeRunDrawer()
  },
})

const runDrawerWidth = computed(() => (isDesktop.value ? '900px' : '100vw'))

async function runNow(): Promise<void> {
  const j = job.value
  if (!j) return
  try {
    const res = await jobs.runNow(j.id)
    if (res.status === 'rejected') message.warning(t('messages.runRejected'))
    else message.success(t('messages.runQueued'))
  } catch (error) {
    message.error(formatToastError(t('errors.runNowFailed'), error, t))
  }
}

const editorModal = ref<JobEditorModalExpose | null>(null)
const deployModal = ref<JobDeployModalExpose | null>(null)

async function openEdit(): Promise<void> {
  const id = jobId.value
  if (!id) return
  await editorModal.value?.openEdit(id, { nodeId: nodeId.value })
}

async function openDeploy(): Promise<void> {
  const id = jobId.value
  if (!id) return
  await deployModal.value?.open(id)
}

const inspectOpen = ref<boolean>(false)
const jobJson = computed(() => {
  const j = job.value
  if (!j) return ''
  try {
    return JSON.stringify(j, null, 2)
  } catch {
    return String(j)
  }
})

const deleteOpen = ref<boolean>(false)
const deleteBusy = ref<'archive' | 'delete' | null>(null)
const archiveCascadeSnapshots = ref<boolean>(false)

function openArchiveOrDelete(): void {
  archiveCascadeSnapshots.value = false
  deleteOpen.value = true
}

async function confirmArchive(): Promise<void> {
  const j = job.value
  if (!j) return
  deleteBusy.value = 'archive'
  try {
    await jobs.archiveJob(j.id, { cascadeSnapshots: archiveCascadeSnapshots.value })
    message.success(t('messages.jobArchived'))
    deleteOpen.value = false
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.archiveJobFailed'), error, t))
  } finally {
    deleteBusy.value = null
  }
}

async function confirmDeletePermanently(): Promise<void> {
  const j = job.value
  if (!j) return
  deleteBusy.value = 'delete'
  try {
    await jobs.deleteJob(j.id)
    message.success(t('messages.jobDeleted'))
    deleteOpen.value = false
    // Return to jobs list.
    await router.push(`/n/${encodeURIComponent(nodeId.value)}/jobs`)
  } catch (error) {
    message.error(formatToastError(t('errors.deleteJobFailed'), error, t))
  } finally {
    deleteBusy.value = null
  }
}

async function unarchiveJob(): Promise<void> {
  const j = job.value
  if (!j) return
  try {
    await jobs.unarchiveJob(j.id)
    message.success(t('messages.jobUnarchived'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.unarchiveJobFailed'), error, t))
  }
}

const moreOptions = computed(() => {
  const archived = !!job.value?.archived_at
  return [
    { label: t('common.edit'), key: 'edit', disabled: archived },
    { label: t('jobs.actions.deploy'), key: 'deploy', disabled: archived },
    { type: 'divider', key: '__d0' },
    { label: t('common.json'), key: 'inspect' },
    { type: 'divider', key: '__d1' },
    archived
      ? { label: t('jobs.actions.unarchive'), key: 'unarchive' }
      : { label: t('jobs.actions.archive'), key: 'archive', props: { style: 'color: var(--app-warning);' } },
    { label: t('jobs.actions.deletePermanently'), key: 'delete', props: { style: 'color: var(--app-danger);' } },
  ]
})

function onSelectMore(key: string | number): void {
  if (key === 'edit') return void openEdit()
  if (key === 'deploy') return void openDeploy()
  if (key === 'inspect') {
    inspectOpen.value = true
    return
  }
  if (key === 'unarchive') return void unarchiveJob()
  if (key === 'archive') return openArchiveOrDelete()
  if (key === 'delete') return openArchiveOrDelete()
}
</script>

<template>
  <div :class="isDesktop ? 'h-full min-h-0 flex flex-col gap-4' : 'space-y-4'">
    <MobileTopBar v-if="!isDesktop" :title="job?.name ?? t('jobs.detail.title')" :back-to="`/n/${encodeURIComponent(nodeId)}/jobs`" />

    <n-card class="app-card" :bordered="false">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="flex items-center gap-2 min-w-0">
            <div class="text-lg font-semibold truncate">{{ job?.name ?? t('jobs.detail.title') }}</div>
            <n-tag v-if="job?.archived_at" size="small" :bordered="false" type="warning">{{ t('jobs.archived') }}</n-tag>
          </div>
          <div class="mt-1 flex flex-wrap items-center gap-2 text-sm opacity-70">
            <NodeContextTag :node-id="nodeId" />
            <span v-if="job" class="font-mono tabular-nums truncate">{{ job.id }}</span>
            <span v-if="job" class="font-mono tabular-nums">{{ formatUnixSeconds(job.updated_at) }}</span>
          </div>
        </div>

        <div class="flex items-center gap-2 flex-wrap justify-end">
          <n-button size="small" :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
          <n-button size="small" type="primary" :disabled="!!job?.archived_at" @click="runNow">{{ t('jobs.actions.runNow') }}</n-button>

          <template v-if="isDesktop">
            <n-button size="small" :disabled="!!job?.archived_at" @click="openEdit">{{ t('common.edit') }}</n-button>
            <n-button size="small" :disabled="!!job?.archived_at" @click="openDeploy">{{ t('jobs.actions.deploy') }}</n-button>
          </template>

          <n-dropdown trigger="click" :options="moreOptions" @select="onSelectMore">
            <n-button size="small" quaternary>
              <template #icon>
                <EllipsisHorizontal />
              </template>
              {{ t('common.more') }}
            </n-button>
          </n-dropdown>
        </div>
      </div>
    </n-card>

    <n-card class="app-card" :bordered="false">
      <n-tabs :value="activeSection" type="line" size="small" :pane-style="{ display: 'none' }" @update:value="goSection">
        <n-tab-pane name="overview" :tab="t('jobs.workspace.sections.overview')" />
        <n-tab-pane name="history" :tab="t('jobs.workspace.sections.history')" />
        <n-tab-pane name="data" :tab="t('jobs.workspace.sections.data')" />
      </n-tabs>
    </n-card>

    <ScrollShadowPane
      v-if="isDesktop"
      wrapper-class="flex-1 min-h-0"
      data-testid="job-section-scroll"
      shadow-from="var(--app-bg-solid)"
    >
      <router-view v-if="jobId" />
      <AppEmptyState v-else :title="t('common.noData')" />
    </ScrollShadowPane>

    <div v-else>
      <router-view v-if="jobId" />
      <AppEmptyState v-else :title="t('common.noData')" />
    </div>

    <n-modal v-model:show="inspectOpen" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('common.json')">
      <div class="space-y-3">
        <div class="text-sm opacity-70">{{ t('jobs.detail.title') }}</div>
        <n-code :code="jobJson" language="json" class="text-xs" />
      </div>
    </n-modal>

    <JobEditorModal ref="editorModal" @saved="refresh" />
    <JobDeployModal ref="deployModal" />

    <n-modal v-model:show="deleteOpen" preset="card" :style="{ width: MODAL_WIDTH.sm }" :title="t('jobs.deleteTitle')">
      <div class="space-y-3">
        <div class="text-sm opacity-80">
          {{
            job?.archived_at
              ? t('jobs.deletePermanentlyHelp')
              : t('jobs.deleteHelp')
          }}
        </div>

        <div v-if="job && !job.archived_at" class="rounded border border-slate-200/60 dark:border-slate-700/60 p-3 space-y-1">
          <n-checkbox :checked="archiveCascadeSnapshots" @update:checked="(v) => (archiveCascadeSnapshots = v)">
            {{ t('jobs.archiveCascadeLabel') }}
          </n-checkbox>
          <div class="text-xs opacity-70">{{ t('jobs.archiveCascadeHelp') }}</div>
        </div>

        <div class="flex items-center justify-end gap-2">
          <n-button :disabled="deleteBusy !== null" @click="deleteOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button
            v-if="job && !job.archived_at"
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

    <n-drawer v-model:show="runDrawerOpen" placement="right" :width="runDrawerWidth">
      <n-drawer-content :title="t('runs.title')" closable>
        <RunDetailPanel v-if="runId" :node-id="nodeId" :run-id="runId" />
      </n-drawer-content>
    </n-drawer>
  </div>
</template>
