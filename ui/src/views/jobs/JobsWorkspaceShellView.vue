<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NCheckbox, NDataTable, NInput, NRadioButton, NRadioGroup, useMessage, type DropdownOption } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import AppModalShell from '@/components/AppModalShell.vue'
import AppPagination from '@/components/list/AppPagination.vue'
import ListPageScaffold from '@/components/list/ListPageScaffold.vue'
import SelectionToolbar from '@/components/list/SelectionToolbar.vue'
import ListToolbar from '@/components/list/ListToolbar.vue'
import ListFilterSummaryRow from '@/components/list/ListFilterSummaryRow.vue'
import ListStatePresenter from '@/components/list/ListStatePresenter.vue'
import ScrollShadowPane from '@/components/scroll/ScrollShadowPane.vue'
import PickerFiltersPopoverDrawer from '@/components/pickers/PickerFiltersPopoverDrawer.vue'
import JobsListRowItem from './JobsListRowItem.vue'
import { useJobsStore, type JobListItem, type RunStatus } from '@/stores/jobs'
import { useAgentsStore } from '@/stores/agents'
import { useUiStore, type JobsWorkspaceLayoutMode, type JobsWorkspaceListView } from '@/stores/ui'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatUnixSecondsYmdHm, formatUnixSecondsYmdHms } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { createDebouncedTask } from '@/lib/asyncControl'
import { useIdBusyState } from '@/lib/idBusyState'
import {
  buildListRangeSummary,
  DEFAULT_LIST_PAGE_SIZE,
  LIST_PAGE_SIZE_OPTIONS,
  LIST_QUERY_DEBOUNCE_MS,
} from '@/lib/listUi'
import { runStatusLabel } from '@/lib/runs'
import { nodeScopedPath } from '@/lib/nodeRoute'
import JobEditorModal, { type JobEditorModalExpose } from '@/components/jobs/JobEditorModal.vue'
import JobsFiltersPanel from './JobsFiltersPanel.vue'
import { useJobsFilters } from './useJobsFilters'
import { useJobsTableColumns } from './useJobsTableColumns'
import { useSplitWorkspaceResize } from './useSplitWorkspaceResize'

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

const filtersPopoverOpen = ref<boolean>(false)
const filtersDrawerOpen = ref<boolean>(false)

const jobsPage = ref<number>(1)
const jobsPageSize = ref<number>(DEFAULT_LIST_PAGE_SIZE)
const jobsPageSizeOptions = [...LIST_PAGE_SIZE_OPTIONS]

const {
  showArchived,
  searchText,
  sortKey,
  latestStatusFilter,
  scheduleFilter,
  sortOptions,
  latestStatusFilterOptions,
  scheduleFilterOptions,
  filtersActiveCount,
  activeFilterChips,
  hasActiveFilters,
  clearFilters,
  applyRouteQuery,
} = useJobsFilters(t)

applyRouteQuery(route.query as Record<string, unknown>)

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

const persistedSplitListWidthPx = computed<number>(() => ui.jobsWorkspaceSplitListWidthPx)
const {
  splitGridEl,
  splitResizeActive,
  gridStyle,
  onSplitResizePointerDown,
  cleanupSplitResize,
} = useSplitWorkspaceResize({
  layoutMode,
  persistedListWidthPx: persistedSplitListWidthPx,
  setPersistedListWidthPx: (next) => ui.setJobsWorkspaceSplitListWidthPx(next),
})

const pagedFilteredJobs = computed<JobListItem[]>(() => jobs.items)
const nodeScopedJobs = computed<JobListItem[]>(() => jobs.items)
const jobsVisibleCount = computed<number>(() => pagedFilteredJobs.value.length)
const jobsRangeSummary = computed(() => buildListRangeSummary(jobs.total, jobsPage.value, jobsPageSize.value))
const jobsPaginationLabel = computed(() => t('common.paginationRange', jobsRangeSummary.value))
const jobsResultsLabel = computed(() => t('jobs.workspace.filters.resultsCount', { visible: jobsVisibleCount.value, filtered: jobs.total }))

