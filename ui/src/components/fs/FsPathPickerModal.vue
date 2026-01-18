<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
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
  NInputNumber,
  NModal,
  NPopover,
  NSelect,
  NSwitch,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { ListOutline } from '@vicons/ionicons5'

import { apiFetch } from '@/lib/api'
import { copyText } from '@/lib/clipboard'
import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatBytes } from '@/lib/format'
import { formatToastError, toApiErrorInfo } from '@/lib/errors'
import { formatUnixSecondsYmdHms } from '@/lib/datetime'
import AppEmptyState from '@/components/AppEmptyState.vue'
import PickerPathBarInput, { type PickerPathBarInputExpose } from '@/components/pickers/PickerPathBarInput.vue'
import PickerActiveChipsRow from '@/components/pickers/PickerActiveChipsRow.vue'
import PickerFiltersPopoverDrawer from '@/components/pickers/PickerFiltersPopoverDrawer.vue'
import PickerFooterRow from '@/components/pickers/PickerFooterRow.vue'
import PickerModalCard from '@/components/pickers/PickerModalCard.vue'
import PickerSearchInput from '@/components/pickers/PickerSearchInput.vue'
import PickerShortcutsHelp, { type PickerShortcutItem } from '@/components/pickers/PickerShortcutsHelp.vue'
import { usePickerTableBodyMaxHeightPx } from '@/components/pickers/usePickerTableBodyMaxHeightPx'
import { usePickerKeyboardShortcuts } from '@/components/pickers/usePickerKeyboardShortcuts'
import { useShiftKeyPressed } from '@/components/pickers/useShiftKeyPressed'

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
  next_cursor?: string | null
  total?: number | null
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
const nextCursor = ref<string | null>(null)
const total = ref<number | null>(null)

type ListErrorKind = 'none' | 'agent_offline' | 'permission_denied' | 'not_found' | 'invalid_cursor' | 'error'
const listErrorKind = ref<ListErrorKind>('none')
const listErrorMessage = ref<string>('')

const searchDraft = ref<string>('')
const searchApplied = ref<string>('')
const kindFilter = ref<'all' | 'dir' | 'file' | 'symlink'>('all')
const hideDotfiles = ref<boolean>(false)

type SizeUnit = 'B' | 'KB' | 'MB' | 'GB'

const typeSort = ref<'dir_first' | 'file_first'>('dir_first')

type FsSortBy = 'name' | 'mtime' | 'size'
type FsSortDir = 'asc' | 'desc'

const sortBy = ref<FsSortBy>('name')
const sortDir = ref<FsSortDir>('asc')

const sizeMinDraft = ref<number | null>(null)
const sizeMaxDraft = ref<number | null>(null)
const sizeUnitDraft = ref<SizeUnit>('MB')

const sizeMinApplied = ref<number | null>(null)
const sizeMaxApplied = ref<number | null>(null)
const sizeUnitApplied = ref<SizeUnit>('MB')

const filtersPopoverOpen = ref<boolean>(false)
const filtersDrawerOpen = ref<boolean>(false)
const shortcutsPopoverOpen = ref<boolean>(false)
const shortcutsDrawerOpen = ref<boolean>(false)
const selectionPopoverOpen = ref<boolean>(false)
const selectionDrawerOpen = ref<boolean>(false)
const pickCurrentDirConfirmOpen = ref<boolean>(false)
const loadingMore = ref<boolean>(false)

const lastRangeAnchorPath = ref<string | null>(null)
const { shiftPressed } = useShiftKeyPressed(show)

const PAGE_LIMIT = 200
let listFetchSeq = 0

type SingleDirStatus = 'unknown' | 'ok' | 'not_found' | 'not_directory' | 'permission_denied' | 'agent_offline' | 'error'
const singleDirStatus = ref<SingleDirStatus>('unknown')
const singleDirMessage = ref<string>('')
const singleDirValidatedPath = ref<string>('')
const singleDirNotFoundConfirmPath = ref<string>('')

