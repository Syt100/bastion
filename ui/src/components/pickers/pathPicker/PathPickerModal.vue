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

import { copyText } from '@/lib/clipboard'
import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatBytes } from '@/lib/format'
import { formatToastError } from '@/lib/errors'
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

import type {
  PathPickerCapabilities,
  PathPickerListRequest,
  PathPickerMode,
  PathPickerOpenOptions,
  PathPickerSizeUnit,
  PickerDataSource,
  PathPickerEntry,
  PathPickerErrorKind,
  PathPickerSortBy,
  PathPickerSortDir,
  PathPickerTypeSort,
} from '@/components/pickers/pathPicker/types'

export type PathPickerModalExpose = {
  open: (ctx: unknown, initialPath?: string | PathPickerOpenOptions) => void
}

const props = defineProps<{
  dataSource: PickerDataSource
}>()

const emit = defineEmits<{
  (e: 'picked', paths: string[]): void
}>()

const { t } = useI18n()
const message = useMessage()
const isDesktop = useMediaQuery(MQ.mdUp)

const show = ref<boolean>(false)
const pathBar = ref<PickerPathBarInputExpose | null>(null)
const loading = ref<boolean>(false)
const ctx = ref<unknown>(null)

const pickerMode = ref<PathPickerMode>('multi_paths')
const isSingleDirMode = computed(() => pickerMode.value === 'single_dir')

const currentPath = ref<string>('/')
const entries = ref<PathPickerEntry[]>([])
const checked = ref<string[]>([])
const nextCursor = ref<string | null>(null)
const total = ref<number | null>(null)

type ListErrorKind = 'none' | PathPickerErrorKind
const listErrorKind = ref<ListErrorKind>('none')
const listErrorMessage = ref<string>('')

const searchDraft = ref<string>('')
const searchApplied = ref<string>('')
const kindFilter = ref<'all' | 'dir' | 'file' | 'symlink'>('all')
const hideDotfiles = ref<boolean>(false)

const typeSort = ref<PathPickerTypeSort>('dir_first')

const sortBy = ref<PathPickerSortBy>('name')
const sortDir = ref<PathPickerSortDir>('asc')

const sizeMinDraft = ref<number | null>(null)
const sizeMaxDraft = ref<number | null>(null)
const sizeUnitDraft = ref<PathPickerSizeUnit>('MB')

const sizeMinApplied = ref<number | null>(null)
const sizeMaxApplied = ref<number | null>(null)
const sizeUnitApplied = ref<PathPickerSizeUnit>('MB')

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

const i18nPrefix = computed(() => props.dataSource.i18nPrefix)
const capabilities = computed<PathPickerCapabilities>(() => props.dataSource.capabilities(ctx.value))
const persistenceContextKey = computed(() => {
  try {
    return props.dataSource.contextKey(ctx.value)
  } catch {
    return ''
  }
})

const pageLimit = computed(() => capabilities.value.pagination?.pageSize ?? 200)
let listFetchSeq = 0

type SingleDirStatus = 'unknown' | 'ok' | 'not_found' | 'not_directory' | 'permission_denied' | 'offline' | 'error'
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

const modalTitle = computed(() =>
  isSingleDirMode.value ? t(`${i18nPrefix.value}.dirTitle`) : t(`${i18nPrefix.value}.title`),
)
const singleDirConfirmLabel = computed(() =>
  isSingleDirMode.value &&
  singleDirStatus.value === 'not_found' &&
  singleDirNotFoundConfirmPath.value === normalizePath(currentPath.value)
    ? t(`${i18nPrefix.value}.selectDirAnyway`)
    : t(`${i18nPrefix.value}.selectCurrentDir`),
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
  { label: t(`${i18nPrefix.value}.kindAll`), value: 'all' as const },
  ...((capabilities.value.kindFilter?.values ?? ['dir', 'file', 'symlink']).map((v) => ({
    label: v === 'dir' ? t('common.dir') : v === 'file' ? t('common.file') : t('common.symlink'),
    value: v,
  })) satisfies Array<{ label: string; value: 'dir' | 'file' | 'symlink' }>),
])

