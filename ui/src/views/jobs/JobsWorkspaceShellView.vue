<script setup lang="ts">
import { computed, h, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NCheckbox, NDataTable, NIcon, NInput, NModal, NPagination, NRadioButton, NRadioGroup, NSelect, NSwitch, NTag, useMessage, type DataTableColumns, type DropdownOption } from 'naive-ui'
import { CreateOutline, PlayOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import NodeContextTag from '@/components/NodeContextTag.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import SelectionToolbar from '@/components/list/SelectionToolbar.vue'
import OverflowActionsButton from '@/components/list/OverflowActionsButton.vue'
import ListToolbar from '@/components/list/ListToolbar.vue'
import ScrollShadowPane from '@/components/scroll/ScrollShadowPane.vue'
import PickerActiveChipsRow, { type PickerActiveChip } from '@/components/pickers/PickerActiveChipsRow.vue'
import PickerFiltersPopoverDrawer from '@/components/pickers/PickerFiltersPopoverDrawer.vue'
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
const filtersPopoverOpen = ref<boolean>(false)
const filtersDrawerOpen = ref<boolean>(false)

const jobsPage = ref<number>(1)
const jobsPageSize = ref<number>(20)
const jobsPageSizeOptions = [20, 50, 100]

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

const SPLIT_LIST_MIN_PX = 280
const SPLIT_LIST_MAX_PX = 640
const SPLIT_DETAIL_MIN_PX = 360

const splitGridEl = ref<HTMLElement | null>(null)
const splitResizeActive = ref<boolean>(false)
const splitListWidthDraftPx = ref<number | null>(null)
let splitResizeCleanup: (() => void) | null = null

const splitListWidthPx = computed<number>(() => splitListWidthDraftPx.value ?? ui.jobsWorkspaceSplitListWidthPx)

const gridStyle = computed<Record<string, string> | undefined>(() => {
  if (layoutMode.value !== 'split') return undefined
  return {
    gridTemplateColumns: `minmax(0, ${splitListWidthPx.value}px) minmax(0, 1fr)`,
  }
})

function clampInt(n: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.round(n)))
}

function onSplitResizePointerDown(event: PointerEvent): void {
  if (layoutMode.value !== 'split') return
  const el = splitGridEl.value
  if (!el) return
  splitResizeCleanup?.()

  const handle = event.currentTarget as HTMLElement | null
  handle?.setPointerCapture?.(event.pointerId)

  const startX = event.clientX
  const startWidth = splitListWidthPx.value
  splitResizeActive.value = true
  splitListWidthDraftPx.value = startWidth

  const rect = el.getBoundingClientRect()
  const style = window.getComputedStyle(el)
  const colGap = Number.parseFloat(style.columnGap || '0') || 0
  const maxByContainer = rect.width - colGap - SPLIT_DETAIL_MIN_PX
  const maxWidth = clampInt(Math.min(SPLIT_LIST_MAX_PX, maxByContainer), SPLIT_LIST_MIN_PX, SPLIT_LIST_MAX_PX)

  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'

  let raf: number | null = null
  const onMove = (e: PointerEvent) => {
    const dx = e.clientX - startX
    const next = clampInt(startWidth + dx, SPLIT_LIST_MIN_PX, maxWidth)
    if (raf != null) cancelAnimationFrame(raf)
    raf = requestAnimationFrame(() => {
      raf = null
      splitListWidthDraftPx.value = next
    })
  }

  const cleanup = () => {
    window.removeEventListener('pointermove', onMove)
    splitResizeActive.value = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    const next = splitListWidthDraftPx.value
    splitListWidthDraftPx.value = null
    splitResizeCleanup = null
    if (typeof next === 'number') {
      ui.setJobsWorkspaceSplitListWidthPx(next)
    }
  }

  const onUp = () => {
    window.removeEventListener('pointerup', onUp)
    cleanup()
  }

  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
  splitResizeCleanup = cleanup
}

const filtersActiveCount = computed(() => {
  let n = 0
  if (searchText.value.trim().length > 0) n += 1
  if (showArchived.value) n += 1
  if (listLatestStatusFilter.value !== 'all') n += 1
  if (listScheduleFilter.value !== 'all') n += 1
  if (sortKey.value !== 'updated_desc') n += 1
  return n
})