const { tableContainerEl, tableBodyMaxHeightPx } = usePickerTableBodyMaxHeightPx(show, {
  onOpen: () => {
    // Keep the initial focus on the path input so the first action button doesn't look "selected".
    try {
      pathBar.value?.focus?.()
    } catch {
      // ignore
    }
  },
  onClose: () => {
    pickCurrentDirConfirmOpen.value = false
    shortcutsPopoverOpen.value = false
    shortcutsDrawerOpen.value = false
  },
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
  refreshForFilters()
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

const sortByOptions = computed(() => [
  { label: t('common.name'), value: 'name' as const },
  { label: t('common.modified'), value: 'mtime' as const },
  { label: t('common.size'), value: 'size' as const },
])

const sortDirOptions = computed(() => [
  { label: t('common.asc'), value: 'asc' as const },
  { label: t('common.desc'), value: 'desc' as const },
])

const hasSizeApplied = computed(() => sizeMinApplied.value != null || sizeMaxApplied.value != null)

const activeFilterCount = computed(() => {
  let count = 0
  if (kindFilter.value !== 'all') count += 1
  if (hideDotfiles.value) count += 1
  if (hasSizeApplied.value) count += 1
  if (typeSort.value !== 'dir_first') count += 1
  if (sortBy.value !== 'name' || sortDir.value !== 'asc') count += 1
  return count
})

const hasAnySearchOrFilters = computed(() => searchApplied.value.trim().length > 0 || activeFilterCount.value > 0)

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
  if (q) out.push({ key: 'search', label: `${t('common.search')}: ${q}`, onClose: clearSearchAndRefresh })

  if (kindFilter.value !== 'all') {
    out.push({
      key: 'kind',
      label: `${t('common.type')}: ${kindLabel(kindFilter.value)}`,
      onClose: clearKindFilterAndRefresh,
    })
  }

  if (hideDotfiles.value) {
    out.push({ key: 'dotfiles', label: t('common.hideDotfiles'), onClose: clearDotfilesAndRefresh })
  }

  if (hasSizeApplied.value) {
    out.push({
      key: 'size',
      label: `${t('common.fileSize')}: ${formatSizeRange(sizeMinApplied.value, sizeMaxApplied.value, sizeUnitApplied.value)}`,
      onClose: clearSizeFilterAndRefresh,
    })
  }

  if (typeSort.value !== 'dir_first') {
    out.push({
      key: 'typeSort',
      label: `${t('common.typeSort')}: ${typeSort.value === 'file_first' ? t('common.fileFirst') : t('common.dirFirst')}`,
      onClose: resetTypeSortAndRefresh,
    })
  }

  if (sortBy.value !== 'name' || sortDir.value !== 'asc') {
    out.push({
      key: 'sort',
      label: `${t('common.sort')}: ${sortByLabel(sortBy.value)} · ${sortDirLabel(sortDir.value)}`,
      onClose: resetSortAndRefresh,
    })
  }

  return out
})

const pickerShortcuts = computed<PickerShortcutItem[]>(() => [
  { combo: 'Enter', description: t('pickers.shortcuts.enterDir') },
  { combo: 'Backspace', description: t('pickers.shortcuts.up') },
  { combo: 'Ctrl/Cmd+L', description: t('pickers.shortcuts.focusPath') },
  { combo: 'Esc', description: t('pickers.shortcuts.close') },
  { combo: 'Shift', description: t('pickers.shortcuts.rangeSelect') },
])

const pickerShortcutsNote = computed(() => t('pickers.shortcuts.note'))

const visibleEntries = computed(() => entries.value)

const checkedSet = computed(() => new Set(checked.value))

function rowClassName(row: FsListEntry): string {
  if (checkedSet.value.has(row.path)) return 'app-picker-row app-picker-row--checked'
  return 'app-picker-row'
}

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

function sortByLabel(v: FsSortBy): string {
  if (v === 'mtime') return t('common.modified')
  if (v === 'size') return t('common.size')
  return t('common.name')
}

