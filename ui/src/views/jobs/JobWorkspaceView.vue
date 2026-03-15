<script setup lang="ts">
import { computed, provide, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NCheckbox, NCode, NDrawer, NDrawerContent, NDropdown, NTabPane, NTabs, NTag, useMessage } from 'naive-ui'
import { EllipsisHorizontal, PlayOutline, RefreshOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import NodeContextTag from '@/components/NodeContextTag.vue'
import MobileTopBar from '@/components/MobileTopBar.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import AppModalShell from '@/components/AppModalShell.vue'
import JobDeployModal, { type JobDeployModalExpose } from '@/components/jobs/JobDeployModal.vue'
import JobWorkspaceSupportPane from '@/components/jobs/JobWorkspaceSupportPane.vue'
import RunDetailPanel from '@/components/runs/RunDetailPanel.vue'
import ScrollShadowPane from '@/components/scroll/ScrollShadowPane.vue'
import { useJobsStore, type JobDetail, type JobWorkspaceDetail, type RunStatus } from '@/stores/jobs'
import { useUiStore } from '@/stores/ui'
import { formatCommandCenterScopeLabel } from '@/lib/commandCenterPresentation'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { MODAL_WIDTH } from '@/lib/modal'
import { MQ } from '@/lib/breakpoints'
import { useMediaQuery } from '@/lib/media'
import { JOB_DETAIL_CONTEXT } from '@/lib/jobDetailContext'
import {
  buildJobEditorLocation,
  buildJobSectionPath,
  buildJobsCollectionLocation,
  buildJobsCollectionQuery,
  readJobsCollectionState,
  resolveJobsScope,
} from '@/lib/jobsRoute'
import { buildRunDetailLocation, runStatusLabel } from '@/lib/runs'
import { scopeToNodeId } from '@/lib/scope'

type SectionKey = 'overview' | 'history' | 'data'

const { t } = useI18n()
const message = useMessage()
const route = useRoute()
const router = useRouter()
const ui = useUiStore()
const jobs = useJobsStore()

const isDesktop = useMediaQuery(MQ.mdUp)
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const jobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))
const runId = computed(() => (typeof route.params.runId === 'string' ? route.params.runId : null))
const collectionState = computed(() => readJobsCollectionState(route.query, resolveJobsScope(route, ui.preferredScope)))
const scopeNodeId = computed(() => scopeToNodeId(collectionState.value.scope))
const jobsListPath = computed(() => router.resolve(buildJobsCollectionLocation(collectionState.value)).fullPath)
const scopeLabel = computed(() => formatCommandCenterScopeLabel(collectionState.value.scope, t))
const latestRunId = computed(() => job.value?.latest_run_id ?? workspace.value?.recent_runs[0]?.id ?? null)
const selectedSavedViewLabel = computed<string | null>(() => {
  const viewId = collectionState.value.view
  if (!viewId) return null
  if (viewId === 'failed-recently') return t('jobs.savedViews.builtins.failedRecently')
  if (viewId === 'manual-jobs') return t('jobs.savedViews.builtins.manualJobs')
  if (viewId === 'archived') return t('jobs.savedViews.builtins.archived')
  return ui.jobsSavedViews.find((item) => item.id === viewId)?.name ?? viewId
})
const collectionContextTags = computed<Array<{ key: string; label: string }>>(() => {
  const tags: Array<{ key: string; label: string }> = [
    { key: 'scope', label: t('jobs.detail.context.scope', { scope: scopeLabel.value }) },
  ]
  if (selectedSavedViewLabel.value) {
    tags.push({ key: 'view', label: t('jobs.detail.context.view', { name: selectedSavedViewLabel.value }) })
  }
  if (collectionState.value.q) {
    tags.push({ key: 'search', label: `${t('common.search')}: ${collectionState.value.q}` })
  }
  if (collectionState.value.status !== 'all') {
    const statusLabel = collectionState.value.status === 'never'
      ? t('runs.neverRan')
      : runStatusLabel(t, collectionState.value.status as RunStatus)
    tags.push({ key: 'status', label: `${t('runs.columns.status')}: ${statusLabel}` })
  }
  if (collectionState.value.schedule !== 'all') {
    const scheduleLabel = collectionState.value.schedule === 'manual'
      ? t('jobs.scheduleMode.manual')
      : t('jobs.workspace.filters.scheduled')
    tags.push({ key: 'schedule', label: `${t('jobs.columns.schedule')}: ${scheduleLabel}` })
  }
  if (collectionState.value.includeArchived) {
    tags.push({ key: 'archived', label: t('jobs.showArchived') })
  }
  return tags
})

const loading = ref<boolean>(false)
const job = ref<JobDetail | null>(null)
const workspace = ref<JobWorkspaceDetail | null>(null)
const nodeId = computed(() => job.value?.agent_id ?? scopeNodeId.value ?? 'hub')

