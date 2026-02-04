<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NDataTable, NInput, NRadioButton, NRadioGroup, NSelect, NSwitch, NTag, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import ListToolbar from '@/components/list/ListToolbar.vue'
import ScrollShadowPane from '@/components/scroll/ScrollShadowPane.vue'
import { useJobsStore, type JobListItem, type RunStatus } from '@/stores/jobs'
import { useAgentsStore } from '@/stores/agents'
import { useUiStore, type JobsWorkspaceLayoutMode, type JobsWorkspaceListView } from '@/stores/ui'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatUnixSecondsYmdHm, formatUnixSecondsYmdHms } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { runStatusLabel } from '@/lib/runs'
import JobEditorModal, { type JobEditorModalExpose } from '@/components/jobs/JobEditorModal.vue'

type JobSortKey = 'updated_desc' | 'updated_asc' | 'name_asc' | 'name_desc'

const { t } = useI18n()
const message = useMessage()
const route = useRoute()
const router = useRouter()

const isDesktop = useMediaQuery(MQ.mdUp)

const ui = useUiStore()
const jobs = useJobsStore()
const agents = useAgentsStore()

const nodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : 'hub'))
const selectedJobId = computed(() => (typeof route.params.jobId === 'string' ? route.params.jobId : null))

const editorModal = ref<JobEditorModalExpose | null>(null)

const showArchived = ref<boolean>(false)
const searchText = ref<string>('')
const sortKey = ref<JobSortKey>('updated_desc')
const listLatestStatusFilter = ref<RunStatus | 'never' | 'all'>('all')
const listScheduleFilter = ref<'all' | 'manual' | 'scheduled'>('all')

const sortOptions = computed(() => [
  { label: t('jobs.sort.updatedDesc'), value: 'updated_desc' },
  { label: t('jobs.sort.updatedAsc'), value: 'updated_asc' },
  { label: t('jobs.sort.nameAsc'), value: 'name_asc' },
  { label: t('jobs.sort.nameDesc'), value: 'name_desc' },
])

const layoutMode = computed<JobsWorkspaceLayoutMode>(() => {
  if (!isDesktop.value) return 'split'
  const mode = ui.jobsWorkspaceLayoutMode
  if (mode === 'detail' && !selectedJobId.value) return 'list'
  return mode
})

const layoutModeModel = computed<JobsWorkspaceLayoutMode>({
  get: () => layoutMode.value,
  set: (value) => {
    if (!isDesktop.value) return
    if (value === 'detail' && !selectedJobId.value) return
    ui.setJobsWorkspaceLayoutMode(value)
  },
})

const jobsListView = computed<JobsWorkspaceListView>(() => {
  if (!isDesktop.value) return 'list'
  if (layoutMode.value !== 'list') return 'list'
  return ui.jobsWorkspaceListView
})

const jobsListViewModel = computed<JobsWorkspaceListView>({
  get: () => jobsListView.value,
  set: (value) => {
    // Table view requires full-width list. Selecting it forces list-only layout.
    if (value === 'table') {
      ui.setJobsWorkspaceLayoutMode('list')
    }
    ui.setJobsWorkspaceListView(value)
  },
})

const gridColsClass = computed(() =>
  layoutMode.value === 'split' ? 'md:grid-cols-[minmax(0,360px)_minmax(0,1fr)]' : 'md:grid-cols-1',
)

const nodeScopedJobs = computed<JobListItem[]>(() => {
  const id = nodeId.value
  if (id === 'hub') return jobs.items.filter((j) => j.agent_id === null)
  return jobs.items.filter((j) => j.agent_id === id)
})

