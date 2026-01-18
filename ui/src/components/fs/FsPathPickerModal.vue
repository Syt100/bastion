<script setup lang="ts">
import { computed, h, nextTick, ref, watch } from 'vue'
import {
  NAlert,
  NBadge,
  NButton,
  NDataTable,
  NDrawer,
  NDrawerContent,
  NForm,
  NFormItem,
  NIcon,
  NInput,
  NInputNumber,
  NModal,
  NPopover,
  NSelect,
  NSpace,
  NSwitch,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { FilterOutline, SearchOutline } from '@vicons/ionicons5'

import { apiFetch } from '@/lib/api'
import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatBytes } from '@/lib/format'
import { formatToastError, toApiErrorInfo } from '@/lib/errors'
import { formatUnixSecondsYmdHms } from '@/lib/datetime'
import { useObservedElementHeightPx } from '@/lib/resizeObserver'
import PickerPathBarInput, { type PickerPathBarInputExpose } from '@/components/pickers/PickerPathBarInput.vue'

type FsListEntry = {
  name: string
  path: string
  kind: string
  size: number
  mtime?: number | null
}

type FsListResponse = {
  path: string
  entries: FsListEntry[]
}

export type FsPathPickerModalExpose = {
  open: (nodeId: 'hub' | string, initialPath?: string | FsPathPickerOpenOptions) => void
}

const emit = defineEmits<{
  (e: 'picked', paths: string[]): void
}>()

const { t } = useI18n()
const message = useMessage()
const isDesktop = useMediaQuery(MQ.mdUp)

const show = ref<boolean>(false)
const pathBar = ref<PickerPathBarInputExpose | null>(null)
const loading = ref<boolean>(false)
const nodeId = ref<'hub' | string>('hub')
export type FsPathPickerMode = 'multi_paths' | 'single_dir'
export type FsPathPickerOpenOptions = {
  path?: string
  mode?: FsPathPickerMode
}

const pickerMode = ref<FsPathPickerMode>('multi_paths')
const isSingleDirMode = computed(() => pickerMode.value === 'single_dir')

const currentPath = ref<string>('/')
const entries = ref<FsListEntry[]>([])
const checked = ref<string[]>([])

const searchDraft = ref<string>('')
const searchApplied = ref<string>('')
const kindFilter = ref<'all' | 'dir' | 'file' | 'symlink'>('all')
const hideDotfiles = ref<boolean>(false)

type SizeUnit = 'B' | 'KB' | 'MB' | 'GB'

const typeSort = ref<'dir_first' | 'file_first'>('dir_first')

const sizeMinDraft = ref<number | null>(null)
const sizeMaxDraft = ref<number | null>(null)
const sizeUnitDraft = ref<SizeUnit>('MB')

const sizeMinApplied = ref<number | null>(null)
const sizeMaxApplied = ref<number | null>(null)
const sizeUnitApplied = ref<SizeUnit>('MB')

const filtersPopoverOpen = ref<boolean>(false)
const filtersDrawerOpen = ref<boolean>(false)
const pickCurrentDirConfirmOpen = ref<boolean>(false)

type SingleDirStatus = 'unknown' | 'ok' | 'not_found' | 'not_directory' | 'permission_denied' | 'agent_offline' | 'error'
const singleDirStatus = ref<SingleDirStatus>('unknown')
const singleDirMessage = ref<string>('')
const singleDirValidatedPath = ref<string>('')
const singleDirNotFoundConfirmPath = ref<string>('')

const tableContainerEl = ref<HTMLElement | null>(null)
function computeTableBodyMaxHeightPx(container: HTMLElement): number {
  const containerHeight = container.clientHeight
  const headerEl = container.querySelector('.n-data-table-base-table-header') as HTMLElement | null
  const theadEl = container.querySelector('thead') as HTMLElement | null
  const headerHeight = headerEl?.clientHeight || theadEl?.clientHeight || 0
  return containerHeight - headerHeight
}
const {
  heightPx: tableBodyMaxHeightPx,
  start: startTableHeightObserver,
  stop: stopTableHeightObserver,
  measure: measureTableHeight,
} = useObservedElementHeightPx(tableContainerEl, computeTableBodyMaxHeightPx)