const sizeUnitOptions = computed(() => [
  ...((capabilities.value.sizeFilter?.units ?? ['B', 'KB', 'MB', 'GB']).map((v) => ({ label: v, value: v })) satisfies Array<{
    label: string
    value: PathPickerSizeUnit
  }>),
])

const typeSortOptions = computed(() => [
  { label: t('common.dirFirst'), value: 'dir_first' as const },
  { label: t('common.fileFirst'), value: 'file_first' as const },
])

const sortByOptions = computed(() => [
  ...((capabilities.value.sort?.by ?? ['name', 'mtime', 'size']).map((v) => ({
    label: v === 'name' ? t('common.name') : v === 'mtime' ? t('common.modified') : v === 'size' ? t('common.size') : String(v),
    value: v,
  })) satisfies Array<{ label: string; value: PathPickerSortBy }>),
])

const sortDirOptions = computed(() => [
  ...((capabilities.value.sort?.dir ?? ['asc', 'desc']).map((v) => ({
    label: v === 'desc' ? t('common.desc') : t('common.asc'),
    value: v,
  })) satisfies Array<{ label: string; value: PathPickerSortDir }>),
])

const supportsSearch = computed(() => Boolean(capabilities.value.search))
const supportsKindFilter = computed(() => Boolean(capabilities.value.kindFilter))
const supportsHideDotfiles = computed(() => Boolean(capabilities.value.hideDotfiles))
const supportsSizeFilter = computed(() => Boolean(capabilities.value.sizeFilter))
const supportsTypeSort = computed(() => Boolean(capabilities.value.typeSort))
const supportsSort = computed(() => Boolean(capabilities.value.sort))
const showFilters = computed(
  () =>
    supportsKindFilter.value ||
    supportsHideDotfiles.value ||
    supportsSizeFilter.value ||
    supportsTypeSort.value ||
    supportsSort.value,
)

const hasSizeApplied = computed(() => sizeMinApplied.value != null || sizeMaxApplied.value != null)

const activeFilterCount = computed(() => {
  const cap = capabilities.value
  let count = 0
  if (supportsKindFilter.value && kindFilter.value !== (cap.kindFilter?.default ?? 'all')) count += 1
  if (supportsHideDotfiles.value && hideDotfiles.value) count += 1
  if (supportsSizeFilter.value && hasSizeApplied.value) count += 1
  if (supportsTypeSort.value && typeSort.value !== (cap.typeSort?.default ?? 'dir_first')) count += 1
  if (supportsSort.value && (sortBy.value !== cap.sort?.defaultBy || sortDir.value !== cap.sort?.defaultDir)) count += 1
  return count
})

const hasAnySearchOrFilters = computed(
  () => (supportsSearch.value && searchApplied.value.trim().length > 0) || activeFilterCount.value > 0,
)