const activeFilterChips = computed<PickerActiveChip[]>(() => {
  const chips: PickerActiveChip[] = []

  const q = searchText.value.trim()
  if (q.length > 0) {
    chips.push({
      key: 'q',
      label: `${t('common.search')}: ${q}`,
      onClose: () => {
        searchText.value = ''
      },
    })
  }

  if (showArchived.value) {
    chips.push({
      key: 'archived',
      label: t('jobs.showArchived'),
      onClose: () => {
        showArchived.value = false
      },
    })
  }

  if (listLatestStatusFilter.value !== 'all') {
    const label =
      latestStatusFilterOptions.value.find((o) => o.value === listLatestStatusFilter.value)?.label ??
      String(listLatestStatusFilter.value)
    chips.push({
      key: 'status',
      label: `${t('runs.columns.status')}: ${label}`,
      onClose: () => {
        listLatestStatusFilter.value = 'all'
      },
    })
  }

  if (listScheduleFilter.value !== 'all') {
    const label =
      scheduleFilterOptions.value.find((o) => o.value === listScheduleFilter.value)?.label ??
      String(listScheduleFilter.value)
    chips.push({
      key: 'schedule',
      label: `${t('jobs.columns.schedule')}: ${label}`,
      onClose: () => {
        listScheduleFilter.value = 'all'
      },
    })
  }

  if (sortKey.value !== 'updated_desc') {
    const label = sortOptions.value.find((o) => o.value === sortKey.value)?.label ?? String(sortKey.value)
    chips.push({
      key: 'sort',
      label: `${t('common.sort')}: ${label}`,
      onClose: () => {
        sortKey.value = 'updated_desc'
      },
    })
  }

  return chips
})

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

  const latestStatus = listLatestStatusFilter.value
  const scheduleMode = listScheduleFilter.value

  const filtered = list.filter((j) => {
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

const pagedFilteredJobs = computed<JobListItem[]>(() => {
  const start = (jobsPage.value - 1) * jobsPageSize.value
  return filteredJobs.value.slice(start, start + jobsPageSize.value)
})

const jobsPageCount = computed<number>(() => Math.max(1, Math.ceil(filteredJobs.value.length / jobsPageSize.value)))

const selectedJobIds = ref<string[]>([])
const listSelectMode = ref<boolean>(false)

const nodeJobsById = computed(() => new Map(nodeScopedJobs.value.map((j) => [j.id, j])))
const selectedJobs = computed<JobListItem[]>(() =>
  selectedJobIds.value.map((id) => nodeJobsById.value.get(id)).filter((v): v is JobListItem => !!v),
)
const selectedActiveJobs = computed<JobListItem[]>(() => selectedJobs.value.filter((j) => !j.archived_at))
const selectedArchivedJobs = computed<JobListItem[]>(() => selectedJobs.value.filter((j) => !!j.archived_at))

function setJobSelected(jobId: string, checked: boolean): void {
  const next = new Set(selectedJobIds.value)
  if (checked) next.add(jobId)
  else next.delete(jobId)
  selectedJobIds.value = [...next]
}

watch(nodeScopedJobs, () => {
  const allowed = new Set(nodeScopedJobs.value.map((j) => j.id))
  selectedJobIds.value = selectedJobIds.value.filter((id) => allowed.has(id))
})

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
      const summary = t('jobs.workspace.bulk.archiveSummary', { ok, skipped, failed })
      if (failed > 0) message.warning(summary)
      else message.success(summary)
    } else {
      const targets = selectedArchivedJobs.value
      const skipped = selectedJobIds.value.length - targets.length
      const results = await Promise.allSettled(targets.map((j) => jobs.unarchiveJob(j.id)))
      const ok = results.filter((r) => r.status === 'fulfilled').length
      const failed = results.length - ok
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

function onJobRowClick(jobId: string): void {
  if (layoutMode.value === 'list' && listSelectMode.value) {
    setJobSelected(jobId, !selectedJobIds.value.includes(jobId))
    return
  }
  openJob(jobId)
}

function jobRowOverflowOptions(): DropdownOption[] {
  return [{ label: t('jobs.workspace.actions.openDetails'), key: 'open_details' }]
}

function onSelectJobRowOverflow(job: JobListItem, key: string | number): void {
  if (key === 'open_details') {
    if (layoutMode.value === 'list') {
      ui.setJobsWorkspaceLayoutMode('split')
    }
    openJob(job.id)
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

const tableColumns = computed<DataTableColumns<JobListItem>>(() => [
  { type: 'selection' as const },
  {
    title: t('jobs.columns.name'),
    key: 'name',
    sorter: 'default',
    sortOrder: tableNameSortOrder.value,
    fixed: 'left',
    width: 260,
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
    width: 160,
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
    width: 180,
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
    width: 120,
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
    width: 140,
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
    sorter: 'default',
    sortOrder: tableUpdatedSortOrder.value,
    width: 140,
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
    fixed: 'right',
    width: 200,
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
    splitResizeCleanup?.()
  }
  if (layoutMode.value !== 'list') {
    listSelectMode.value = false
    bulkConfirmOpen.value = false
    bulkConfirmKind.value = null
  }
})

watch(showArchived, () => void refresh())
watch([searchText, sortKey, listLatestStatusFilter, listScheduleFilter, showArchived], () => {
  jobsPage.value = 1
})
watch([() => filteredJobs.value.length, jobsPageSize], () => {
  if (jobsPage.value > jobsPageCount.value) jobsPage.value = jobsPageCount.value
})

watch(nodeId, () => {
  jobsPage.value = 1
  selectedJobIds.value = []
  listSelectMode.value = false
  bulkConfirmOpen.value = false
  bulkConfirmKind.value = null
})

onBeforeUnmount(() => {
  splitResizeCleanup?.()
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
            @clear="selectedJobIds = []"
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
                  <div class="space-y-4">
                    <div class="flex items-center justify-between gap-3">
                      <span class="text-sm app-text-muted">{{ t('jobs.showArchived') }}</span>
                      <n-switch v-model:value="showArchived" :aria-label="t('jobs.showArchived')" />
                    </div>

                    <div class="space-y-2">
                      <div class="text-sm app-text-muted">{{ t('runs.columns.status') }}</div>
                      <n-select
                        v-model:value="listLatestStatusFilter"
                        size="small"
                        :aria-label="t('runs.columns.status')"
                        :options="latestStatusFilterOptions"
                        :consistent-menu-width="false"
                        class="w-full"
                      />
                    </div>

                    <div class="space-y-2">
                      <div class="text-sm app-text-muted">{{ t('jobs.columns.schedule') }}</div>
                      <n-select
                        v-model:value="listScheduleFilter"
                        size="small"
                        :aria-label="t('jobs.columns.schedule')"
                        :options="scheduleFilterOptions"
                        :consistent-menu-width="false"
                        class="w-full"
                      />
                    </div>

                    <div class="space-y-2">
                      <div class="text-sm app-text-muted">{{ t('common.sort') }}</div>
                      <n-select v-model:value="sortKey" size="small" :aria-label="t('common.sort')" :options="sortOptions" class="w-full" />
                    </div>
                  </div>

                  <template #popoverFooter>
                    <div class="mt-4 pt-3 border-t border-[color:var(--app-border)] flex items-center justify-end">
                      <n-button size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
                    </div>
                  </template>
                </PickerFiltersPopoverDrawer>
              </div>
            </template>

            <template #filters>
              <template v-if="layoutMode !== 'split'">
                <div class="shrink-0 flex items-center gap-2 whitespace-nowrap h-7">
                  <span class="text-sm app-text-muted">{{ t('jobs.showArchived') }}</span>
                  <n-switch v-model:value="showArchived" :aria-label="t('jobs.showArchived')" />
                </div>

                <div v-if="layoutMode === 'list'" class="shrink-0 flex items-center gap-2 whitespace-nowrap">
                  <span class="text-sm app-text-muted">{{ t('runs.columns.status') }}</span>
                  <n-select
                    v-model:value="listLatestStatusFilter"
                    size="small"
                    :aria-label="t('runs.columns.status')"
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
                    :aria-label="t('jobs.columns.schedule')"
                    :options="scheduleFilterOptions"
                    :consistent-menu-width="false"
                    class="min-w-[8rem]"
                  />
                </div>
              </template>
            </template>

            <template #sort>
              <div v-if="layoutMode !== 'split'" class="w-full md:w-56 md:flex-none">
                <n-select v-model:value="sortKey" size="small" :aria-label="t('common.sort')" :options="sortOptions" />
              </div>
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

          <div class="mt-3 space-y-2">
            <div class="text-xs app-text-muted">
              {{ t('jobs.workspace.filters.resultsCount', { filtered: filteredJobs.length, total: nodeScopedJobs.length }) }}
            </div>
            <PickerActiveChipsRow
              :chips="activeFilterChips"
              :clear-label="t('common.clear')"
              :wrap="layoutMode !== 'split'"
              @clear="clearFilters"
            />
          </div>

          <div class="mt-2 flex-1 min-h-0">
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
                    <n-data-table
                      class="app-picker-table"
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
                  <button
                    v-for="job in pagedFilteredJobs"
                    :key="job.id"
                    type="button"
                    class="app-list-row group"
                    :class="isSelected(job.id) || (layoutMode === 'list' && selectedJobIds.includes(job.id)) ? 'bg-[var(--app-primary-soft)]' : ''"
                    @click="onJobRowClick(job.id)"
                  >
                    <div class="min-w-0 flex items-start gap-2">
                      <div v-if="layoutMode === 'list' && listSelectMode" class="pt-0.5" @click.stop>
                        <n-checkbox
                          :checked="selectedJobIds.includes(job.id)"
                          @update:checked="(v) => setJobSelected(job.id, v)"
                        />
                      </div>

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
                    </div>

                    <div class="shrink-0 relative text-right min-w-[6.5rem]">
                      <div class="flex flex-col items-end gap-1 transition-opacity group-hover:opacity-0 group-focus-within:opacity-0">
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

                      <div
                        class="absolute inset-0 flex items-center justify-end gap-1 opacity-0 pointer-events-none transition-opacity group-hover:opacity-100 group-hover:pointer-events-auto group-focus-within:opacity-100 group-focus-within:pointer-events-auto"
                        @click.stop
                      >
                        <n-button
                          size="small"
                          quaternary
                          :disabled="!!job.archived_at"
                          :title="t('jobs.actions.runNow')"
                          :aria-label="t('jobs.actions.runNow')"
                          @click="() => void runNow(job.id)"
                        >
                          <template #icon>
                            <n-icon><PlayOutline /></n-icon>
                          </template>
                        </n-button>
                        <n-button
                          size="small"
                          quaternary
                          :disabled="!!job.archived_at"
                          :title="t('common.edit')"
                          :aria-label="t('common.edit')"
                          @click="() => void openEdit(job.id)"
                        >
                          <template #icon>
                            <n-icon><CreateOutline /></n-icon>
                          </template>
                        </n-button>
                        <OverflowActionsButton
                          size="small"
                          :options="jobRowOverflowOptions()"
                          @select="(key) => onSelectJobRowOverflow(job, key)"
                        />
                      </div>
                    </div>
                  </button>
                </template>
              </ScrollShadowPane>

              <div v-if="filteredJobs.length > jobsPageSize" class="mt-3 flex justify-end">
                <n-pagination
                  v-model:page="jobsPage"
                  v-model:page-size="jobsPageSize"
                  :item-count="filteredJobs.length"
                  :page-sizes="jobsPageSizeOptions"
                  show-size-picker
                  size="small"
                />
              </div>
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
                <div class="space-y-4">
                  <div class="flex items-center justify-between gap-3">
                    <span class="text-sm app-text-muted">{{ t('jobs.showArchived') }}</span>
                    <n-switch v-model:value="showArchived" :aria-label="t('jobs.showArchived')" />
                  </div>

                  <div class="space-y-2">
                    <div class="text-sm app-text-muted">{{ t('runs.columns.status') }}</div>
                    <n-select
                      v-model:value="listLatestStatusFilter"
                      size="small"
                      :aria-label="t('runs.columns.status')"
                      :options="latestStatusFilterOptions"
                      :consistent-menu-width="false"
                      class="w-full"
                    />
                  </div>

                  <div class="space-y-2">
                    <div class="text-sm app-text-muted">{{ t('jobs.columns.schedule') }}</div>
                    <n-select
                      v-model:value="listScheduleFilter"
                      size="small"
                      :aria-label="t('jobs.columns.schedule')"
                      :options="scheduleFilterOptions"
                      :consistent-menu-width="false"
                      class="w-full"
                    />
                  </div>

                  <div class="space-y-2">
                    <div class="text-sm app-text-muted">{{ t('common.sort') }}</div>
                    <n-select v-model:value="sortKey" size="small" :aria-label="t('common.sort')" :options="sortOptions" class="w-full" />
                  </div>
                </div>

                <template #drawerFooter>
                  <div class="pt-3 border-t border-[color:var(--app-border)] flex items-center justify-end">
                    <n-button size="small" @click="clearFilters">{{ t('common.clear') }}</n-button>
                  </div>
                </template>
              </PickerFiltersPopoverDrawer>
            </div>
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
              v-for="job in pagedFilteredJobs"
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

          <div v-if="filteredJobs.length > jobsPageSize" class="mt-3 flex justify-end">
            <n-pagination
              v-model:page="jobsPage"
              v-model:page-size="jobsPageSize"
              :item-count="filteredJobs.length"
              :page-sizes="jobsPageSizeOptions"
              show-size-picker
              size="small"
            />
          </div>
        </n-card>
      </div>

      <router-view v-else />
    </template>

    <JobEditorModal ref="editorModal" @saved="refresh" />

    <n-modal
      v-model:show="bulkConfirmOpen"
      :mask-closable="bulkBusy === null"
      preset="card"
      :style="{ width: isDesktop ? '520px' : '92vw' }"
      :title="bulkConfirmTitle"
    >
      <div class="space-y-4">
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
          <div class="text-xs app-text-muted">{{ t('jobs.archiveCascadeHelp') }}</div>
        </div>

        <div class="flex items-center justify-end gap-2">
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
        </div>
      </div>
    </n-modal>
  </div>
</template>