watch(show, (open) => {
  if (!open) {
    pickCurrentDirConfirmOpen.value = false
    stopTableHeightObserver()
    return
  }

  // Keep the initial focus on the path input so the first action button doesn't look "selected".
  nextTick().then(() => {
    try {
      pathBar.value?.focus?.()
    } catch {
      // ignore
    }
    startTableHeightObserver()
    requestAnimationFrame(() => {
      measureTableHeight()
      requestAnimationFrame(() => {
        measureTableHeight()
      })
    })
  })
})

const modalTitle = computed(() => (isSingleDirMode.value ? t('fsPicker.dirTitle') : t('fsPicker.title')))
const singleDirConfirmLabel = computed(() =>
  isSingleDirMode.value &&
  singleDirStatus.value === 'not_found' &&
  singleDirNotFoundConfirmPath.value === normalizePath(currentPath.value)
    ? t('fsPicker.selectDirAnyway')
    : t('fsPicker.selectCurrentDir'),
)

const selectedCount = computed(() => checked.value.length)
const selectedUnique = computed(() => uniqueNormalizedPaths(checked.value))
const currentDirNormalized = computed(() => normalizePath(currentPath.value))

const currentPathModel = computed({
  get: () => currentPath.value,
  set: (v: string) => {
    currentPath.value = v
    onCurrentPathEdited()
  },
})

function onPathBarNavigate(): void {
  refresh()
}

const hasSearchDraftChanges = computed(() => searchDraft.value.trim() !== searchApplied.value)

function applySearch(): void {
  searchApplied.value = searchDraft.value.trim()
}

const kindOptions = computed(() => [
  { label: t('fsPicker.kindAll'), value: 'all' as const },
  { label: t('common.dir'), value: 'dir' as const },
  { label: t('common.file'), value: 'file' as const },
  { label: t('common.symlink'), value: 'symlink' as const },
])

const sizeUnitOptions = computed(() => [
  { label: 'B', value: 'B' as const },
  { label: 'KB', value: 'KB' as const },
  { label: 'MB', value: 'MB' as const },
  { label: 'GB', value: 'GB' as const },
])

const typeSortOptions = computed(() => [
  { label: t('common.dirFirst'), value: 'dir_first' as const },
  { label: t('common.fileFirst'), value: 'file_first' as const },
])

const hasSizeApplied = computed(() => sizeMinApplied.value != null || sizeMaxApplied.value != null)

const activeFilterCount = computed(() => {
  let count = 0
  if (kindFilter.value !== 'all') count += 1
  if (hideDotfiles.value) count += 1
  if (hasSizeApplied.value) count += 1
  if (typeSort.value !== 'dir_first') count += 1
  return count
})

function sizeUnitMultiplier(unit: SizeUnit): number {
  if (unit === 'KB') return 1024
  if (unit === 'MB') return 1024 * 1024
  if (unit === 'GB') return 1024 * 1024 * 1024
  return 1
}

function formatSizeRange(min: number | null, max: number | null, unit: SizeUnit): string {
  const u = unit
  if (min != null && max != null) return `${min}–${max} ${u}`
  if (min != null) return `≥ ${min} ${u}`
  if (max != null) return `≤ ${max} ${u}`
  return '-'
}

type ActiveChip = {
  key: string
  label: string
  onClose: () => void
}

const activeChips = computed<ActiveChip[]>(() => {
  const out: ActiveChip[] = []

  const q = searchApplied.value.trim()
  if (q) out.push({ key: 'search', label: `${t('common.search')}: ${q}`, onClose: clearSearch })

  if (kindFilter.value !== 'all') {
    out.push({
      key: 'kind',
      label: `${t('common.type')}: ${kindLabel(kindFilter.value)}`,
      onClose: clearKindFilter,
    })
  }

  if (hideDotfiles.value) {
    out.push({ key: 'dotfiles', label: t('common.hideDotfiles'), onClose: clearDotfiles })
  }

  if (hasSizeApplied.value) {
    out.push({
      key: 'size',
      label: `${t('common.fileSize')}: ${formatSizeRange(sizeMinApplied.value, sizeMaxApplied.value, sizeUnitApplied.value)}`,
      onClose: clearSizeFilter,
    })
  }

  if (typeSort.value !== 'dir_first') {
    out.push({
      key: 'typeSort',
      label: `${t('common.typeSort')}: ${typeSort.value === 'file_first' ? t('common.fileFirst') : t('common.dirFirst')}`,
      onClose: resetTypeSort,
    })
  }

  return out
})