function sortDirLabel(v: FsSortDir): string {
  return v === 'desc' ? t('common.desc') : t('common.asc')
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
  refreshForFilters()
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

function resetSort(): void {
  sortBy.value = 'name'
  sortDir.value = 'asc'
}

function resetAllFilters(): void {
  clearSearch()
  clearKindFilter()
  clearDotfiles()
  clearSizeFilter()
  sizeUnitDraft.value = 'MB'
  sizeUnitApplied.value = 'MB'
  resetTypeSort()
  resetSort()
}

function refreshForFilters(): void {
  if (!show.value) return
  if (isSingleDirMode.value) return
  void refresh()
}

function clearSearchAndRefresh(): void {
  clearSearch()
  refreshForFilters()
}

function clearKindFilterAndRefresh(): void {
  clearKindFilter()
  refreshForFilters()
}

function clearDotfilesAndRefresh(): void {
  clearDotfiles()
  refreshForFilters()
}

function clearSizeFilterAndRefresh(): void {
  clearSizeFilter()
  refreshForFilters()
}

function resetTypeSortAndRefresh(): void {
  resetTypeSort()
  refreshForFilters()
}

function resetSortAndRefresh(): void {
  resetSort()
  refreshForFilters()
}

function resetAllFiltersAndRefresh(): void {
  resetAllFilters()
  refreshForFilters()
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

const FILTERS_KEY_PREFIX = 'bastion.fsPicker.filters.'

type PersistedFiltersV1 = {
  v: 1
  search?: string
  kind?: 'all' | 'dir' | 'file' | 'symlink'
  hideDotfiles?: boolean
  typeSort?: 'dir_first' | 'file_first'
  sortBy?: FsSortBy
  sortDir?: FsSortDir
  sizeMin?: number | null
  sizeMax?: number | null
  sizeUnit?: SizeUnit
}

function filtersStorageKey(id: string): string {
  return `${FILTERS_KEY_PREFIX}${encodeURIComponent(id)}`
}

function loadPersistedFilters(id: string): PersistedFiltersV1 | null {
  try {
    const raw = localStorage.getItem(filtersStorageKey(id))
    if (!raw || !raw.trim()) return null
    const parsed = JSON.parse(raw) as Partial<PersistedFiltersV1>
    if (!parsed || typeof parsed !== 'object' || parsed.v !== 1) return null
    return parsed as PersistedFiltersV1
  } catch {
    return null
  }
}

function savePersistedFilters(id: string, state: PersistedFiltersV1): void {
  try {
    localStorage.setItem(filtersStorageKey(id), JSON.stringify(state))
  } catch {
    // ignore
  }
}

function normalizeKind(raw: unknown): NonNullable<PersistedFiltersV1['kind']> {
  if (raw === 'all' || raw === 'dir' || raw === 'file' || raw === 'symlink') return raw
  return 'all'
}

function normalizeTypeSort(raw: unknown): NonNullable<PersistedFiltersV1['typeSort']> {
  if (raw === 'dir_first' || raw === 'file_first') return raw
  return 'dir_first'
}

function normalizeSortBy(raw: unknown): FsSortBy {
  if (raw === 'name' || raw === 'mtime' || raw === 'size') return raw
  return 'name'
}

function normalizeSortDir(raw: unknown): FsSortDir {
  if (raw === 'asc' || raw === 'desc') return raw
  return 'asc'
}

function normalizeSizeUnit(raw: unknown): SizeUnit {
  if (raw === 'B' || raw === 'KB' || raw === 'MB' || raw === 'GB') return raw
  return 'MB'
}

function normalizeNumberOrNull(raw: unknown): number | null {
  if (raw == null) return null
  const n = typeof raw === 'number' ? raw : typeof raw === 'string' ? Number(raw) : Number.NaN
  if (!Number.isFinite(n)) return null
  return n
}

function applyPersistedFilters(state: PersistedFiltersV1): void {
  const search = (typeof state.search === 'string' ? state.search : '').trim()
  searchDraft.value = search
  searchApplied.value = search

  kindFilter.value = normalizeKind(state.kind)
  hideDotfiles.value = Boolean(state.hideDotfiles)
  typeSort.value = normalizeTypeSort(state.typeSort)
  sortBy.value = normalizeSortBy(state.sortBy)
  sortDir.value = normalizeSortDir(state.sortDir)

  const sizeMin = normalizeNumberOrNull(state.sizeMin)
  const sizeMax = normalizeNumberOrNull(state.sizeMax)
  const sizeUnit = normalizeSizeUnit(state.sizeUnit)
  sizeMinDraft.value = sizeMin
  sizeMaxDraft.value = sizeMax
  sizeMinApplied.value = sizeMin
  sizeMaxApplied.value = sizeMax
  sizeUnitDraft.value = sizeUnit
  sizeUnitApplied.value = sizeUnit
}

function persistCurrentFilters(): void {
  if (!show.value) return
  if (isSingleDirMode.value) return
  const id = nodeId.value
  savePersistedFilters(id, {
    v: 1,
    search: searchApplied.value.trim(),
    kind: kindFilter.value,
    hideDotfiles: hideDotfiles.value,
    typeSort: typeSort.value,
    sortBy: sortBy.value,
    sortDir: sortDir.value,
    sizeMin: sizeMinApplied.value,
    sizeMax: sizeMaxApplied.value,
    sizeUnit: sizeUnitApplied.value,
  })
}

watch(
  [
    () => show.value,
    () => nodeId.value,
    () => pickerMode.value,
    () => searchApplied.value,
    () => kindFilter.value,
    () => hideDotfiles.value,
    () => typeSort.value,
    () => sortBy.value,
    () => sortDir.value,
    () => sizeMinApplied.value,
    () => sizeMaxApplied.value,
    () => sizeUnitApplied.value,
  ],
  () => {
    persistCurrentFilters()
  },
)

async function copyCurrentPath(): Promise<void> {
  const p = normalizePath(currentPath.value)
  if (!p) {
    message.error(t('errors.fsPathRequired'))
    return
  }
  const ok = await copyText(p)
  if (ok) message.success(t('messages.copied'))
  else message.error(t('errors.copyFailed'))
}

async function refresh(): Promise<void> {
  const p = normalizePath(currentPath.value)
  if (!p) {
    message.error(t('errors.fsPathRequired'))
    return
  }

  const seq = ++listFetchSeq

  loading.value = true
  loadingMore.value = false
  nextCursor.value = null
  total.value = null
  if (isSingleDirMode.value) {
    singleDirStatus.value = 'unknown'
    singleDirMessage.value = ''
    singleDirValidatedPath.value = p
    singleDirNotFoundConfirmPath.value = ''
  } else {
    listErrorKind.value = 'none'
    listErrorMessage.value = ''
  }
  try {
    const res = await apiFetch<FsListResponse>(buildFsListUrl(p, null))
    if (seq !== listFetchSeq) return
    currentPath.value = res.path
    entries.value = res.entries
    nextCursor.value = normalizeCursor(res.next_cursor)
    total.value = typeof res.total === 'number' ? res.total : null
    saveLastDir(nodeId.value, currentPath.value)
    if (isSingleDirMode.value) {
      singleDirStatus.value = 'ok'
      singleDirMessage.value = ''
      singleDirValidatedPath.value = normalizePath(res.path)
      singleDirNotFoundConfirmPath.value = ''
    }
  } catch (error) {
    if (seq !== listFetchSeq) return
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
          const res = await apiFetch<FsListResponse>(buildFsListUrl(parent, null))
          if (seq !== listFetchSeq) return
          currentPath.value = res.path
          entries.value = res.entries
          nextCursor.value = normalizeCursor(res.next_cursor)
          total.value = typeof res.total === 'number' ? res.total : null
          saveLastDir(nodeId.value, currentPath.value)
          listErrorKind.value = 'none'
          listErrorMessage.value = ''
          return
        } catch (error2) {
          const info2 = toApiErrorInfo(error2, t)
          listErrorKind.value = 'error'
          listErrorMessage.value = info2.message || t('errors.fsListFailed')
          entries.value = []
          return
        }
      }
    }

    if (code === 'agent_offline') {
      listErrorKind.value = 'agent_offline'
      listErrorMessage.value = info.message
      entries.value = []
      return
    }
    if (code === 'permission_denied') {
      listErrorKind.value = 'permission_denied'
      listErrorMessage.value = info.message
      entries.value = []
      return
    }
    if (code === 'path_not_found') {
      listErrorKind.value = 'not_found'
      listErrorMessage.value = info.message
      entries.value = []
      return
    }
    if (code === 'invalid_cursor') {
      listErrorKind.value = 'invalid_cursor'
      listErrorMessage.value = info.message
      entries.value = []
      return
    }

    listErrorKind.value = 'error'
    listErrorMessage.value = info.message || t('errors.fsListFailed')
    entries.value = []
  } finally {
    if (seq === listFetchSeq) loading.value = false
  }
}