function formatSizeRange(min: number | null, max: number | null, unit: PathPickerSizeUnit): string {
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
  if (supportsSearch.value && q) {
    out.push({ key: 'search', label: `${t('common.search')}: ${q}`, onClose: clearSearchAndRefresh })
  }

  if (supportsKindFilter.value && kindFilter.value !== (capabilities.value.kindFilter?.default ?? 'all')) {
    out.push({
      key: 'kind',
      label: `${t('common.type')}: ${kindLabel(kindFilter.value)}`,
      onClose: clearKindFilterAndRefresh,
    })
  }

  if (supportsHideDotfiles.value && hideDotfiles.value) {
    out.push({ key: 'dotfiles', label: t('common.hideDotfiles'), onClose: clearDotfilesAndRefresh })
  }

  if (supportsSizeFilter.value && hasSizeApplied.value) {
    out.push({
      key: 'size',
      label: `${t('common.fileSize')}: ${formatSizeRange(sizeMinApplied.value, sizeMaxApplied.value, sizeUnitApplied.value)}`,
      onClose: clearSizeFilterAndRefresh,
    })
  }

  if (supportsTypeSort.value && typeSort.value !== (capabilities.value.typeSort?.default ?? 'dir_first')) {
    out.push({
      key: 'typeSort',
      label: `${t('common.typeSort')}: ${typeSort.value === 'file_first' ? t('common.fileFirst') : t('common.dirFirst')}`,
      onClose: resetTypeSortAndRefresh,
    })
  }

  if (supportsSort.value && (sortBy.value !== capabilities.value.sort?.defaultBy || sortDir.value !== capabilities.value.sort?.defaultDir)) {
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

function rowClassName(row: PathPickerEntry): string {
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

function sortByLabel(v: PathPickerSortBy): string {
  if (v === 'mtime') return t('common.modified')
  if (v === 'size') return t('common.size')
  return t('common.name')
}

function sortDirLabel(v: PathPickerSortDir): string {
  return v === 'desc' ? t('common.desc') : t('common.asc')
}

function normalizePath(p: string): string {
  return props.dataSource.normalizePath(p, ctx.value)
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
  kindFilter.value = capabilities.value.kindFilter?.default ?? 'all'
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
  typeSort.value = capabilities.value.typeSort?.default ?? 'dir_first'
}

function resetSort(): void {
  sortBy.value = capabilities.value.sort?.defaultBy ?? 'name'
  sortDir.value = capabilities.value.sort?.defaultDir ?? 'asc'
}

function resetAllFilters(): void {
  const cap = capabilities.value
  clearSearch()
  clearKindFilter()
  clearDotfiles()
  clearSizeFilter()
  sizeUnitDraft.value = cap.sizeFilter?.defaultUnit ?? 'MB'
  sizeUnitApplied.value = cap.sizeFilter?.defaultUnit ?? 'MB'
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

const modalStyle = computed(() =>
  isDesktop.value
    ? { width: MODAL_WIDTH.lg, height: MODAL_HEIGHT.desktopLoose }
    : { width: '100vw', height: '100vh', borderRadius: '0', margin: '0' },
)

function lastDirStorageKey(id: string): string {
  return `bastion.${props.dataSource.persistenceNamespace}.lastDir.${encodeURIComponent(id)}`
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

type PersistedFiltersV1 = {
  v: 1
  search?: string
  kind?: 'all' | 'dir' | 'file' | 'symlink'
  hideDotfiles?: boolean
  typeSort?: 'dir_first' | 'file_first'
  sortBy?: PathPickerSortBy
  sortDir?: PathPickerSortDir
  sizeMin?: number | null
  sizeMax?: number | null
  sizeUnit?: PathPickerSizeUnit
}

function filtersStorageKey(id: string): string {
  return `bastion.${props.dataSource.persistenceNamespace}.filters.${encodeURIComponent(id)}`
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

function normalizeSortBy(raw: unknown): PathPickerSortBy {
  if (raw === 'name' || raw === 'mtime' || raw === 'size') return raw
  return 'name'
}

function normalizeSortDir(raw: unknown): PathPickerSortDir {
  if (raw === 'asc' || raw === 'desc') return raw
  return 'asc'
}

function normalizeSizeUnit(raw: unknown): PathPickerSizeUnit {
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
  const cap = capabilities.value

  const search = supportsSearch.value ? (typeof state.search === 'string' ? state.search : '').trim() : ''
  searchDraft.value = search
  searchApplied.value = search

  const defaultKind = cap.kindFilter?.default ?? 'all'
  if (!supportsKindFilter.value) {
    kindFilter.value = defaultKind
  } else {
    const next = normalizeKind(state.kind)
    const allowed = cap.kindFilter?.values ?? ['dir', 'file', 'symlink']
    kindFilter.value = next === 'all' || allowed.includes(next) ? next : defaultKind
  }

  hideDotfiles.value = supportsHideDotfiles.value ? Boolean(state.hideDotfiles) : false

  typeSort.value = supportsTypeSort.value ? normalizeTypeSort(state.typeSort) : cap.typeSort?.default ?? 'dir_first'

  if (!supportsSort.value) {
    sortBy.value = cap.sort?.defaultBy ?? 'name'
    sortDir.value = cap.sort?.defaultDir ?? 'asc'
  } else {
    const nextBy = normalizeSortBy(state.sortBy)
    const nextDir = normalizeSortDir(state.sortDir)
    sortBy.value = cap.sort?.by?.includes(nextBy) ? nextBy : cap.sort?.defaultBy ?? 'name'
    sortDir.value = cap.sort?.dir?.includes(nextDir) ? nextDir : cap.sort?.defaultDir ?? 'asc'
  }

  const defaultUnit = cap.sizeFilter?.defaultUnit ?? 'MB'
  if (!supportsSizeFilter.value) {
    sizeMinDraft.value = null
    sizeMaxDraft.value = null
    sizeMinApplied.value = null
    sizeMaxApplied.value = null
    sizeUnitDraft.value = defaultUnit
    sizeUnitApplied.value = defaultUnit
    return
  }

  const sizeMin = normalizeNumberOrNull(state.sizeMin)
  const sizeMax = normalizeNumberOrNull(state.sizeMax)
  let sizeUnit = normalizeSizeUnit(state.sizeUnit)
  if (cap.sizeFilter?.units && !cap.sizeFilter.units.includes(sizeUnit)) sizeUnit = defaultUnit
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
  const id = persistenceContextKey.value.trim()
  if (!id) return
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
    () => persistenceContextKey.value,
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
  if (!ctx.value) return

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
    const res = await props.dataSource.list(ctx.value, buildListRequest(p, null), t)
    if (seq !== listFetchSeq) return
    currentPath.value = res.path
    entries.value = res.entries
    nextCursor.value = normalizeCursor(res.nextCursor)
    total.value = typeof res.total === 'number' ? res.total : null
    const id = persistenceContextKey.value.trim()
    if (id) saveLastDir(id, currentPath.value)
    if (isSingleDirMode.value) {
      singleDirStatus.value = 'ok'
      singleDirMessage.value = ''
      singleDirValidatedPath.value = normalizePath(res.path)
      singleDirNotFoundConfirmPath.value = ''
    }
  } catch (error) {
    if (seq !== listFetchSeq) return
    const info = props.dataSource.mapError(error, ctx.value, t)
    if (isSingleDirMode.value) {
      const errorText = info.message || t('errors.fsListFailed')

      if (info.kind === 'not_found') {
        singleDirStatus.value = 'not_found'
        singleDirMessage.value = ''
        entries.value = []
        return
      }
      if (info.kind === 'permission_denied') {
        singleDirStatus.value = 'permission_denied'
        singleDirMessage.value = errorText
        entries.value = []
        return
      }
      if (info.kind === 'offline') {
        singleDirStatus.value = 'offline'
        singleDirMessage.value = errorText
        entries.value = []
        return
      }
      if (info.kind === 'not_directory') {
        singleDirStatus.value = 'not_directory'
        singleDirMessage.value = errorText
        entries.value = []
        return
      }

      singleDirStatus.value = 'error'
      singleDirMessage.value = errorText
      entries.value = []
      return
    }

    listErrorKind.value = info.kind
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
  if (!ctx.value) return

  const seq = ++listFetchSeq
  loadingMore.value = true
  try {
    const res = await props.dataSource.list(ctx.value, buildListRequest(p, cursor), t)
    if (seq !== listFetchSeq) return

    currentPath.value = res.path
    entries.value = [...entries.value, ...res.entries]
    nextCursor.value = normalizeCursor(res.nextCursor)
    total.value = typeof res.total === 'number' ? res.total : null
    const id = persistenceContextKey.value.trim()
    if (id) saveLastDir(id, currentPath.value)
  } catch (error) {
    if (seq !== listFetchSeq) return
    const info = props.dataSource.mapError(error, ctx.value, t)
    if (info.kind === 'invalid_cursor' || info.kind === 'offline' || info.kind === 'permission_denied') {
      listErrorKind.value = info.kind
      listErrorMessage.value = info.message
      nextCursor.value = null
      return
    }
    message.error(formatToastError(t('errors.fsListFailed'), error, t))
  } finally {
    if (seq === listFetchSeq) loadingMore.value = false
  }
}

function normalizeCursor(v?: string | null): string | null {
  const t = (v ?? '').trim()
  return t ? t : null
}

function buildListRequest(path: string, cursor: string | null): PathPickerListRequest {
  const cap = capabilities.value
  const req: PathPickerListRequest = {
    path,
    cursor,
    limit: pageLimit.value,
    mode: pickerMode.value,
  }

  if (cap.sort) {
    req.sortBy = sortBy.value
    req.sortDir = sortDir.value
  }

  const effectiveKind =
    isSingleDirMode.value
      ? 'dir'
      : cap.kindFilter
        ? kindFilter.value === 'all'
          ? null
          : kindFilter.value
        : null
  if (effectiveKind) req.kind = effectiveKind

  if (!isSingleDirMode.value) {
    if (cap.search) {
      const q = searchApplied.value.trim()
      if (q) req.q = q
    }
    if (cap.hideDotfiles && hideDotfiles.value) req.hideDotfiles = true
    if (cap.typeSort) req.typeSort = typeSort.value
    if (cap.sizeFilter && (sizeMinApplied.value != null || sizeMaxApplied.value != null)) {
      req.size = { min: sizeMinApplied.value, max: sizeMaxApplied.value, unit: sizeUnitApplied.value }
    }
  }

  return req
}

function open(nextCtx: unknown, initialPath?: string | PathPickerOpenOptions): void {
  const opts =
    typeof initialPath === 'string' ? ({ path: initialPath } satisfies PathPickerOpenOptions) : initialPath

  ctx.value = nextCtx
  pickerMode.value = opts?.mode ?? 'multi_paths'

  const explicitPath = opts?.path?.trim()
  const id = persistenceContextKey.value.trim()
  if (explicitPath) {
    currentPath.value = explicitPath
  } else {
    const remembered = id ? loadLastDir(id) : null
    currentPath.value = remembered ?? props.dataSource.defaultPath(nextCtx)
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
    const persisted = id ? loadPersistedFilters(id) : null
    if (persisted) applyPersistedFilters(persisted)
    else resetAllFilters()
  }
  listErrorKind.value = 'none'
  listErrorMessage.value = ''
  singleDirStatus.value = 'unknown'
  singleDirMessage.value = ''
  filtersPopoverOpen.value = false
  filtersDrawerOpen.value = false
  shortcutsPopoverOpen.value = false
  shortcutsDrawerOpen.value = false
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
  currentPath.value = props.dataSource.parentPath(currentPath.value, ctx.value)
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

const columns = computed<DataTableColumns<PathPickerEntry>>(() => {
  const cap = capabilities.value
  const showKindCol = cap.columns?.kind ?? true
  const showSizeCol = cap.columns?.size ?? true
  const showMtimeCol = cap.columns?.mtime ?? true

  const nameColumn = {
    title: t('common.name'),
    key: 'name',
    render(row: PathPickerEntry) {
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
      if (showSizeCol && row.kind === 'file' && typeof row.size === 'number' && Number.isFinite(row.size)) {
        parts.push(formatBytes(row.size))
      }
      if (showMtimeCol) parts.push(formatMtimeMobile(row.mtime))
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
        ? (showMtimeCol
            ? ([
                {
                  title: t('common.modified'),
                  key: 'mtime',
                  width: 190,
                  render: (row: PathPickerEntry) => formatMtimeDesktop(row.mtime),
                },
              ] as const)
            : [])
        : []),
    ]
  }

  return [
    { type: 'selection' },
    nameColumn,
    ...(isDesktop.value
      ? ([
          ...(showKindCol
            ? ([
                {
                  title: t('common.type'),
                  key: 'kind',
                  width: 110,
                  render: (row: PathPickerEntry) => kindLabel(row.kind),
                },
              ] as const)
            : []),
          ...(showSizeCol
            ? ([
                {
                  title: t('common.size'),
                  key: 'size',
                  width: 120,
                  align: 'right',
                  render: (row: PathPickerEntry) =>
                    row.kind === 'file' && typeof row.size === 'number' && Number.isFinite(row.size) ? formatBytes(row.size) : '-',
                },
              ] as const)
            : []),
          ...(showMtimeCol
            ? ([
                {
                  title: t('common.modified'),
                  key: 'mtime',
                  width: 190,
                  render: (row: PathPickerEntry) => formatMtimeDesktop(row.mtime),
                },
              ] as const)
            : []),
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

defineExpose<PathPickerModalExpose>({ open })
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
          :placeholder="t(`${i18nPrefix}.currentPath`)"
          :up-title="t(`${i18nPrefix}.up`)"
          :refresh-title="t('common.refresh')"
          @up="up"
          @refresh="refresh"
          @enter="refresh"
          @navigate="onPathBarNavigate"
        />
        <n-alert v-if="isSingleDirMode && singleDirStatus === 'not_found'" type="warning" :bordered="false">
          {{ t(`${i18nPrefix}.dirNotFoundWillCreate`) }}
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
          v-if="supportsSearch"
          v-model:value="searchDraft"
          :placeholder="t(`${i18nPrefix}.searchPlaceholder`)"
          :search-disabled="!hasSearchDraftChanges"
          @search="applySearch"
        />

        <PickerFiltersPopoverDrawer
          v-if="showFilters"
          :is-desktop="isDesktop"
          :title="t('common.filters')"
          :active-count="activeFilterCount"
          v-model:popover-open="filtersPopoverOpen"
          v-model:drawer-open="filtersDrawerOpen"
        >
          <n-form label-placement="top" size="small">
            <n-form-item v-if="supportsKindFilter" :label="t('common.type')">
              <n-select v-model:value="kindFilter" :options="kindOptions" @update:value="refreshForFilters" />
            </n-form-item>
            <n-form-item v-if="supportsHideDotfiles" :label="t('common.hideDotfiles')">
              <n-switch v-model:value="hideDotfiles" @update:value="refreshForFilters" />
            </n-form-item>
            <n-form-item v-if="supportsSizeFilter" :label="t('common.fileSize')">
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
            <n-form-item v-if="supportsTypeSort" :label="t('common.typeSort')">
              <n-select v-model:value="typeSort" :options="typeSortOptions" @update:value="refreshForFilters" />
            </n-form-item>
            <n-form-item v-if="supportsSort" :label="t('common.sort')">
              <div class="grid grid-cols-2 gap-2">
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
              <n-button size="tiny" secondary @click="up">{{ t(`${i18nPrefix}.up`) }}</n-button>
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
              <n-button size="small" secondary @click="up">{{ t(`${i18nPrefix}.up`) }}</n-button>
              <n-button size="small" secondary @click="copyCurrentPath">{{ t('common.copy') }}</n-button>
              <n-button v-if="hasAnySearchOrFilters" size="small" tertiary @click="resetAllFiltersAndRefresh">
                {{ t('common.clear') }}
              </n-button>
            </template>
          </AppEmptyState>

          <AppEmptyState
            v-else-if="showNoMatchesState"
            :title="t(`${i18nPrefix}.noMatchesTitle`)"
            :description="t(`${i18nPrefix}.noMatchesDescription`)"
          >
            <template #actions>
              <n-button size="small" secondary @click="resetAllFiltersAndRefresh">{{ t('common.clear') }}</n-button>
              <n-button size="small" secondary @click="refresh">{{ t('common.refresh') }}</n-button>
            </template>
          </AppEmptyState>

          <AppEmptyState
            v-else-if="showEmptyDirState"
            :title="t(`${i18nPrefix}.emptyDirTitle`)"
            :description="t(`${i18nPrefix}.emptyDirDescription`)"
          >
            <template #actions>
              <n-button size="small" secondary @click="refresh">{{ t('common.refresh') }}</n-button>
              <n-button size="small" secondary @click="up">{{ t(`${i18nPrefix}.up`) }}</n-button>
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
                {{ t(`${i18nPrefix}.selectedCount`, { count: selectedCount }) }}
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
          :title="t(`${i18nPrefix}.selectedCount`, { count: selectedCount })"
          :aria-label="t(`${i18nPrefix}.selectedCount`, { count: selectedCount })"
          @click="selectionDrawerOpen = true"
        >
          <template #icon>
            <n-icon><list-outline /></n-icon>
          </template>
        </n-button>
        <n-button v-if="!isSingleDirMode" :disabled="!currentPath.trim()" @click="requestPickCurrentDir">
          {{ t(`${i18nPrefix}.selectCurrentDir`) }}
        </n-button>
        <n-button
          v-if="isSingleDirMode"
          type="primary"
          :loading="loading"
          :disabled="!currentPath.trim() || ['permission_denied', 'not_directory', 'offline', 'error'].includes(singleDirStatus)"
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
            {{ t(`${i18nPrefix}.addSelected`) }}
          </n-button>
        </n-badge>
        <n-button v-else type="primary" :disabled="checked.length === 0" @click="pick">
          {{ t(`${i18nPrefix}.addSelected`) }}
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
    <n-drawer-content :title="t(`${i18nPrefix}.confirm.selectedItems`, { count: selectedCount })" closable>
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
    :title="t(`${i18nPrefix}.confirm.title`)"
  >
    <div class="space-y-3">
      <div class="space-y-1">
        <div class="text-xs opacity-70">{{ t(`${i18nPrefix}.confirm.currentDir`) }}</div>
        <div class="font-mono text-xs break-all rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5">
          {{ currentDirNormalized }}
        </div>
      </div>

      <div class="space-y-1">
        <div class="text-xs opacity-70">{{ t(`${i18nPrefix}.confirm.selectedItems`, { count: selectedUnique.length }) }}</div>
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
          {{ t(`${i18nPrefix}.confirm.withSelected`) }}
        </n-button>
        <n-button type="primary" @click="confirmPickCurrentDirOnly">
          {{ t(`${i18nPrefix}.confirm.onlyCurrent`) }}
        </n-button>
      </div>
    </template>
  </n-modal>

  <n-drawer v-else v-model:show="pickCurrentDirConfirmOpen" placement="bottom" height="80vh">
    <n-drawer-content :title="t(`${i18nPrefix}.confirm.title`)" closable>
      <div class="space-y-3">
        <div class="space-y-1">
          <div class="text-xs opacity-70">{{ t(`${i18nPrefix}.confirm.currentDir`) }}</div>
          <div class="font-mono text-xs break-all rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5">
            {{ currentDirNormalized }}
          </div>
        </div>

        <div class="space-y-1">
          <div class="text-xs opacity-70">{{ t(`${i18nPrefix}.confirm.selectedItems`, { count: selectedUnique.length }) }}</div>
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
          {{ t(`${i18nPrefix}.confirm.withSelected`) }}
        </n-button>
        <n-button type="primary" @click="confirmPickCurrentDirOnly">
          {{ t(`${i18nPrefix}.confirm.onlyCurrent`) }}
        </n-button>
      </div>
    </n-drawer-content>
  </n-drawer>
</template>