const visibleEntries = computed(() => {
  const needle = searchApplied.value.trim().toLowerCase()
  const mult = sizeUnitMultiplier(sizeUnitApplied.value)
  const minBytes =
    sizeMinApplied.value != null && Number.isFinite(sizeMinApplied.value)
      ? Math.max(0, Math.floor(sizeMinApplied.value * mult))
      : null
  const maxBytes =
    sizeMaxApplied.value != null && Number.isFinite(sizeMaxApplied.value)
      ? Math.max(0, Math.floor(sizeMaxApplied.value * mult))
      : null

  const filtered = entries.value.filter((e) => {
    if (hideDotfiles.value && e.name.startsWith('.')) return false
    if (kindFilter.value !== 'all' && e.kind !== kindFilter.value) return false
    if (needle && !e.name.toLowerCase().includes(needle)) return false
    if ((minBytes != null || maxBytes != null) && e.kind !== 'dir') {
      if (minBytes != null && e.size < minBytes) return false
      if (maxBytes != null && e.size > maxBytes) return false
    }
    return true
  })

  function rank(kind: string): number {
    const fileLike = kind === 'file' || kind === 'symlink'
    if (typeSort.value === 'file_first') return fileLike ? 0 : kind === 'dir' ? 1 : 2
    return kind === 'dir' ? 0 : fileLike ? 1 : 2
  }

  filtered.sort((a, b) => {
    const ra = rank(a.kind)
    const rb = rank(b.kind)
    if (ra !== rb) return ra - rb
    return a.name.localeCompare(b.name)
  })

  return filtered
})

function formatMtimeDesktop(ts?: number | null): string {
  if (!Number.isFinite(ts as number) || !ts) return '-'
  return formatUnixSecondsYmdHms(ts)
}

function formatMtimeMobile(ts?: number | null): string {
  if (!Number.isFinite(ts as number) || !ts) return '-'
  return formatUnixSecondsYmdHms(ts)
}

function kindLabel(kind: string): string {
  if (kind === 'dir') return t('common.dir')
  if (kind === 'symlink') return t('common.symlink')
  if (kind === 'file') return t('common.file')
  return kind
}

function normalizePath(p: string): string {
  return p.trim()
}

function uniqueNormalizedPaths(paths: string[]): string[] {
  const out: string[] = []
  const seen = new Set<string>()
  for (const raw of paths) {
    const v = normalizePath(raw)
    if (!v) continue
    if (seen.has(v)) continue
    seen.add(v)
    out.push(v)
  }
  return out
}

function onCurrentPathEdited(): void {
  if (!isSingleDirMode.value) return
  singleDirStatus.value = 'unknown'
  singleDirMessage.value = ''
  singleDirValidatedPath.value = ''
  singleDirNotFoundConfirmPath.value = ''
  entries.value = []
}

function applySizeFilter(): void {
  const min = sizeMinDraft.value
  const max = sizeMaxDraft.value
  let nextMin = min != null && Number.isFinite(min) ? Math.max(0, min) : null
  let nextMax = max != null && Number.isFinite(max) ? Math.max(0, max) : null
  if (nextMin != null && nextMax != null && nextMin > nextMax) [nextMin, nextMax] = [nextMax, nextMin]
  sizeMinApplied.value = nextMin
  sizeMaxApplied.value = nextMax
  sizeUnitApplied.value = sizeUnitDraft.value
}

function clearSearch(): void {
  searchDraft.value = ''
  searchApplied.value = ''
}

function clearKindFilter(): void {
  kindFilter.value = 'all'
}

function clearDotfiles(): void {
  hideDotfiles.value = false
}

function clearSizeFilter(): void {
  sizeMinDraft.value = null
  sizeMaxDraft.value = null
  sizeMinApplied.value = null
  sizeMaxApplied.value = null
}

function resetTypeSort(): void {
  typeSort.value = 'dir_first'
}

function resetAllFilters(): void {
  clearSearch()
  clearKindFilter()
  clearDotfiles()
  clearSizeFilter()
  sizeUnitDraft.value = 'MB'
  sizeUnitApplied.value = 'MB'
  resetTypeSort()
}