async function loadMore(): Promise<void> {
  if (loading.value || loadingMore.value) return
  const cursor = nextCursor.value
  if (!cursor) return

  const p = normalizePath(currentPath.value)
  if (!p) return

  const seq = ++listFetchSeq
  loadingMore.value = true
  try {
    const res = await apiFetch<FsListResponse>(buildFsListUrl(p, cursor))
    if (seq !== listFetchSeq) return

    currentPath.value = res.path
    entries.value = [...entries.value, ...res.entries]
    nextCursor.value = normalizeCursor(res.next_cursor)
    total.value = typeof res.total === 'number' ? res.total : null
    saveLastDir(nodeId.value, currentPath.value)
  } catch (error) {
    if (seq !== listFetchSeq) return
    const info = toApiErrorInfo(error, t)
    if (info.code === 'invalid_cursor' || info.code === 'agent_offline' || info.code === 'permission_denied') {
      listErrorKind.value = info.code === 'agent_offline' ? 'agent_offline' : info.code === 'permission_denied' ? 'permission_denied' : 'invalid_cursor'
      listErrorMessage.value = info.message
      nextCursor.value = null
      return
    }
    message.error(formatToastError(t('errors.fsListFailed'), error, t))
  } finally {
    if (seq === listFetchSeq) loadingMore.value = false
  }
}