const listBaseEmpty = computed<boolean>(() => jobs.total === 0 && !hasActiveFilters.value)

const selectedJobIds = ref<string[]>([])
const selectedJobArchived = ref<Record<string, boolean>>({})
const listSelectMode = ref<boolean>(false)
const rowRunNowBusy = useIdBusyState<string>()

const nodeJobsById = computed(() => new Map(nodeScopedJobs.value.map((j) => [j.id, j])))

function syncSelectedJobArchived(): void {
  if (selectedJobIds.value.length === 0) {
    if (Object.keys(selectedJobArchived.value).length > 0) {
      selectedJobArchived.value = {}
    }
    return
  }

  const nextArchived: Record<string, boolean> = {}
  for (const id of selectedJobIds.value) {
    const current = nodeJobsById.value.get(id)
    if (current) {
      nextArchived[id] = !!current.archived_at
      continue
    }
    if (Object.prototype.hasOwnProperty.call(selectedJobArchived.value, id)) {
      nextArchived[id] = selectedJobArchived.value[id] === true
    }
  }
  selectedJobArchived.value = nextArchived
}

const selectedJobs = computed<Array<{ id: string; archived: boolean }>>(() =>
  selectedJobIds.value
    .map((id) => {
      const current = nodeJobsById.value.get(id)
      if (current) {
        return { id, archived: !!current.archived_at }
      }
      if (Object.prototype.hasOwnProperty.call(selectedJobArchived.value, id)) {
        return { id, archived: selectedJobArchived.value[id] === true }
      }
      return null
    })
    .filter((v): v is { id: string; archived: boolean } => !!v),
)
const selectedActiveJobs = computed(() => selectedJobs.value.filter((j) => !j.archived))
const selectedArchivedJobs = computed(() => selectedJobs.value.filter((j) => j.archived))

function setJobSelected(jobId: string, checked: boolean): void {
  const next = new Set(selectedJobIds.value)
  if (checked) next.add(jobId)
  else next.delete(jobId)
  selectedJobIds.value = [...next]
  syncSelectedJobArchived()
}

function clearSelectedJobs(): void {
  selectedJobIds.value = []
  selectedJobArchived.value = {}
}

watch(selectedJobIds, syncSelectedJobArchived)
watch(nodeScopedJobs, syncSelectedJobArchived)

watch(jobsListView, () => {
  if (jobsListView.value !== 'list') {
    listSelectMode.value = false
  }
})

const bulkConfirmOpen = ref<boolean>(false)
const bulkConfirmKind = ref<'archive' | 'unarchive' | null>(null)
const bulkBusy = ref<'run' | 'archive' | 'unarchive' | null>(null)
const bulkArchiveCascadeSnapshots = ref<boolean>(false)

const bulkConfirmTitle = computed(() => {
  if (bulkConfirmKind.value === 'archive') return t('jobs.workspace.bulk.archiveTitle')
  if (bulkConfirmKind.value === 'unarchive') return t('jobs.workspace.bulk.unarchiveTitle')
  return ''
})

const bulkConfirmBody = computed(() => {
  if (bulkConfirmKind.value === 'archive') {
    return t('jobs.workspace.bulk.archiveConfirm', {
      total: selectedJobIds.value.length,
      eligible: selectedActiveJobs.value.length,
    })
  }
  if (bulkConfirmKind.value === 'unarchive') {
    return t('jobs.workspace.bulk.unarchiveConfirm', {
      total: selectedJobIds.value.length,
      eligible: selectedArchivedJobs.value.length,
    })
  }
  return ''
})

function openBulkConfirm(kind: 'archive' | 'unarchive'): void {
  bulkConfirmKind.value = kind
  bulkArchiveCascadeSnapshots.value = false
  bulkConfirmOpen.value = true
}