function computeParentPath(p: string): string {
  const raw = p.trim()
  if (!raw) return raw

  const trimmed = raw.replace(/[\\/]+$/, '')
  if (trimmed === '' || trimmed === '/') return '/'
  if (/^[A-Za-z]:$/.test(trimmed)) return `${trimmed}\\`

  const idxSlash = Math.max(trimmed.lastIndexOf('/'), trimmed.lastIndexOf('\\'))
  if (idxSlash <= 0) {
    if (/^[A-Za-z]:/.test(trimmed)) return trimmed.slice(0, 2) + '\\'
    return '/'
  }
  return trimmed.slice(0, idxSlash) || '/'
}

const modalStyle = computed(() =>
  isDesktop.value
    ? { width: MODAL_WIDTH.lg, height: MODAL_HEIGHT.desktopLoose }
    : { width: '100vw', height: '100vh', borderRadius: '0', margin: '0' },
)

const LAST_DIR_KEY_PREFIX = 'bastion.fsPicker.lastDir.'

function lastDirStorageKey(id: string): string {
  return `${LAST_DIR_KEY_PREFIX}${encodeURIComponent(id)}`
}

function loadLastDir(id: string): string | null {
  try {
    const v = localStorage.getItem(lastDirStorageKey(id))
    return v && v.trim() ? v.trim() : null
  } catch {
    return null
  }
}

function saveLastDir(id: string, path: string): void {
  try {
    const v = path.trim()
    if (!v) return
    localStorage.setItem(lastDirStorageKey(id), v)
  } catch {
    // ignore
  }
}

async function refresh(): Promise<void> {
  const p = normalizePath(currentPath.value)
  if (!p) {
    message.error(t('errors.fsPathRequired'))
    return
  }

  loading.value = true
  if (isSingleDirMode.value) {
    singleDirStatus.value = 'unknown'
    singleDirMessage.value = ''
    singleDirValidatedPath.value = p
    singleDirNotFoundConfirmPath.value = ''
  }
  try {
    const url = (path: string) =>
      `/api/nodes/${encodeURIComponent(nodeId.value)}/fs/list?path=${encodeURIComponent(path)}`

    const res = await apiFetch<FsListResponse>(url(p))
    currentPath.value = res.path
    entries.value = res.entries
    saveLastDir(nodeId.value, currentPath.value)
    if (isSingleDirMode.value) {
      singleDirStatus.value = 'ok'
      singleDirMessage.value = ''
      singleDirValidatedPath.value = normalizePath(res.path)
      singleDirNotFoundConfirmPath.value = ''
    }
  } catch (error) {
    const info = toApiErrorInfo(error, t)
    const code = info.code
    const msgLower = info.message.toLowerCase()
    if (isSingleDirMode.value) {
      const errorText = info.message || t('errors.fsListFailed')

      if (code === 'path_not_found') {
        singleDirStatus.value = 'not_found'
        singleDirMessage.value = ''
        entries.value = []
        return
      }
      if (code === 'permission_denied') {
        singleDirStatus.value = 'permission_denied'
        singleDirMessage.value = errorText
        entries.value = []
        return
      }
      if (code === 'agent_offline') {
        singleDirStatus.value = 'agent_offline'
        singleDirMessage.value = errorText
        entries.value = []
        return
      }
      if (code === 'not_directory') {
        singleDirStatus.value = 'not_directory'
        singleDirMessage.value = errorText
        entries.value = []
        return
      }

      if (code === 'agent_fs_list_failed') {
        if (msgLower.includes('no such file') || msgLower.includes('not found') || msgLower.includes('cannot find the')) {
          singleDirStatus.value = 'not_found'
          singleDirMessage.value = ''
          entries.value = []
          return
        }
        if (msgLower.includes('permission denied') || msgLower.includes('access is denied')) {
          singleDirStatus.value = 'permission_denied'
          singleDirMessage.value = errorText
          entries.value = []
          return
        }
        if (msgLower.includes('not a directory')) {
          singleDirStatus.value = 'not_directory'
          singleDirMessage.value = errorText
          entries.value = []
          return
        }
      }

      singleDirStatus.value = 'error'
      singleDirMessage.value = errorText
      entries.value = []
      return
    }

    const shouldFallback =
      code === 'not_directory' || (code === 'agent_fs_list_failed' && msgLower.includes('not a directory'))

    if (shouldFallback) {
      const parent = computeParentPath(p)
      if (parent && parent !== p) {
        try {
          const res = await apiFetch<FsListResponse>(
            `/api/nodes/${encodeURIComponent(nodeId.value)}/fs/list?path=${encodeURIComponent(parent)}`,
          )
          currentPath.value = res.path
          entries.value = res.entries
          saveLastDir(nodeId.value, currentPath.value)
          return
        } catch (error2) {
          message.error(formatToastError(t('errors.fsListFailed'), error2, t))
          return
        }
      }
    }

    message.error(formatToastError(t('errors.fsListFailed'), error, t))
  } finally {
    loading.value = false
  }
}