function buildFsListUrl(path: string, cursor: string | null): string {
  const params = new URLSearchParams()
  params.set('path', path)
  params.set('limit', String(PAGE_LIMIT))
  if (cursor && cursor.trim()) params.set('cursor', cursor.trim())
  params.set('sort_by', sortBy.value)
  params.set('sort_dir', sortDir.value)

  const effectiveKind = isSingleDirMode.value ? 'dir' : kindFilter.value === 'all' ? null : kindFilter.value
  if (effectiveKind) params.set('kind', effectiveKind)

  if (!isSingleDirMode.value) {
    const q = searchApplied.value.trim()
    if (q) params.set('q', q)
    if (hideDotfiles.value) params.set('hide_dotfiles', 'true')
    if (typeSort.value) params.set('type_sort', typeSort.value)

    const mult = sizeUnitMultiplier(sizeUnitApplied.value)
    const minBytes =
      sizeMinApplied.value != null && Number.isFinite(sizeMinApplied.value)
        ? Math.max(0, Math.floor(sizeMinApplied.value * mult))
        : null
    const maxBytes =
      sizeMaxApplied.value != null && Number.isFinite(sizeMaxApplied.value)
        ? Math.max(0, Math.floor(sizeMaxApplied.value * mult))
        : null

    if (minBytes != null) params.set('size_min_bytes', String(minBytes))
    if (maxBytes != null) params.set('size_max_bytes', String(maxBytes))
  }

  return `/api/nodes/${encodeURIComponent(nodeId.value)}/fs/list?${params.toString()}`
}

function normalizeCursor(v?: string | null): string | null {
  const t = (v ?? '').trim()
  return t ? t : null
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
  lastRangeAnchorPath.value = null
  nextCursor.value = null
  total.value = null
  loadingMore.value = false
  if (isSingleDirMode.value) {
    resetAllFilters()
  } else {
    const persisted = loadPersistedFilters(nextNodeId)
    if (persisted) applyPersistedFilters(persisted)
    else resetAllFilters()
  }
  listErrorKind.value = 'none'
  listErrorMessage.value = ''
  singleDirStatus.value = 'unknown'
  singleDirMessage.value = ''
  filtersPopoverOpen.value = false
  filtersDrawerOpen.value = false
  selectionPopoverOpen.value = false
  selectionDrawerOpen.value = false
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
    return entries.value.filter((e) => e.kind === 'dir')
  }
  return visibleEntries.value
})

const showListErrorState = computed(
  () => !loading.value && !isSingleDirMode.value && listErrorKind.value !== 'none',
)
const showListErrorBanner = computed(() => showListErrorState.value && tableData.value.length > 0)
const showListErrorEmptyState = computed(() => showListErrorState.value && tableData.value.length === 0)
const showEmptyDirState = computed(
  () =>
    !loading.value &&
    !isSingleDirMode.value &&
    listErrorKind.value === 'none' &&
    tableData.value.length === 0 &&
    !hasAnySearchOrFilters.value,
)
const showNoMatchesState = computed(
  () =>
    !loading.value &&
    !isSingleDirMode.value &&
    listErrorKind.value === 'none' &&
    tableData.value.length === 0 &&
    hasAnySearchOrFilters.value,
)

function loadedRowPaths(): string[] {
  if (isSingleDirMode.value) return []
  return tableData.value.map((e) => normalizePath(e.path)).filter((v) => v.length > 0)
}

function clearSelection(): void {
  checked.value = []
  lastRangeAnchorPath.value = null
}

function selectAllLoadedRows(): void {
  checked.value = uniqueNormalizedPaths([...checked.value, ...loadedRowPaths()])
}

function invertLoadedRowsSelection(): void {
  const loaded = loadedRowPaths()
  const loadedSet = new Set(loaded)
  const current = new Set(selectedUnique.value)

  // Toggle only the loaded rows; keep selection from other directories intact.
  for (const p of loadedSet) {
    if (current.has(p)) current.delete(p)
    else current.add(p)
  }

  checked.value = Array.from(current)
}