async function bulkRunNow(): Promise<void> {
  if (selectedJobIds.value.length === 0) return
  const active = selectedActiveJobs.value
  const skipped = selectedJobIds.value.length - active.length
  bulkBusy.value = 'run'
  try {
    const results = await Promise.allSettled(active.map((j) => jobs.runNow(j.id)))
    let queued = 0
    let rejected = 0
    let failed = 0
    for (const r of results) {
      if (r.status === 'fulfilled') {
        if (r.value.status === 'rejected') rejected += 1
        else queued += 1
      } else {
        failed += 1
      }
    }
    const summary = t('jobs.workspace.bulk.runSummary', { queued, rejected, skipped, failed })
    if (failed > 0) message.warning(summary)
    else message.success(summary)
    await refresh()
  } finally {
    bulkBusy.value = null
  }
}

async function confirmBulkAction(): Promise<void> {
  const kind = bulkConfirmKind.value
  if (!kind) return
  bulkBusy.value = kind
  try {
    if (kind === 'archive') {
      const targets = selectedActiveJobs.value
      const skipped = selectedJobIds.value.length - targets.length
      const results = await Promise.allSettled(
        targets.map((j) => jobs.archiveJob(j.id, { cascadeSnapshots: bulkArchiveCascadeSnapshots.value })),
      )
      const ok = results.filter((r) => r.status === 'fulfilled').length
      const failed = results.length - ok
      if (ok > 0) {
        const nextArchived = { ...selectedJobArchived.value }
        results.forEach((result, idx) => {
          if (result.status === 'fulfilled') {
            const target = targets[idx]
            if (target) nextArchived[target.id] = true
          }
        })
        selectedJobArchived.value = nextArchived
      }
      const summary = t('jobs.workspace.bulk.archiveSummary', { ok, skipped, failed })
      if (failed > 0) message.warning(summary)
      else message.success(summary)
    } else {
      const targets = selectedArchivedJobs.value
      const skipped = selectedJobIds.value.length - targets.length
      const results = await Promise.allSettled(targets.map((j) => jobs.unarchiveJob(j.id)))
      const ok = results.filter((r) => r.status === 'fulfilled').length
      const failed = results.length - ok
      if (ok > 0) {
        const nextArchived = { ...selectedJobArchived.value }
        results.forEach((result, idx) => {
          if (result.status === 'fulfilled') {
            const target = targets[idx]
            if (target) nextArchived[target.id] = false
          }
        })
        selectedJobArchived.value = nextArchived
      }
      const summary = t('jobs.workspace.bulk.unarchiveSummary', { ok, skipped, failed })
      if (failed > 0) message.warning(summary)
      else message.success(summary)
    }

    bulkConfirmOpen.value = false
    await refresh()
  } catch (error) {
    const msgKey = kind === 'archive' ? 'errors.archiveJobFailed' : 'errors.unarchiveJobFailed'
    message.error(formatToastError(t(msgKey), error, t))
  } finally {
    bulkBusy.value = null
  }
}

async function refresh(): Promise<void> {
  try {
    await jobs.refresh({
      includeArchived: showArchived.value,
      nodeId: nodeId.value,
      q: searchText.value,
      latestStatus: latestStatusFilter.value,
      scheduleMode: scheduleFilter.value,
      sort: sortKey.value,
      page: jobsPage.value,
      pageSize: jobsPageSize.value,
    })
  } catch (error) {
    message.error(formatToastError(t('errors.fetchJobsFailed'), error, t))
  }
}

const debouncedRefresh = createDebouncedTask(
  () => {
    void refresh()
  },
  LIST_QUERY_DEBOUNCE_MS,
)

function scheduleRefresh(): void {
  debouncedRefresh.schedule()
}

function resetToFirstPageAndRefresh(): void {
  const pageChanged = jobsPage.value !== 1
  jobsPage.value = 1
  if (!pageChanged) {
    scheduleRefresh()
  }
}

function openCreate(): void {
  editorModal.value?.openCreate({ nodeId: nodeId.value })
}

async function openEdit(jobId: string): Promise<void> {
  await editorModal.value?.openEdit(jobId, { nodeId: nodeId.value })
}