function open(nextNodeId: 'hub' | string, initialPath?: string | FsPathPickerOpenOptions): void {
  const opts =
    typeof initialPath === 'string' ? ({ path: initialPath } satisfies FsPathPickerOpenOptions) : initialPath

  nodeId.value = nextNodeId
  pickerMode.value = opts?.mode ?? 'multi_paths'

  const explicitPath = opts?.path?.trim()
  if (explicitPath) {
    currentPath.value = explicitPath
  } else {
    const remembered = loadLastDir(nextNodeId)
    currentPath.value = remembered ?? '/'
  }
  entries.value = []
  checked.value = []
  resetAllFilters()
  singleDirStatus.value = 'unknown'
  singleDirMessage.value = ''
  filtersPopoverOpen.value = false
  filtersDrawerOpen.value = false
  pickCurrentDirConfirmOpen.value = false
  show.value = true
  void refresh()
}

function emitPicked(paths: string[]): void {
  show.value = false
  pickCurrentDirConfirmOpen.value = false
  emit('picked', paths)
}

function requestPickCurrentDir(): void {
  if (isSingleDirMode.value) return

  const p = currentDirNormalized.value
  if (!p) {
    message.error(t('errors.fsPathRequired'))
    return
  }

  if (selectedCount.value === 0) {
    emitPicked([p])
    return
  }

  pickCurrentDirConfirmOpen.value = true
}

function confirmPickCurrentDirOnly(): void {
  const p = currentDirNormalized.value
  if (!p) {
    message.error(t('errors.fsPathRequired'))
    return
  }
  emitPicked([p])
}

function confirmPickCurrentDirWithSelected(): void {
  const p = currentDirNormalized.value
  if (!p) {
    message.error(t('errors.fsPathRequired'))
    return
  }
  const out = uniqueNormalizedPaths([p, ...selectedUnique.value])
  emitPicked(out)
}

function up(): void {
  currentPath.value = computeParentPath(currentPath.value)
  void refresh()
}

async function pick(): Promise<void> {
  if (isSingleDirMode.value) {
    let p = normalizePath(currentPath.value)
    if (!p) {
      message.error(t('errors.fsPathRequired'))
      return
    }

    const validated = normalizePath(singleDirValidatedPath.value)
    if (!validated || validated !== p) {
      await refresh()
      p = normalizePath(currentPath.value)
      if (!p) {
        message.error(t('errors.fsPathRequired'))
        return
      }
    }

    if (singleDirStatus.value === 'not_found') {
      if (singleDirNotFoundConfirmPath.value !== p) {
        singleDirNotFoundConfirmPath.value = p
        return
      }
    } else if (singleDirStatus.value !== 'ok') {
      return
    }

    show.value = false
    emit('picked', [p])
    return
  }

  const unique = Array.from(new Set(checked.value.map((v) => v.trim()).filter((v) => v.length > 0)))
  if (unique.length === 0) {
    message.error(t('errors.sourcePathsRequired'))
    return
  }
  show.value = false
  emit('picked', unique)
}

const tableData = computed(() => {
  if (isSingleDirMode.value) {
    const dirs = entries.value.filter((e) => e.kind === 'dir')
    dirs.sort((a, b) => a.name.localeCompare(b.name))
    return dirs
  }
  return visibleEntries.value
})