const filteredJobs = computed<JobListItem[]>(() => {
  const q = searchText.value.trim().toLowerCase()
  const list = nodeScopedJobs.value.filter((j) => {
    if (!q) return true
    return j.name.toLowerCase().includes(q) || j.id.toLowerCase().includes(q)
  })

  const listModeFiltersEnabled = layoutMode.value === 'list'
  const latestStatus = listLatestStatusFilter.value
  const scheduleMode = listScheduleFilter.value

  const filtered = listModeFiltersEnabled
    ? list.filter((j) => {
        if (latestStatus !== 'all') {
          if (latestStatus === 'never') {
            if (j.latest_run_status != null) return false
          } else if (j.latest_run_status !== latestStatus) {
            return false
          }
        }

        if (scheduleMode !== 'all') {
          if (scheduleMode === 'manual' && j.schedule != null) return false
          if (scheduleMode === 'scheduled' && j.schedule == null) return false
        }

        return true
      })
    : list

  const sorted = filtered.slice()
  sorted.sort((a, b) => {
    if (sortKey.value === 'updated_asc') return a.updated_at - b.updated_at
    if (sortKey.value === 'updated_desc') return b.updated_at - a.updated_at
    if (sortKey.value === 'name_asc') return a.name.localeCompare(b.name)
    if (sortKey.value === 'name_desc') return b.name.localeCompare(a.name)
    return 0
  })
  return sorted
})

async function refresh(): Promise<void> {
  try {
    await jobs.refresh({ includeArchived: showArchived.value })
  } catch (error) {
    message.error(formatToastError(t('errors.fetchJobsFailed'), error, t))
  }
}

function clearFilters(): void {
  searchText.value = ''
  showArchived.value = false
  sortKey.value = 'updated_desc'
  listLatestStatusFilter.value = 'all'
  listScheduleFilter.value = 'all'
}

function openCreate(): void {
  editorModal.value?.openCreate({ nodeId: nodeId.value })
}

async function openEdit(jobId: string): Promise<void> {
  await editorModal.value?.openEdit(jobId, { nodeId: nodeId.value })
}

async function runNow(jobId: string): Promise<void> {
  try {
    const res = await jobs.runNow(jobId)
    if (res.status === 'rejected') message.warning(t('messages.runRejected'))
    else message.success(t('messages.runQueued'))
  } catch (error) {
    message.error(formatToastError(t('errors.runNowFailed'), error, t))
  }
}

function openJob(jobId: string): void {
  void router.push(`/n/${encodeURIComponent(nodeId.value)}/jobs/${encodeURIComponent(jobId)}/overview`)
}

function isSelected(jobId: string): boolean {
  return selectedJobId.value === jobId
}

function formatNodeLabel(agentId: string | null): string {
  if (!agentId) return t('jobs.nodes.hub')
  const agent = agents.items.find((a) => a.id === agentId)
  return agent?.name ?? agentId
}

function runStatusTagType(status: RunStatus): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'rejected') return 'warning'
  return 'default'
}

function formatScheduleLabel(job: JobListItem): string {
  return job.schedule ?? t('jobs.scheduleMode.manual')
}

const latestStatusFilterOptions = computed(() => [
  { label: t('runs.filters.all'), value: 'all' },
  { label: t('runs.neverRan'), value: 'never' },
  { label: runStatusLabel(t, 'success'), value: 'success' },
  { label: runStatusLabel(t, 'failed'), value: 'failed' },
  { label: runStatusLabel(t, 'running'), value: 'running' },
  { label: runStatusLabel(t, 'queued'), value: 'queued' },
  { label: runStatusLabel(t, 'rejected'), value: 'rejected' },
])

const scheduleFilterOptions = computed(() => [
  { label: t('runs.filters.all'), value: 'all' },
  { label: t('jobs.scheduleMode.manual'), value: 'manual' },
  { label: t('jobs.workspace.filters.scheduled'), value: 'scheduled' },
])