async function refresh(): Promise<void> {
  const id = jobId.value
  if (!id) return
  loading.value = true
  try {
    workspace.value = await jobs.getJobWorkspace(id)
    job.value = workspace.value.job
  } catch (error) {
    workspace.value = null
    job.value = null
    message.error(formatToastError(t('errors.fetchJobFailed'), error, t))
  } finally {
    loading.value = false
  }
}

watch(jobId, () => void refresh(), { immediate: true })

provide(JOB_DETAIL_CONTEXT, { nodeId, jobId, job, workspace, loading, refresh })

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
  void router.push({
    path: buildJobSectionPath(id, key),
    query: buildJobsCollectionQuery(collectionState.value),
  })
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

const deployModal = ref<JobDeployModalExpose | null>(null)

async function openEdit(): Promise<void> {
  const id = jobId.value
  if (!id) return
  await router.push(buildJobEditorLocation('edit', { jobId: id, collection: collectionState.value }))
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
    await router.push(jobsListPath.value)
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

function openRun(runId: string): void {
  const id = jobId.value
  if (!id) return
  void router.push(
    buildRunDetailLocation(runId, {
      fromScope: collectionState.value.scope,
      fromJob: id,
      fromSection: activeSection.value,
    }),
  )
}

function latestRunStatusTagType(state: JobWorkspaceDetail['readiness']['state'] | null | undefined): 'success' | 'warning' | 'error' | 'default' {
  if (state === 'healthy') return 'success'
  if (state === 'warning') return 'warning'
  if (state === 'critical') return 'error'
  return 'default'
}

function latestRunStatusLabel(state: JobWorkspaceDetail['readiness']['state'] | null | undefined): string {
  if (state === 'healthy') return t('jobs.workspace.support.healthHealthy')
  if (state === 'warning') return t('jobs.workspace.support.healthWarning')
  if (state === 'critical') return t('jobs.workspace.support.healthCritical')
  return t('jobs.archived')
}

function openLatestRun(): void {
  const id = latestRunId.value
  if (!id) return
  openRun(id)
}
</script>

<template>
  <div :class="isDesktop ? 'h-full min-h-0 flex flex-col gap-4' : 'space-y-4'">
    <MobileTopBar
      v-if="!isDesktop"
      sticky
      :title="job?.name ?? t('jobs.detail.title')"
      :back-to="jobsListPath"
    >
      <template #actions>
        <n-button
          size="small"
          quaternary
          :disabled="!job || !!job.archived_at"
          :title="t('jobs.actions.runNow')"
          :aria-label="t('jobs.actions.runNow')"
          @click="runNow"
        >
          <template #icon>
            <PlayOutline />
          </template>
        </n-button>

        <n-button
          size="small"
          quaternary
          :loading="loading"
          :title="t('jobs.workspace.actions.refreshJob')"
          :aria-label="t('jobs.workspace.actions.refreshJob')"
          @click="refresh"
        >
          <template #icon>
            <RefreshOutline />
          </template>
        </n-button>

        <n-dropdown trigger="click" :options="moreOptions" @select="onSelectMore">
          <n-button
            size="small"
            quaternary
            :title="t('common.more')"
            :aria-label="t('common.more')"
          >
            <template #icon>
              <EllipsisHorizontal />
            </template>
          </n-button>
        </n-dropdown>
      </template>
    </MobileTopBar>

    <div v-if="!isDesktop && job" class="grid grid-cols-2 gap-2">
      <n-button type="primary" :disabled="!!job.archived_at" @click="runNow">
        {{ t('jobs.actions.runNow') }}
      </n-button>
      <n-button :disabled="!!job.archived_at" @click="openEdit">
        {{ t('common.edit') }}
      </n-button>
      <n-button v-if="latestRunId" @click="openLatestRun">
        {{ t('jobs.workspace.support.openLatestRun') }}
      </n-button>
      <n-button :loading="loading" @click="refresh">
        {{ t('jobs.workspace.actions.refreshJob') }}
      </n-button>
      <n-button :disabled="!!job.archived_at" @click="openDeploy">
        {{ t('jobs.actions.deploy') }}
      </n-button>
    </div>

    <div v-if="job" class="flex items-center gap-2 flex-wrap" data-testid="job-workspace-context-row">
      <n-button size="small" quaternary @click="void router.push(jobsListPath)">
        {{ t('jobs.workspace.actions.backToList') }}
      </n-button>
      <n-tag
        v-for="tag in collectionContextTags"
        :key="tag.key"
        size="small"
        :bordered="false"
      >
        {{ tag.label }}
      </n-tag>
    </div>

    <n-card class="app-card" :bordered="false" data-testid="job-workspace-object-header">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="text-xs uppercase tracking-[0.16em] app-text-muted">
            {{ t('jobs.detail.kicker') }}
          </div>
          <div class="flex items-center gap-2 min-w-0">
            <div class="text-lg font-semibold truncate">{{ job?.name ?? t('jobs.detail.title') }}</div>
            <n-tag
              v-if="workspace"
              size="small"
              :bordered="false"
              :type="latestRunStatusTagType(workspace.readiness.state)"
            >
              {{ latestRunStatusLabel(workspace.readiness.state) }}
            </n-tag>
            <n-tag v-if="job?.archived_at" size="small" :bordered="false" type="warning">{{ t('jobs.archived') }}</n-tag>
          </div>
          <div class="mt-1 flex flex-wrap items-center gap-2 text-sm app-text-muted">
            <NodeContextTag :node-id="nodeId" />
            <span v-if="job" class="font-mono tabular-nums truncate">{{ job.id }}</span>
            <span v-if="job" class="font-mono tabular-nums">{{ formatUnixSeconds(job.updated_at) }}</span>
          </div>
        </div>

        <div v-if="isDesktop" class="flex items-center gap-2 flex-wrap justify-end">
          <n-button size="small" type="primary" :disabled="!!job?.archived_at" @click="runNow">{{ t('jobs.actions.runNow') }}</n-button>
          <n-button size="small" :disabled="!!job?.archived_at" @click="openEdit">{{ t('common.edit') }}</n-button>
          <n-button v-if="latestRunId" size="small" @click="openLatestRun">
            {{ t('jobs.workspace.support.openLatestRun') }}
          </n-button>
          <n-button
            size="small"
            :loading="loading"
            :title="t('jobs.workspace.actions.refreshJob')"
            :aria-label="t('jobs.workspace.actions.refreshJob')"
            @click="refresh"
          >
            {{ t('jobs.workspace.actions.refreshJob') }}
          </n-button>
          <n-button size="small" :disabled="!!job?.archived_at" @click="openDeploy">{{ t('jobs.actions.deploy') }}</n-button>

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

    <div :class="isDesktop ? 'grid min-h-0 flex-1 gap-4 xl:grid-cols-[minmax(0,1fr)_320px]' : 'space-y-4'">
      <n-card class="app-card" :class="isDesktop ? 'min-h-0 flex flex-col' : ''" :bordered="false">
        <div class="app-tabs-embedded">
          <n-tabs :value="activeSection" type="line" size="small" :pane-style="{ display: 'none' }" @update:value="goSection">
            <n-tab-pane name="overview" :tab="t('jobs.workspace.sections.overview')" />
            <n-tab-pane name="history" :tab="t('jobs.workspace.sections.history')" />
            <n-tab-pane name="data" :tab="t('jobs.workspace.sections.data')" />
          </n-tabs>
        </div>

        <ScrollShadowPane
          v-if="isDesktop"
          wrapper-class="flex-1 min-h-0"
          class="app-embedded-card-stack app-tab-shell-body"
          data-testid="job-section-scroll"
          shadow-from="var(--app-surface)"
        >
          <router-view v-if="jobId" />
          <AppEmptyState v-else :title="t('common.noData')" />
        </ScrollShadowPane>

        <div v-else class="app-embedded-card-stack app-tab-shell-body">
          <router-view v-if="jobId" />
          <AppEmptyState v-else :title="t('common.noData')" />
        </div>
      </n-card>

      <div :class="isDesktop ? 'min-h-0' : ''">
        <ScrollShadowPane
          v-if="isDesktop"
          wrapper-class="h-full min-h-0"
          class="pr-1"
          shadow-from="var(--app-bg)"
        >
          <JobWorkspaceSupportPane :workspace="workspace" :loading="loading" @open-run="openRun" />
        </ScrollShadowPane>
        <JobWorkspaceSupportPane v-else :workspace="workspace" :loading="loading" @open-run="openRun" />
      </div>
    </div>

    <AppModalShell
      v-model:show="inspectOpen"
      :width="MODAL_WIDTH.lg"
      :title="t('common.json')"
    >
      <div class="text-sm app-text-muted">{{ t('jobs.detail.title') }}</div>
      <n-code :code="jobJson" language="json" class="text-xs" />
    </AppModalShell>

    <JobDeployModal ref="deployModal" />

    <AppModalShell
      v-model:show="deleteOpen"
      :width="MODAL_WIDTH.sm"
      :title="t('jobs.deleteTitle')"
    >
      <div class="text-sm app-text-muted">
        {{
          job?.archived_at
            ? t('jobs.deletePermanentlyHelp')
            : t('jobs.deleteHelp')
        }}
      </div>

      <div v-if="job && !job.archived_at" class="rounded app-panel-inset p-3 space-y-1">
        <n-checkbox :checked="archiveCascadeSnapshots" @update:checked="(v) => (archiveCascadeSnapshots = v)">
          {{ t('jobs.archiveCascadeLabel') }}
        </n-checkbox>
        <div class="text-xs app-text-muted">{{ t('jobs.archiveCascadeHelp') }}</div>
      </div>

      <template #footer>
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
      </template>
    </AppModalShell>

    <n-drawer v-model:show="runDrawerOpen" placement="right" :width="runDrawerWidth">
      <n-drawer-content :title="t('runs.title')" closable>
        <RunDetailPanel v-if="runId" :node-id="nodeId" :run-id="runId" />
      </n-drawer-content>
    </n-drawer>
  </div>
</template>