function updateCheckedRowKeys(keys: Array<string | number>): void {
  if (isSingleDirMode.value) return

  const loaded = loadedRowPaths()
  const loadedSet = new Set(loaded)

  const desiredLoaded = new Set(keys.map((k) => normalizePath(String(k))).filter((v) => v.length > 0))
  const prev = new Set(selectedUnique.value)

  const next = new Set(prev)
  for (const p of loadedSet) {
    if (desiredLoaded.has(p)) next.add(p)
    else next.delete(p)
  }

  const added: string[] = []
  const removed: string[] = []
  for (const p of loaded) {
    const was = prev.has(p)
    const now = next.has(p)
    if (!was && now) added.push(p)
    else if (was && !now) removed.push(p)
  }

  if (shiftPressed.value && lastRangeAnchorPath.value && added.length === 1 && removed.length === 0) {
    const a = lastRangeAnchorPath.value
    const b = added[0]
    if (!b) {
      checked.value = Array.from(next)
      return
    }
    const idxA = loaded.indexOf(a)
    const idxB = loaded.indexOf(b)
    if (idxA !== -1 && idxB !== -1) {
      const from = Math.min(idxA, idxB)
      const to = Math.max(idxA, idxB)
      for (const p of loaded.slice(from, to + 1)) next.add(p)
    }
  }

  if (added.length === 1 && removed.length === 0) lastRangeAnchorPath.value = added[0] ?? null
  else if (removed.length === 1 && added.length === 0) lastRangeAnchorPath.value = removed[0] ?? null
  else if (next.size === 0) lastRangeAnchorPath.value = null

  checked.value = Array.from(next)
}

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

function isShortcutBlocked(): boolean {
  return (
    pickCurrentDirConfirmOpen.value ||
    shortcutsPopoverOpen.value ||
    shortcutsDrawerOpen.value ||
    selectionPopoverOpen.value ||
    selectionDrawerOpen.value ||
    filtersPopoverOpen.value ||
    filtersDrawerOpen.value
  )
}

function isTableFocused(): boolean {
  const active = document.activeElement as HTMLElement | null
  return Boolean(active && tableContainerEl.value && tableContainerEl.value.contains(active))
}

function enterSelectedDirByKeyboard(): boolean {
  if (isSingleDirMode.value) return false
  if (!isTableFocused()) return false
  if (checked.value.length !== 1) return false

  const key = normalizePath(checked.value[0] ?? '')
  if (!key) return false

  const row = entries.value.find((e) => normalizePath(e.path) === key)
  if (!row || row.kind !== 'dir') return false

  currentPath.value = row.path
  void refresh()
  return true
}

usePickerKeyboardShortcuts(show, {
  onEscape: () => {
    if (pickCurrentDirConfirmOpen.value) {
      pickCurrentDirConfirmOpen.value = false
      return
    }
    if (shortcutsDrawerOpen.value) {
      shortcutsDrawerOpen.value = false
      return
    }
    if (shortcutsPopoverOpen.value) {
      shortcutsPopoverOpen.value = false
      return
    }
    if (selectionDrawerOpen.value) {
      selectionDrawerOpen.value = false
      return
    }
    if (selectionPopoverOpen.value) {
      selectionPopoverOpen.value = false
      return
    }
    if (filtersDrawerOpen.value) {
      filtersDrawerOpen.value = false
      return
    }
    if (filtersPopoverOpen.value) {
      filtersPopoverOpen.value = false
      return
    }
    show.value = false
  },
  onFocusPath: () => {
    try {
      pathBar.value?.focus?.()
    } catch {
      // ignore
    }
  },
  onBackspace: () => {
    if (isShortcutBlocked()) return
    up()
  },
  onEnter: () => {
    if (isShortcutBlocked()) return false
    return enterSelectedDirByKeyboard()
  },
})

defineExpose<FsPathPickerModalExpose>({ open })
</script>