const tableColumns = computed<DataTableColumns<JobListItem>>(() => [
  {
    title: t('jobs.columns.name'),
    key: 'name',
    render: (row) =>
      h('div', { class: 'min-w-0' }, [
        h('div', { class: 'flex items-center gap-2 min-w-0' }, [
          h(
            'button',
            {
              type: 'button',
              class: 'text-left font-medium truncate hover:underline',
              title: row.name,
              onClick: () => openJob(row.id),
            },
            row.name,
          ),
          row.archived_at
            ? h(NTag, { size: 'small', bordered: false, type: 'warning' }, { default: () => t('jobs.archived') })
            : null,
        ]),
      ]),
  },
  {
    title: t('jobs.columns.node'),
    key: 'node',
    render: (row) =>
      h(
        NTag,
        { size: 'small', bordered: false, type: row.agent_id ? 'default' : 'info' },
        { default: () => formatNodeLabel(row.agent_id) },
      ),
  },
  {
    title: t('jobs.columns.schedule'),
    key: 'schedule',
    render: (row) => {
      const schedule = formatScheduleLabel(row)
      return h('div', { class: 'min-w-0' }, [
        h('div', { class: 'font-mono tabular-nums truncate', title: schedule }, schedule),
        row.schedule
          ? h('div', { class: 'text-xs app-text-muted font-mono tabular-nums truncate' }, row.schedule_timezone)
          : null,
      ])
    },
  },
  {
    title: t('runs.columns.status'),
    key: 'latest_run_status',
    render: (row) =>
      row.latest_run_status
        ? h(
            NTag,
            { size: 'small', bordered: false, type: runStatusTagType(row.latest_run_status) },
            { default: () => runStatusLabel(t, row.latest_run_status!) },
          )
        : h(NTag, { size: 'small', bordered: false }, { default: () => t('runs.neverRan') }),
  },
  {
    title: t('dashboard.recent.columns.startedAt'),
    key: 'latest_run_started_at',
    render: (row) =>
      h(
        'span',
        {
          class: 'font-mono tabular-nums text-xs',
          title: row.latest_run_started_at != null ? formatUnixSecondsYmdHms(row.latest_run_started_at) : '-',
        },
        row.latest_run_started_at != null ? formatUnixSecondsYmdHm(row.latest_run_started_at) : '-',
      ),
  },
  {
    title: t('jobs.columns.updatedAt'),
    key: 'updated_at',
    render: (row) =>
      h(
        'span',
        { class: 'font-mono tabular-nums text-xs', title: formatUnixSecondsYmdHms(row.updated_at) },
        formatUnixSecondsYmdHm(row.updated_at),
      ),
  },
  {
    title: t('jobs.columns.actions'),
    key: 'actions',
    render: (row) =>
      h('div', { class: 'flex items-center gap-2 justify-end' }, [
        h(
          NButton,
          { size: 'small', disabled: !!row.archived_at, onClick: () => void runNow(row.id) },
          { default: () => t('jobs.actions.runNow') },
        ),
        h(
          NButton,
          { size: 'small', disabled: !!row.archived_at, onClick: () => void openEdit(row.id) },
          { default: () => t('common.edit') },
        ),
      ]),
  },
])

onMounted(async () => {
  await refresh()
  try {
    // Ensure node context labels are friendly (agent name vs id).
    await agents.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  }
})

watch(showArchived, () => void refresh())
</script>