async function runNow(jobId: string): Promise<void> {
  if (!rowRunNowBusy.start(jobId)) return
  try {
    const res = await jobs.runNow(jobId)
    if (res.status === 'rejected') message.warning(t('messages.runRejected'))
    else message.success(t('messages.runQueued'))
  } catch (error) {
    message.error(formatToastError(t('errors.runNowFailed'), error, t))
  } finally {
    rowRunNowBusy.stop(jobId)
  }
}

function isRowRunNowBusy(jobId: string): boolean {
  return rowRunNowBusy.isBusy(jobId)
}

function openJob(jobId: string): void {
  void router.push(nodeScopedPath(nodeId.value, `jobs/${encodeURIComponent(jobId)}/overview`))
}

function onJobRowClick(jobId: string): void {
  if (layoutMode.value === 'list' && listSelectMode.value) {
    setJobSelected(jobId, !selectedJobIds.value.includes(jobId))
    return
  }
  openJob(jobId)
}

function jobRowOverflowOptions(job: JobListItem): DropdownOption[] {
  return [
    { label: t('jobs.workspace.actions.openDetails'), key: 'open_details' },
    { label: t('common.edit'), key: 'edit', disabled: !!job.archived_at },
  ]
}

function onSelectJobRowOverflow(job: JobListItem, key: string | number): void {
  if (key === 'open_details') {
    if (layoutMode.value === 'list') {
      ui.setJobsWorkspaceLayoutMode('split')
    }
    openJob(job.id)
    return
  }
  if (key === 'edit') {
    void openEdit(job.id)
  }
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

const tableNameSortOrder = computed<'ascend' | 'descend' | false>(() => {
  if (sortKey.value === 'name_asc') return 'ascend'
  if (sortKey.value === 'name_desc') return 'descend'
  return false
})

const tableUpdatedSortOrder = computed<'ascend' | 'descend' | false>(() => {
  if (sortKey.value === 'updated_asc') return 'ascend'
  if (sortKey.value === 'updated_desc') return 'descend'
  return false
})

function onTableSorterUpdate(sorter: unknown): void {
  const state = Array.isArray(sorter) ? sorter[0] : sorter
  if (!state || typeof state !== 'object') {
    sortKey.value = 'updated_desc'
    return
  }
  const columnKey = (state as { columnKey?: unknown }).columnKey
  const order = (state as { order?: unknown }).order

  if (columnKey === 'name') {
    sortKey.value = order === 'descend' ? 'name_desc' : 'name_asc'
    return
  }

  if (columnKey === 'updated_at') {
    sortKey.value = order === 'ascend' ? 'updated_asc' : 'updated_desc'
    return
  }

  sortKey.value = 'updated_desc'
}

const { tableColumns } = useJobsTableColumns({
  t,
  tableNameSortOrder,
  tableUpdatedSortOrder,
  formatNodeLabel,
  formatScheduleLabel,
  runStatusTagType,
  isRowRunNowBusy,
  openJob,
  openEdit,
  runNow,
  formatUnixSecondsYmdHm,
  formatUnixSecondsYmdHms,
})

onMounted(() => {
  void Promise.allSettled([
    refresh(),
    // Ensure node context labels are friendly (agent name vs id).
    agents.refresh().catch((error) => {
      message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
    }),
  ])
})

watch(layoutMode, () => {
  if (layoutMode.value !== 'split') {
    filtersPopoverOpen.value = false
    filtersDrawerOpen.value = false
    cleanupSplitResize()
  }
  if (layoutMode.value !== 'list') {
    listSelectMode.value = false
    bulkConfirmOpen.value = false
    bulkConfirmKind.value = null
  }
})

watch([searchText, sortKey, latestStatusFilter, scheduleFilter, showArchived], resetToFirstPageAndRefresh)
watch(
  () => [route.query.q, route.query.archived, route.query.status, route.query.schedule, route.query.sort],
  () => {
    applyRouteQuery(route.query as Record<string, unknown>)
  },
)
watch(jobsPage, () => {
  void refresh()
})
watch(jobsPageSize, () => {
  if (jobsPage.value !== 1) {
    jobsPage.value = 1
    return
  }
  void refresh()
})

watch(nodeId, () => {
  const pageChanged = jobsPage.value !== 1
  jobsPage.value = 1
  clearSelectedJobs()
  listSelectMode.value = false
  bulkConfirmOpen.value = false
  bulkConfirmKind.value = null
  if (!pageChanged) {
    void refresh()
  }
})

onBeforeUnmount(() => {
  cleanupSplitResize()
  debouncedRefresh.cancel()
})
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

      <n-button :title="t('jobs.workspace.actions.refreshList')" :aria-label="t('jobs.workspace.actions.refreshList')" @click="refresh">
        {{ t('jobs.workspace.actions.refreshList') }}
      </n-button>
      <n-button type="primary" @click="openCreate">{{ t('jobs.actions.create') }}</n-button>
    </PageHeader>

    <template v-if="isDesktop">
      <div ref="splitGridEl" class="grid grid-cols-1 gap-4 flex-1 min-h-0" :style="gridStyle">
        <n-card
          v-if="layoutMode !== 'detail'"
          class="app-card flex flex-col min-h-0 relative"
          :bordered="false"
        >
          <div
            v-if="layoutMode === 'split'"
            class="absolute top-0 right-0 bottom-0 w-2 cursor-col-resize select-none"
            :class="splitResizeActive ? 'opacity-100' : 'opacity-80 hover:opacity-100'"
            :title="t('jobs.workspace.actions.resizeSplit')"
            @pointerdown="onSplitResizePointerDown"
          >
            <div
              class="absolute top-4 bottom-4 left-1/2 -translate-x-1/2 w-px rounded"
              :class="splitResizeActive ? 'bg-[var(--app-primary)]' : 'bg-[var(--app-border)]'"
            />
          </div>

          <SelectionToolbar
            v-if="layoutMode === 'list'"
            class="mb-3"
            :count="selectedJobIds.length"
            :hint="t('common.selectionLoadedHint')"
            @clear="clearSelectedJobs"
          >
            <template #actions>
              <n-button
                size="small"
                type="primary"
                :loading="bulkBusy === 'run'"
                :disabled="bulkBusy !== null || selectedActiveJobs.length === 0"
                @click="bulkRunNow"
              >
                {{ t('jobs.actions.runNow') }}
              </n-button>
              <n-button
                size="small"
                type="warning"
                :loading="bulkBusy === 'archive'"
                :disabled="bulkBusy !== null || selectedActiveJobs.length === 0"
                @click="openBulkConfirm('archive')"
              >
                {{ t('jobs.actions.archive') }}
              </n-button>
              <n-button
                size="small"
                :loading="bulkBusy === 'unarchive'"
                :disabled="bulkBusy !== null || selectedArchivedJobs.length === 0"
                @click="openBulkConfirm('unarchive')"
              >
                {{ t('jobs.actions.unarchive') }}
              </n-button>
            </template>
          </SelectionToolbar>

          <ListPageScaffold>
            <template #toolbar>
              <ListToolbar compact embedded :stacked="layoutMode === 'split'">
                <template #search>
                  <div class="flex items-center gap-2">
                    <n-input
                      v-model:value="searchText"
                      size="small"
                      clearable
                      :placeholder="t('jobs.filters.searchPlaceholder')"
                      class="flex-1 min-w-0"
                      :input-props="{ name: 'jobs-search' }"
                    />

                    <PickerFiltersPopoverDrawer
                      v-if="layoutMode === 'split'"
                      :is-desktop="true"
                      :title="t('common.filters')"
                      :active-count="filtersActiveCount"
                      width-class="w-96"
                      :popover-open="filtersPopoverOpen"
                      :drawer-open="filtersDrawerOpen"
                      @update:popover-open="(v) => (filtersPopoverOpen = v)"
                      @update:drawer-open="(v) => (filtersDrawerOpen = v)"
                    >
                      <JobsFiltersPanel
                        v-model:show-archived="showArchived"
                        v-model:latest-status-filter="latestStatusFilter"
                        v-model:schedule-filter="scheduleFilter"
                        v-model:sort-key="sortKey"
                        layout="stack"
                        :latest-status-options="latestStatusFilterOptions"
                        :schedule-options="scheduleFilterOptions"
                        :sort-options="sortOptions"
                      />

                      <template #popoverFooter>
                        <div class="mt-4 pt-3 border-t border-[color:var(--app-border)] flex items-center justify-end">
                          <n-button size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
                        </div>
                      </template>
                    </PickerFiltersPopoverDrawer>
                  </div>
                </template>

                <template #filters>
                  <JobsFiltersPanel
                    v-if="layoutMode !== 'split'"
                    v-model:show-archived="showArchived"
                    v-model:latest-status-filter="latestStatusFilter"
                    v-model:schedule-filter="scheduleFilter"
                    v-model:sort-key="sortKey"
                    layout="inline"
                    :latest-status-options="latestStatusFilterOptions"
                    :schedule-options="scheduleFilterOptions"
                    :sort-options="sortOptions"
                  />
                </template>

                <template #actions>
                  <n-button
                    v-if="layoutMode === 'list' && jobsListView === 'list'"
                    size="small"
                    tertiary
                    @click="listSelectMode = !listSelectMode"
                  >
                    {{ listSelectMode ? t('common.done') : t('jobs.workspace.actions.select') }}
                  </n-button>
                  <n-button v-if="layoutMode !== 'split'" size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
                </template>
              </ListToolbar>
            </template>

            <template #content>
              <ListFilterSummaryRow
                :results-label="jobsResultsLabel"
                :chips="activeFilterChips"
                :clear-label="t('common.clear')"
                :wrap="layoutMode !== 'split'"
                @clear="clearFilters"
              />

              <div class="mt-2 flex-1 min-h-0">
                <ListStatePresenter
                  :loading="jobs.loading"
                  :item-count="jobs.items.length"
                  :base-empty="listBaseEmpty"
                  :loading-title="t('common.loading')"
                  :base-empty-title="t('jobs.empty.title')"
                  :base-empty-description="t('jobs.empty.description')"
                  :filtered-empty-title="t('common.noData')"
                  variant="plain"
                >
                  <template #baseActions>
                    <n-button type="primary" size="small" @click="openCreate">
                      {{ t('jobs.actions.create') }}
                    </n-button>
                  </template>

                  <template #filteredActions>
                    <n-button size="small" @click="clearFilters">
                      {{ t('common.clear') }}
                    </n-button>
                  </template>

                  <ScrollShadowPane
                    data-testid="jobs-list-scroll"
                    :class="jobsListView === 'list' ? 'app-divide-y' : ''"
                  >
                    <template v-if="jobsListView === 'table'">
                      <div class="py-2">
                        <n-data-table
                          class="app-list-table app-picker-table"
                          size="small"
                          remote
                          v-model:checked-row-keys="selectedJobIds"
                          :row-key="(row) => row.id"
                          :loading="jobs.loading"
                          :columns="tableColumns"
                          :data="pagedFilteredJobs"
                          :scroll-x="1200"
                          :row-class-name="(row) => (selectedJobIds.includes(row.id) || isSelected(row.id) ? 'app-picker-row--checked' : '')"
                          :row-props="(row) => ({ style: 'cursor: pointer;', onDblclick: () => openJob(row.id) })"
                          @update:sorter="onTableSorterUpdate"
                        />
                      </div>
                    </template>

                    <template v-else>
                      <JobsListRowItem
                        v-for="job in pagedFilteredJobs"
                        :key="job.id"
                        main-trigger-test-id="jobs-row-main-trigger"
                        run-now-test-id="jobs-row-run-now"
                        :job="job"
                        :selected="isSelected(job.id) || (layoutMode === 'list' && selectedJobIds.includes(job.id))"
                        :selectable="layoutMode === 'list' && listSelectMode"
                        :checked="selectedJobIds.includes(job.id)"
                        :open-details-label="t('jobs.workspace.actions.openDetails')"
                        :archived-label="t('jobs.archived')"
                        :never-ran-label="t('runs.neverRan')"
                        :run-now-label="t('jobs.actions.runNow')"
                        :node-label="formatNodeLabel(job.agent_id)"
                        :schedule-label="formatScheduleLabel(job)"
                        :latest-run-status-label="job.latest_run_status ? runStatusLabel(t, job.latest_run_status) : null"
                        :latest-run-status-type="job.latest_run_status ? runStatusTagType(job.latest_run_status) : null"
                        :latest-run-started-at-label="job.latest_run_started_at != null ? formatUnixSecondsYmdHm(job.latest_run_started_at) : null"
                        :latest-run-started-at-title="job.latest_run_started_at != null ? formatUnixSecondsYmdHms(job.latest_run_started_at) : null"
                        :run-now-loading="isRowRunNowBusy(job.id)"
                        :run-now-disabled="!!job.archived_at || isRowRunNowBusy(job.id)"
                        :overflow-options="jobRowOverflowOptions(job)"
                        @main-click="onJobRowClick(job.id)"
                        @update:checked="(value) => setJobSelected(job.id, value)"
                        @run-now="() => void runNow(job.id)"
                        @overflow-select="(key) => onSelectJobRowOverflow(job, key)"
                      />
                    </template>
                  </ScrollShadowPane>
                </ListStatePresenter>
              </div>
            </template>

            <template #footer>
              <AppPagination
                v-if="jobs.total > jobsPageSize"
                :page="jobsPage"
                :page-size="jobsPageSize"
                :item-count="jobs.total"
                :page-sizes="jobsPageSizeOptions"
                :loading="jobs.loading"
                :total-label="jobsPaginationLabel"
                @update:page="(value) => (jobsPage = value)"
                @update:page-size="(value) => (jobsPageSize = value)"
              />
            </template>
          </ListPageScaffold>
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
      <div v-if="!selectedJobId">
        <ListPageScaffold>
          <template #toolbar>
            <ListToolbar>
              <template #search>
                <div class="flex items-center gap-2">
                  <n-input
                    v-model:value="searchText"
                    size="small"
                    clearable
                    :placeholder="t('jobs.filters.searchPlaceholder')"
                    class="flex-1 min-w-0"
                    :input-props="{ name: 'jobs-search' }"
                  />

                  <PickerFiltersPopoverDrawer
                    :is-desktop="false"
                    :title="t('common.filters')"
                    :active-count="filtersActiveCount"
                    width-class="w-full"
                    :popover-open="filtersPopoverOpen"
                    :drawer-open="filtersDrawerOpen"
                    @update:popover-open="(v) => (filtersPopoverOpen = v)"
                    @update:drawer-open="(v) => (filtersDrawerOpen = v)"
                  >
                    <JobsFiltersPanel
                      v-model:show-archived="showArchived"
                      v-model:latest-status-filter="latestStatusFilter"
                      v-model:schedule-filter="scheduleFilter"
                      v-model:sort-key="sortKey"
                      layout="stack"
                      :latest-status-options="latestStatusFilterOptions"
                      :schedule-options="scheduleFilterOptions"
                      :sort-options="sortOptions"
                    />

                    <template #drawerFooter>
                      <div class="pt-3 border-t border-[color:var(--app-border)] flex items-center justify-end">
                        <n-button size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
                      </div>
                    </template>
                  </PickerFiltersPopoverDrawer>
                </div>
              </template>
            </ListToolbar>
          </template>

          <template #content>
            <ListFilterSummaryRow
              :results-label="jobsResultsLabel"
              :chips="activeFilterChips"
              :clear-label="t('common.clear')"
              wrap
              @clear="clearFilters"
            />

            <ListStatePresenter
              :loading="jobs.loading"
              :item-count="jobs.items.length"
              :base-empty="listBaseEmpty"
              :loading-title="t('common.loading')"
              :base-empty-title="t('jobs.empty.title')"
              :base-empty-description="t('jobs.empty.description')"
              :filtered-empty-title="t('common.noData')"
            >
              <template #baseActions>
                <n-button type="primary" size="small" @click="openCreate">
                  {{ t('jobs.actions.create') }}
                </n-button>
              </template>

              <template #filteredActions>
                <n-button size="small" @click="clearFilters">
                  {{ t('common.clear') }}
                </n-button>
              </template>

              <n-card class="app-card" :bordered="false">
                <div class="app-divide-y">
                  <JobsListRowItem
                    v-for="job in pagedFilteredJobs"
                    :key="job.id"
                    mobile
                    main-trigger-test-id="jobs-row-main-trigger-mobile"
                    run-now-test-id="jobs-row-run-now-mobile"
                    :job="job"
                    :open-details-label="t('jobs.workspace.actions.openDetails')"
                    :archived-label="t('jobs.archived')"
                    :never-ran-label="t('runs.neverRan')"
                    :run-now-label="t('jobs.actions.runNow')"
                    :node-label="formatNodeLabel(job.agent_id)"
                    :schedule-label="formatScheduleLabel(job)"
                    :latest-run-status-label="job.latest_run_status ? runStatusLabel(t, job.latest_run_status) : null"
                    :latest-run-status-type="job.latest_run_status ? runStatusTagType(job.latest_run_status) : null"
                    :latest-run-started-at-label="job.latest_run_started_at != null ? formatUnixSecondsYmdHm(job.latest_run_started_at) : null"
                    :latest-run-started-at-title="job.latest_run_started_at != null ? formatUnixSecondsYmdHms(job.latest_run_started_at) : null"
                    :run-now-loading="isRowRunNowBusy(job.id)"
                    :run-now-disabled="!!job.archived_at || isRowRunNowBusy(job.id)"
                    :overflow-options="jobRowOverflowOptions(job)"
                    @main-click="openJob(job.id)"
                    @run-now="() => void runNow(job.id)"
                    @overflow-select="(key) => onSelectJobRowOverflow(job, key)"
                  />
                </div>
              </n-card>
            </ListStatePresenter>
          </template>

          <template #footer>
            <AppPagination
              v-if="jobs.total > jobsPageSize"
              :page="jobsPage"
              :page-size="jobsPageSize"
              :item-count="jobs.total"
              :page-sizes="jobsPageSizeOptions"
              :loading="jobs.loading"
              :total-label="jobsPaginationLabel"
              @update:page="(value) => (jobsPage = value)"
              @update:page-size="(value) => (jobsPageSize = value)"
            />
          </template>
        </ListPageScaffold>
      </div>

      <router-view v-else />
    </template>

    <JobEditorModal ref="editorModal" @saved="refresh" />

    <AppModalShell
      v-model:show="bulkConfirmOpen"
      :mask-closable="bulkBusy === null"
      :width="isDesktop ? '520px' : '92vw'"
      :title="bulkConfirmTitle"
    >
      <div class="text-sm app-text-muted">{{ bulkConfirmBody }}</div>

      <div
        v-if="bulkConfirmKind === 'archive'"
        class="rounded app-panel-inset p-3 space-y-1"
      >
        <n-checkbox
          :checked="bulkArchiveCascadeSnapshots"
          @update:checked="(v) => (bulkArchiveCascadeSnapshots = v)"
        >
          {{ t('jobs.archiveCascadeLabel') }}
        </n-checkbox>
        <div class="app-meta-text">{{ t('jobs.archiveCascadeHelp') }}</div>
      </div>

      <template #footer>
        <n-button :disabled="bulkBusy !== null" @click="bulkConfirmOpen = false">{{ t('common.cancel') }}</n-button>
        <n-button
          v-if="bulkConfirmKind === 'archive'"
          type="warning"
          :loading="bulkBusy === 'archive'"
          :disabled="bulkBusy !== null || selectedActiveJobs.length === 0"
          @click="confirmBulkAction"
        >
          {{ t('jobs.actions.archive') }}
        </n-button>
        <n-button
          v-else-if="bulkConfirmKind === 'unarchive'"
          :loading="bulkBusy === 'unarchive'"
          :disabled="bulkBusy !== null || selectedArchivedJobs.length === 0"
          @click="confirmBulkAction"
        >
          {{ t('jobs.actions.unarchive') }}
        </n-button>
      </template>
    </AppModalShell>
  </div>
</template>