<template>
  <PickerModalCard
    v-model:show="show"
    :style="modalStyle"
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
        <PickerSearchInput
          v-model:value="searchDraft"
          :placeholder="t('fsPicker.searchPlaceholder')"
          :search-disabled="!hasSearchDraftChanges"
          @search="applySearch"
        />

        <PickerFiltersPopoverDrawer
          :is-desktop="isDesktop"
          :title="t('common.filters')"
          :active-count="activeFilterCount"
          v-model:popover-open="filtersPopoverOpen"
          v-model:drawer-open="filtersDrawerOpen"
        >
          <n-form label-placement="top" size="small">
            <n-form-item :label="t('common.type')">
              <n-select v-model:value="kindFilter" :options="kindOptions" @update:value="refreshForFilters" />
            </n-form-item>
            <n-form-item :label="t('common.hideDotfiles')">
              <n-switch v-model:value="hideDotfiles" @update:value="refreshForFilters" />
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
              <n-select v-model:value="typeSort" :options="typeSortOptions" @update:value="refreshForFilters" />
            </n-form-item>
            <n-form-item :label="t('common.sort')">
              <div class="grid grid-cols-[minmax(0,1fr)_7rem] gap-2">
                <n-select
                  v-model:value="sortBy"
                  class="w-full"
                  :options="sortByOptions"
                  :consistent-menu-width="false"
                  @update:value="refreshForFilters"
                />
                <n-select
                  v-model:value="sortDir"
                  class="w-full"
                  :options="sortDirOptions"
                  :consistent-menu-width="false"
                  @update:value="refreshForFilters"
                />
              </div>
            </n-form-item>
          </n-form>

          <template #popoverFooter>
            <div class="flex justify-end">
              <n-button size="tiny" tertiary @click="resetAllFiltersAndRefresh">{{ t('common.clear') }}</n-button>
            </div>
          </template>

          <template #drawerFooter>
            <div class="flex justify-end gap-2 pt-2">
              <n-button tertiary @click="resetAllFiltersAndRefresh">{{ t('common.clear') }}</n-button>
              <n-button type="primary" @click="filtersDrawerOpen = false">{{ t('common.done') }}</n-button>
            </div>
          </template>
        </PickerFiltersPopoverDrawer>

        <PickerShortcutsHelp
          :is-desktop="isDesktop"
          :title="t('pickers.shortcuts.title')"
          :shortcuts="pickerShortcuts"
          :note="pickerShortcutsNote"
          v-model:popover-open="shortcutsPopoverOpen"
          v-model:drawer-open="shortcutsDrawerOpen"
        />
      </div>

      <PickerActiveChipsRow
        v-if="!isSingleDirMode"
        :chips="activeChips"
        :clear-label="t('common.clear')"
        @clear="resetAllFiltersAndRefresh"
      />

      <div class="flex-1 min-h-0 flex flex-col overflow-hidden gap-2">
        <n-alert v-if="showListErrorBanner" type="error" :bordered="false">
          <div class="space-y-2">
            <div>{{ listErrorMessage || t('errors.fsListFailed') }}</div>
            <div class="flex flex-wrap gap-2">
              <n-button size="tiny" secondary @click="refresh">{{ t('common.refresh') }}</n-button>
              <n-button size="tiny" secondary @click="up">{{ t('fsPicker.up') }}</n-button>
              <n-button size="tiny" secondary @click="copyCurrentPath">{{ t('common.copy') }}</n-button>
              <n-button v-if="hasAnySearchOrFilters" size="tiny" tertiary @click="resetAllFiltersAndRefresh">
                {{ t('common.clear') }}
              </n-button>
            </div>
          </div>
        </n-alert>

        <div ref="tableContainerEl" class="flex-1 min-h-0 overflow-hidden">
          <AppEmptyState
            v-if="showListErrorEmptyState"
            :title="listErrorMessage || t('errors.fsListFailed')"
            :description="currentDirNormalized"
          >
            <template #actions>
              <n-button size="small" secondary @click="refresh">{{ t('common.refresh') }}</n-button>
              <n-button size="small" secondary @click="up">{{ t('fsPicker.up') }}</n-button>
              <n-button size="small" secondary @click="copyCurrentPath">{{ t('common.copy') }}</n-button>
              <n-button v-if="hasAnySearchOrFilters" size="small" tertiary @click="resetAllFiltersAndRefresh">
                {{ t('common.clear') }}
              </n-button>
            </template>
          </AppEmptyState>

          <AppEmptyState
            v-else-if="showNoMatchesState"
            :title="t('fsPicker.noMatchesTitle')"
            :description="t('fsPicker.noMatchesDescription')"
          >
            <template #actions>
              <n-button size="small" secondary @click="resetAllFiltersAndRefresh">{{ t('common.clear') }}</n-button>
              <n-button size="small" secondary @click="refresh">{{ t('common.refresh') }}</n-button>
            </template>
          </AppEmptyState>

          <AppEmptyState
            v-else-if="showEmptyDirState"
            :title="t('fsPicker.emptyDirTitle')"
            :description="t('fsPicker.emptyDirDescription')"
          >
            <template #actions>
              <n-button size="small" secondary @click="refresh">{{ t('common.refresh') }}</n-button>
              <n-button size="small" secondary @click="up">{{ t('fsPicker.up') }}</n-button>
              <n-button size="small" secondary @click="copyCurrentPath">{{ t('common.copy') }}</n-button>
            </template>
          </AppEmptyState>

          <div v-else class="h-full overflow-hidden rounded-lg app-border-subtle">
            <n-data-table
              class="app-picker-table"
              :bordered="false"
              :size="isDesktop ? 'medium' : 'small'"
              :row-class-name="rowClassName"
              :loading="loading"
              :columns="columns"
              :data="tableData"
              :row-key="(row) => row.path"
              :checked-row-keys="checked"
              @update:checked-row-keys="updateCheckedRowKeys"
              :max-height="tableBodyMaxHeightPx || undefined"
            />
          </div>
        </div>
        <div v-if="nextCursor" class="pt-2 flex justify-center">
          <n-button size="small" :loading="loadingMore" :disabled="loading" @click="loadMore">
            {{ t('common.loadMore') }}
          </n-button>
        </div>
      </div>
    </div>

    <template #footer>
      <PickerFooterRow>
        <template #left>
          <n-popover
            v-if="!isSingleDirMode && isDesktop"
            v-model:show="selectionPopoverOpen"
            trigger="click"
            placement="bottom-start"
            :show-arrow="false"
          >
            <template #trigger>
              <n-button size="tiny" tertiary>
                {{ t('fsPicker.selectedCount', { count: selectedCount }) }}
              </n-button>
            </template>
            <div class="w-96 space-y-2">
              <div class="text-xs opacity-70">{{ t('common.selectionLoadedHint') }}</div>
              <div class="flex flex-wrap gap-2">
                <n-button size="tiny" secondary @click="selectAllLoadedRows">
                  {{ t('common.selectAllLoaded') }}
                </n-button>
                <n-button size="tiny" secondary @click="invertLoadedRowsSelection">
                  {{ t('common.invertLoaded') }}
                </n-button>
                <n-button size="tiny" tertiary :disabled="selectedCount === 0" @click="clearSelection">
                  {{ t('common.clearSelection') }}
                </n-button>
              </div>

              <div
                v-if="selectedUnique.length > 0"
                class="max-h-[40vh] overflow-auto rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5 space-y-1"
              >
                <div v-for="p in selectedUnique" :key="p" class="font-mono text-xs break-all">
                  {{ p }}
                </div>
              </div>
              <div v-else class="text-xs opacity-60">
                {{ t('common.noSelection') }}
              </div>
            </div>
          </n-popover>
        </template>

        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
        <n-button
          v-if="!isDesktop && !isSingleDirMode"
          size="small"
          :title="t('fsPicker.selectedCount', { count: selectedCount })"
          :aria-label="t('fsPicker.selectedCount', { count: selectedCount })"
          @click="selectionDrawerOpen = true"
        >
          <template #icon>
            <n-icon><list-outline /></n-icon>
          </template>
        </n-button>
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
      </PickerFooterRow>
    </template>
  </PickerModalCard>

  <n-drawer
    v-if="!isDesktop && !isSingleDirMode"
    v-model:show="selectionDrawerOpen"
    placement="bottom"
    height="80vh"
  >
    <n-drawer-content :title="t('fsPicker.confirm.selectedItems', { count: selectedCount })" closable>
      <div class="space-y-2">
        <div class="text-xs opacity-70">{{ t('common.selectionLoadedHint') }}</div>
        <div class="flex flex-wrap gap-2">
          <n-button size="small" secondary @click="selectAllLoadedRows">
            {{ t('common.selectAllLoaded') }}
          </n-button>
          <n-button size="small" secondary @click="invertLoadedRowsSelection">
            {{ t('common.invertLoaded') }}
          </n-button>
          <n-button size="small" tertiary :disabled="selectedCount === 0" @click="clearSelection">
            {{ t('common.clearSelection') }}
          </n-button>
        </div>

        <div
          v-if="selectedUnique.length > 0"
          class="max-h-[55vh] overflow-auto rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5 space-y-1"
        >
          <div v-for="p in selectedUnique" :key="p" class="font-mono text-xs break-all">
            {{ p }}
          </div>
        </div>
        <div v-else class="text-xs opacity-60">
          {{ t('common.noSelection') }}
        </div>
      </div>
    </n-drawer-content>
  </n-drawer>

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