<template>
  <div class="flex flex-col gap-6 h-full min-h-0">
    <PageHeader
      v-if="isDesktop || !selectedJobId"
      :title="t('jobs.title')"
      :subtitle="t('jobs.subtitle')"
    >
      <template #titleSuffix>
        <NodeContextTag :node-id="nodeId" />
      </template>

      <template v-if="isDesktop">
        <n-radio-group v-model:value="layoutModeModel" size="small" class="shrink-0">
          <n-radio-button value="list">{{ t('jobs.workspace.actions.fullList') }}</n-radio-button>
          <n-radio-button value="split">{{ t('jobs.workspace.actions.splitView') }}</n-radio-button>
          <n-radio-button value="detail" :disabled="!selectedJobId">{{ t('jobs.workspace.actions.fullDetail') }}</n-radio-button>
        </n-radio-group>

        <n-radio-group v-model:value="jobsListViewModel" size="small" class="shrink-0">
          <n-radio-button value="list">{{ t('jobs.workspace.views.list') }}</n-radio-button>
          <n-radio-button value="table">{{ t('jobs.workspace.views.table') }}</n-radio-button>
        </n-radio-group>
      </template>

      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button type="primary" @click="openCreate">{{ t('jobs.actions.create') }}</n-button>
    </PageHeader>

    <template v-if="isDesktop">
      <div class="grid grid-cols-1 gap-4 flex-1 min-h-0" :class="gridColsClass">
        <n-card
          v-if="layoutMode !== 'detail'"
          class="app-card flex flex-col min-h-0"
          :bordered="false"
        >
          <ListToolbar compact embedded :stacked="layoutMode === 'split'">
            <template #search>
              <n-input
                v-model:value="searchText"
                size="small"
                clearable
                :placeholder="t('jobs.filters.searchPlaceholder')"
              />
            </template>

            <template #filters>
              <div class="shrink-0 flex items-center gap-2 whitespace-nowrap h-7">
                <span class="text-sm app-text-muted">{{ t('jobs.showArchived') }}</span>
                <n-switch v-model:value="showArchived" />
              </div>

              <div v-if="layoutMode === 'list'" class="shrink-0 flex items-center gap-2 whitespace-nowrap">
                <span class="text-sm app-text-muted">{{ t('runs.columns.status') }}</span>
                <n-select
                  v-model:value="listLatestStatusFilter"
                  size="small"
                  :options="latestStatusFilterOptions"
                  :consistent-menu-width="false"
                  class="min-w-[8rem]"
                />
              </div>

              <div v-if="layoutMode === 'list'" class="shrink-0 flex items-center gap-2 whitespace-nowrap">
                <span class="text-sm app-text-muted">{{ t('jobs.columns.schedule') }}</span>
                <n-select
                  v-model:value="listScheduleFilter"
                  size="small"
                  :options="scheduleFilterOptions"
                  :consistent-menu-width="false"
                  class="min-w-[8rem]"
                />
              </div>
            </template>

            <template #sort>
              <div class="w-full md:w-56 md:flex-none">
                <n-select v-model:value="sortKey" size="small" :options="sortOptions" />
              </div>
            </template>

            <template #actions>
              <n-button size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
            </template>
          </ListToolbar>

          <div class="mt-3 flex-1 min-h-0">
            <AppEmptyState v-if="jobs.loading && filteredJobs.length === 0" :title="t('common.loading')" loading />
            <AppEmptyState
              v-else-if="!jobs.loading && filteredJobs.length === 0"
              :title="jobs.items.length === 0 ? t('jobs.empty.title') : t('common.noData')"
              :description="jobs.items.length === 0 ? t('jobs.empty.description') : undefined"
            >
              <template #actions>
                <n-button v-if="jobs.items.length === 0" type="primary" size="small" @click="openCreate">
                  {{ t('jobs.actions.create') }}
                </n-button>
                <n-button v-else size="small" @click="clearFilters">
                  {{ t('common.clear') }}
                </n-button>
              </template>
            </AppEmptyState>

            <div
              v-else
            >
              <ScrollShadowPane
                data-testid="jobs-list-scroll"
                :class="jobsListView === 'list' ? 'app-divide-y' : ''"
              >
                <template v-if="jobsListView === 'table'">
                  <div class="py-2">
                    <n-data-table class="app-picker-table" size="small" :columns="tableColumns" :data="filteredJobs" :scroll-x="1100" />
                  </div>
                </template>

                <template v-else>
                  <button
                    v-for="job in filteredJobs"
                    :key="job.id"
                    type="button"
                    class="app-list-row"
                    :class="isSelected(job.id) ? 'bg-[var(--app-primary-soft)]' : ''"
                    @click="openJob(job.id)"
                  >
                    <div class="min-w-0">
                      <div class="flex items-center gap-2 min-w-0">
                        <div class="font-medium truncate">{{ job.name }}</div>
                        <n-tag v-if="job.archived_at" size="small" :bordered="false" type="warning">
                          {{ t('jobs.archived') }}
                        </n-tag>
                      </div>
                      <div class="mt-1 flex items-center gap-2 min-w-0 text-xs app-text-muted">
                        <n-tag size="small" :bordered="false" :type="job.agent_id ? 'default' : 'info'">
                          {{ formatNodeLabel(job.agent_id) }}
                        </n-tag>
                        <span class="min-w-0 truncate">{{ formatScheduleLabel(job) }}</span>
                      </div>
                    </div>

                    <div class="shrink-0 flex flex-col items-end gap-1 text-right">
                      <n-tag
                        v-if="job.latest_run_status"
                        size="small"
                        :bordered="false"
                        :type="runStatusTagType(job.latest_run_status)"
                      >
                        {{ runStatusLabel(t, job.latest_run_status) }}
                      </n-tag>
                      <n-tag v-else size="small" :bordered="false">
                        {{ t('runs.neverRan') }}
                      </n-tag>

                      <div
                        v-if="job.latest_run_started_at != null"
                        class="text-xs font-mono tabular-nums app-text-muted max-w-[10rem] truncate"
                        :title="formatUnixSecondsYmdHms(job.latest_run_started_at)"
                      >
                        {{ formatUnixSecondsYmdHm(job.latest_run_started_at) }}
                      </div>
                    </div>
                  </button>
                </template>
              </ScrollShadowPane>
            </div>
          </div>
        </n-card>

        <div v-if="layoutMode !== 'list'" class="min-w-0 min-h-0 flex flex-col">
          <div v-if="selectedJobId" class="flex-1 min-h-0">
            <router-view />
          </div>
          <AppEmptyState
            v-else
            :title="t('jobs.workspace.emptyTitle')"
            :description="t('jobs.workspace.emptyDescription')"
          />
        </div>
      </div>
    </template>

    <template v-else>
      <div v-if="!selectedJobId" class="space-y-4">
        <ListToolbar>
          <template #search>
            <n-input
              v-model:value="searchText"
              size="small"
              clearable
              :placeholder="t('jobs.filters.searchPlaceholder')"
            />
          </template>

          <template #filters>
            <div class="flex items-center gap-2 w-full md:w-auto">
              <span class="text-sm app-text-muted">{{ t('jobs.showArchived') }}</span>
              <n-switch v-model:value="showArchived" />
            </div>
          </template>

          <template #sort>
            <div class="w-full md:w-56 md:flex-none">
              <n-select v-model:value="sortKey" size="small" :options="sortOptions" />
            </div>
          </template>

          <template #actions>
            <n-button size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
          </template>
        </ListToolbar>

        <AppEmptyState v-if="jobs.loading && filteredJobs.length === 0" :title="t('common.loading')" loading />
        <AppEmptyState
          v-else-if="!jobs.loading && filteredJobs.length === 0"
          :title="jobs.items.length === 0 ? t('jobs.empty.title') : t('common.noData')"
          :description="jobs.items.length === 0 ? t('jobs.empty.description') : undefined"
        >
          <template #actions>
            <n-button v-if="jobs.items.length === 0" type="primary" size="small" @click="openCreate">
              {{ t('jobs.actions.create') }}
            </n-button>
            <n-button v-else size="small" @click="clearFilters">
              {{ t('common.clear') }}
            </n-button>
          </template>
        </AppEmptyState>

        <n-card v-else class="app-card" :bordered="false">
          <div class="app-divide-y">
            <button
              v-for="job in filteredJobs"
              :key="job.id"
              type="button"
              class="app-list-row"
              @click="openJob(job.id)"
            >
              <div class="min-w-0">
                <div class="flex items-center gap-2 min-w-0">
                  <div class="font-medium truncate">{{ job.name }}</div>
                  <n-tag v-if="job.archived_at" size="small" :bordered="false" type="warning">
                    {{ t('jobs.archived') }}
                  </n-tag>
                </div>
                <div class="mt-1 flex items-center gap-2 min-w-0 text-xs app-text-muted">
                  <n-tag size="small" :bordered="false" :type="job.agent_id ? 'default' : 'info'">
                    {{ formatNodeLabel(job.agent_id) }}
                  </n-tag>
                  <span class="min-w-0 truncate">{{ job.schedule ?? t('jobs.scheduleMode.manual') }}</span>
                </div>
              </div>

              <div class="shrink-0 flex flex-col items-end gap-1 text-right">
                <n-tag
                  v-if="job.latest_run_status"
                  size="small"
                  :bordered="false"
                  :type="runStatusTagType(job.latest_run_status)"
                >
                  {{ runStatusLabel(t, job.latest_run_status) }}
                </n-tag>
                <n-tag v-else size="small" :bordered="false">
                  {{ t('runs.neverRan') }}
                </n-tag>

                <div
                  v-if="job.latest_run_started_at != null"
                  class="text-xs font-mono tabular-nums app-text-muted max-w-[10rem] truncate"
                  :title="formatUnixSecondsYmdHms(job.latest_run_started_at)"
                >
                  {{ formatUnixSecondsYmdHm(job.latest_run_started_at) }}
                </div>
              </div>
            </button>
          </div>
        </n-card>
      </div>

      <router-view v-else />
    </template>

    <JobEditorModal ref="editorModal" @saved="refresh" />
  </div>
</template>