const columns = computed<DataTableColumns<FsListEntry>>(() => {
  const nameColumn = {
    title: t('common.name'),
    key: 'name',
    render(row: FsListEntry) {
      const label = row.kind === 'dir' ? `📁 ${row.name}` : row.kind === 'symlink' ? `🔗 ${row.name}` : `📄 ${row.name}`

      const nameNode =
        row.kind === 'dir'
          ? h(
              'button',
              {
                class: 'text-left w-full text-[var(--n-primary-color)] hover:underline truncate',
                onClick: () => {
                  currentPath.value = row.path
                  void refresh()
                },
              },
              label,
            )
          : h('div', { class: 'truncate' }, label)

      if (isDesktop.value) return nameNode

      const parts: string[] = []
      parts.push(kindLabel(row.kind))
      if (row.kind === 'file') parts.push(formatBytes(row.size))
      parts.push(formatMtimeMobile(row.mtime))
      const meta = parts.join(' · ')

      return h('div', { class: 'space-y-1 min-w-0' }, [
        h('div', { class: 'min-w-0' }, [nameNode]),
        h('div', { class: 'text-xs opacity-70 truncate' }, meta),
      ])
    },
  } as const

  if (isSingleDirMode.value) {
    return [
      nameColumn,
      ...(isDesktop.value
        ? ([
            {
              title: t('common.modified'),
              key: 'mtime',
              width: 190,
              render: (row: FsListEntry) => formatMtimeDesktop(row.mtime),
            },
          ] as const)
        : []),
    ]
  }

  return [
    { type: 'selection' },
    nameColumn,
    ...(isDesktop.value
      ? ([
          {
            title: t('common.type'),
            key: 'kind',
            width: 110,
            render: (row: FsListEntry) => kindLabel(row.kind),
          },
          {
            title: t('common.size'),
            key: 'size',
            width: 120,
            align: 'right',
            render: (row: FsListEntry) => (row.kind === 'file' ? formatBytes(row.size) : '-'),
          },
          {
            title: t('common.modified'),
            key: 'mtime',
            width: 190,
            render: (row: FsListEntry) => formatMtimeDesktop(row.mtime),
          },
        ] as const)
      : []),
  ]
})

defineExpose<FsPathPickerModalExpose>({ open })
</script>

<template>
  <n-modal
    v-model:show="show"
    preset="card"
    :style="modalStyle"
    :content-style="{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden', minHeight: 0 }"
    :title="modalTitle"
  >
    <div class="flex flex-col gap-3 flex-1 min-h-0">
      <div class="space-y-2">
        <PickerPathBarInput
          ref="pathBar"
          v-model:value="currentPathModel"
          :placeholder="t('fsPicker.currentPath')"
          :up-title="t('fsPicker.up')"
          :refresh-title="t('common.refresh')"
          @up="up"
          @refresh="refresh"
          @enter="refresh"
          @navigate="onPathBarNavigate"
        />
        <n-alert v-if="isSingleDirMode && singleDirStatus === 'not_found'" type="warning" :bordered="false">
          {{ t('fsPicker.dirNotFoundWillCreate') }}
        </n-alert>
        <n-alert
          v-else-if="isSingleDirMode && singleDirStatus !== 'ok' && singleDirStatus !== 'unknown'"
          type="error"
          :bordered="false"
        >
          {{ singleDirMessage || t('errors.fsListFailed') }}
        </n-alert>
      </div>

      <div v-if="!isSingleDirMode" class="flex items-center gap-2">
        <n-input
          v-model:value="searchDraft"
          class="flex-1 min-w-0"
          :placeholder="t('fsPicker.searchPlaceholder')"
          @keyup.enter="applySearch"
        >
          <template #suffix>
            <n-button
              size="tiny"
              quaternary
              :disabled="!hasSearchDraftChanges"
              :title="t('common.search')"
              @click="applySearch"
            >
              <template #icon>
                <n-icon><search-outline /></n-icon>
              </template>
            </n-button>
          </template>
        </n-input>

        <n-popover
          v-if="isDesktop"
          v-model:show="filtersPopoverOpen"
          trigger="click"
          placement="bottom-end"
          :show-arrow="false"
        >
          <template #trigger>
            <n-badge :value="activeFilterCount" :show="activeFilterCount > 0">
              <n-button size="small" secondary :title="t('common.filters')">
                <template #icon>
                  <n-icon><filter-outline /></n-icon>
                </template>
              </n-button>
            </n-badge>
          </template>
          <div class="w-80">
            <n-form label-placement="top" size="small">
              <n-form-item :label="t('common.type')">
                <n-select v-model:value="kindFilter" :options="kindOptions" />
              </n-form-item>
              <n-form-item :label="t('common.hideDotfiles')">
                <n-switch v-model:value="hideDotfiles" />
              </n-form-item>
              <n-form-item :label="t('common.fileSize')">
                <div class="space-y-2">
                  <div class="grid grid-cols-2 gap-2">
                    <n-input-number
                      v-model:value="sizeMinDraft"
                      :min="0"
                      :placeholder="t('common.min')"
                      class="w-full"
                    />
                    <n-input-number
                      v-model:value="sizeMaxDraft"
                      :min="0"
                      :placeholder="t('common.max')"
                      class="w-full"
                    />
                  </div>
                  <div class="grid grid-cols-[1fr_auto] gap-2 items-center">
                    <n-select v-model:value="sizeUnitDraft" :options="sizeUnitOptions" />
                    <n-button size="small" @click="applySizeFilter">{{ t('common.apply') }}</n-button>
                  </div>
                </div>
              </n-form-item>
              <n-form-item :label="t('common.typeSort')">
                <n-select v-model:value="typeSort" :options="typeSortOptions" />
              </n-form-item>
            </n-form>
            <div class="flex justify-end">
              <n-button size="tiny" tertiary @click="resetAllFilters">{{ t('common.clear') }}</n-button>
            </div>
          </div>
        </n-popover>

        <n-badge v-else :value="activeFilterCount" :show="activeFilterCount > 0">
          <n-button size="small" secondary :title="t('common.filters')" @click="filtersDrawerOpen = true">
            <template #icon>
              <n-icon><filter-outline /></n-icon>
            </template>
          </n-button>
        </n-badge>
      </div>

      <div v-if="!isSingleDirMode && activeChips.length > 0" class="flex flex-wrap gap-2 items-center">
        <n-tag v-for="chip in activeChips" :key="chip.key" size="small" closable @close="chip.onClose">
          {{ chip.label }}
        </n-tag>
        <n-button size="tiny" tertiary @click="resetAllFilters">{{ t('common.clear') }}</n-button>
      </div>

      <n-drawer v-if="!isSingleDirMode" v-model:show="filtersDrawerOpen" placement="bottom" height="80vh">
        <n-drawer-content :title="t('common.filters')" closable>
          <n-form label-placement="top" size="small">
            <n-form-item :label="t('common.type')">
              <n-select v-model:value="kindFilter" :options="kindOptions" />
            </n-form-item>
            <n-form-item :label="t('common.hideDotfiles')">
              <n-switch v-model:value="hideDotfiles" />
            </n-form-item>
            <n-form-item :label="t('common.fileSize')">
              <div class="space-y-2">
                <div class="grid grid-cols-2 gap-2">
                  <n-input-number
                    v-model:value="sizeMinDraft"
                    :min="0"
                    :placeholder="t('common.min')"
                    class="w-full"
                  />
                  <n-input-number
                    v-model:value="sizeMaxDraft"
                    :min="0"
                    :placeholder="t('common.max')"
                    class="w-full"
                  />
                </div>
                <div class="grid grid-cols-[1fr_auto] gap-2 items-center">
                  <n-select v-model:value="sizeUnitDraft" :options="sizeUnitOptions" />
                  <n-button size="small" @click="applySizeFilter">{{ t('common.apply') }}</n-button>
                </div>
              </div>
            </n-form-item>
            <n-form-item :label="t('common.typeSort')">
              <n-select v-model:value="typeSort" :options="typeSortOptions" />
            </n-form-item>
          </n-form>
          <div class="flex justify-end gap-2 pt-2">
            <n-button tertiary @click="resetAllFilters">{{ t('common.clear') }}</n-button>
            <n-button type="primary" @click="filtersDrawerOpen = false">{{ t('common.done') }}</n-button>
          </div>
        </n-drawer-content>
      </n-drawer>

      <div ref="tableContainerEl" class="flex-1 min-h-0 overflow-hidden">
        <n-data-table
          :loading="loading"
          :columns="columns"
          :data="tableData"
          :row-key="(row) => row.path"
          v-model:checked-row-keys="checked"
          :max-height="tableBodyMaxHeightPx || undefined"
        />
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-end sm:justify-between gap-2">
        <div v-if="!isSingleDirMode && isDesktop" class="text-xs opacity-70">
          {{ t('fsPicker.selectedCount', { count: selectedCount }) }}
        </div>
        <div class="flex items-center justify-end gap-2 ml-auto">
          <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
          <n-button v-if="!isSingleDirMode" :disabled="!currentPath.trim()" @click="requestPickCurrentDir">
            {{ t('fsPicker.selectCurrentDir') }}
          </n-button>
          <n-button
            v-if="isSingleDirMode"
            type="primary"
            :loading="loading"
            :disabled="!currentPath.trim() || ['permission_denied', 'not_directory', 'agent_offline', 'error'].includes(singleDirStatus)"
            @click="pick"
          >
            {{ singleDirConfirmLabel }}
          </n-button>
          <n-badge
            v-else-if="!isDesktop"
            :value="selectedCount"
            :show="selectedCount > 0"
          >
            <n-button type="primary" :disabled="checked.length === 0" @click="pick">
              {{ t('fsPicker.addSelected') }}
            </n-button>
          </n-badge>
          <n-button v-else type="primary" :disabled="checked.length === 0" @click="pick">
            {{ t('fsPicker.addSelected') }}
          </n-button>
        </div>
      </div>
    </template>
  </n-modal>

  <n-modal
    v-if="isDesktop"
    v-model:show="pickCurrentDirConfirmOpen"
    preset="card"
    :style="{ width: MODAL_WIDTH.md, maxHeight: MODAL_HEIGHT.max }"
    :content-style="{ overflow: 'auto', minHeight: 0 }"
    :title="t('fsPicker.confirm.title')"
  >
    <div class="space-y-3">
      <div class="space-y-1">
        <div class="text-xs opacity-70">{{ t('fsPicker.confirm.currentDir') }}</div>
        <div class="font-mono text-xs break-all rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5">
          {{ currentDirNormalized }}
        </div>
      </div>

      <div class="space-y-1">
        <div class="text-xs opacity-70">{{ t('fsPicker.confirm.selectedItems', { count: selectedUnique.length }) }}</div>
        <div class="max-h-[40vh] overflow-auto rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5 space-y-1">
          <div v-for="p in selectedUnique" :key="p" class="font-mono text-xs break-all">
            {{ p }}
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <n-button @click="pickCurrentDirConfirmOpen = false">{{ t('common.cancel') }}</n-button>
        <n-button secondary @click="confirmPickCurrentDirWithSelected">
          {{ t('fsPicker.confirm.withSelected') }}
        </n-button>
        <n-button type="primary" @click="confirmPickCurrentDirOnly">
          {{ t('fsPicker.confirm.onlyCurrent') }}
        </n-button>
      </div>
    </template>
  </n-modal>

  <n-drawer v-else v-model:show="pickCurrentDirConfirmOpen" placement="bottom" height="80vh">
    <n-drawer-content :title="t('fsPicker.confirm.title')" closable>
      <div class="space-y-3">
        <div class="space-y-1">
          <div class="text-xs opacity-70">{{ t('fsPicker.confirm.currentDir') }}</div>
          <div class="font-mono text-xs break-all rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5">
            {{ currentDirNormalized }}
          </div>
        </div>

        <div class="space-y-1">
          <div class="text-xs opacity-70">{{ t('fsPicker.confirm.selectedItems', { count: selectedUnique.length }) }}</div>
          <div class="max-h-[40vh] overflow-auto rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5 space-y-1">
            <div v-for="p in selectedUnique" :key="p" class="font-mono text-xs break-all">
              {{ p }}
            </div>
          </div>
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-3">
        <n-button @click="pickCurrentDirConfirmOpen = false">{{ t('common.cancel') }}</n-button>
        <n-button secondary @click="confirmPickCurrentDirWithSelected">
          {{ t('fsPicker.confirm.withSelected') }}
        </n-button>
        <n-button type="primary" @click="confirmPickCurrentDirOnly">
          {{ t('fsPicker.confirm.onlyCurrent') }}
        </n-button>
      </div>
    </n-drawer-content>
  </n-drawer>
</template>
