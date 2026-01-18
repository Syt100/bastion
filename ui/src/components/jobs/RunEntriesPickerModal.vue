<script setup lang="ts">
import { computed, h, ref } from 'vue'
import {
  NBadge,
  NButton,
  NDataTable,
  NDrawer,
  NDrawerContent,
  NForm,
  NFormItem,
  NIcon,
  NInputNumber,
  NPopover,
  NSelect,
  NSwitch,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { ListOutline } from '@vicons/ionicons5'

import { apiFetch } from '@/lib/api'
import { MODAL_HEIGHT, MODAL_WIDTH } from '@/lib/modal'
import { formatBytes } from '@/lib/format'
import { formatToastError } from '@/lib/errors'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
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

export type RunEntriesSelection = {
  files: string[]
  dirs: string[]
}

type RunEntry = {
  path: string
  kind: string
  size: number
}

type RunEntriesResponse = {
  prefix: string
  cursor: number
  next_cursor?: number | null
  entries: RunEntry[]
}

export type RunEntriesPickerModalExpose = {
  open: (runId: string) => void
}

const emit = defineEmits<{
  (e: 'picked', selection: RunEntriesSelection): void
}>()

const { t } = useI18n()
const message = useMessage()

const isDesktop = useMediaQuery(MQ.mdUp)

const show = ref<boolean>(false)
const prefixBar = ref<PickerPathBarInputExpose | null>(null)
const loading = ref<boolean>(false)
const loadingMore = ref<boolean>(false)

const runId = ref<string | null>(null)
const prefix = ref<string>('')
const entries = ref<RunEntry[]>([])
const nextCursor = ref<number | null>(null)

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
const shortcutsPopoverOpen = ref<boolean>(false)
const shortcutsDrawerOpen = ref<boolean>(false)
const selectionPopoverOpen = ref<boolean>(false)
const selectionDrawerOpen = ref<boolean>(false)

const lastRangeAnchorPath = ref<string | null>(null)
const { shiftPressed } = useShiftKeyPressed(show)

const { tableContainerEl, tableBodyMaxHeightPx } = usePickerTableBodyMaxHeightPx(show, {
  onOpen: () => {
    // Keep the initial focus on the path input so the first action button doesn't look "selected".
    try {
      prefixBar.value?.focus?.()
    } catch {
      // ignore
    }
  },
})

function onPrefixNavigate(): void {
  refresh()
}

const selected = ref<Map<string, 'file' | 'dir'>>(new Map())
const checkedRowKeys = computed<string[]>(() => Array.from(selected.value.keys()))

const pickerShortcuts = computed<PickerShortcutItem[]>(() => [
  { combo: 'Enter', description: t('pickers.shortcuts.enterDir') },
  { combo: 'Backspace', description: t('pickers.shortcuts.up') },
  { combo: 'Ctrl/Cmd+L', description: t('pickers.shortcuts.focusPath') },
  { combo: 'Esc', description: t('pickers.shortcuts.close') },
  { combo: 'Shift', description: t('pickers.shortcuts.rangeSelect') },
])

const pickerShortcutsNote = computed(() => t('pickers.shortcuts.note'))

function rowClassName(row: RunEntry): string {
  if (selected.value.has(row.path)) return 'app-picker-row app-picker-row--checked'
  return 'app-picker-row'
}

const selectedCount = computed(() => selected.value.size)
const selectedPreviewItems = computed(() => Array.from(selected.value.entries()).map(([path, kind]) => ({ path, kind })))
const hasSearchDraftChanges = computed(() => searchDraft.value.trim() !== searchApplied.value)

const hasSizeApplied = computed(() => sizeMinApplied.value != null || sizeMaxApplied.value != null)

const activeFilterCount = computed(() => {
  let count = 0
  if (kindFilter.value !== 'all') count += 1
  if (hideDotfiles.value) count += 1
  if (hasSizeApplied.value) count += 1
  if (typeSort.value !== 'dir_first') count += 1
  return count
})

const kindOptions = computed(() => [
  { label: t('restore.pick.kindAll'), value: 'all' as const },
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

const modalStyle = computed(() =>
  isDesktop.value
    ? { width: MODAL_WIDTH.lg, height: MODAL_HEIGHT.desktopLoose }
    : { width: '100vw', height: '100vh', borderRadius: '0', margin: '0' },
)

function entryName(p: string): string {
  const parts = p.split('/')
  return parts[parts.length - 1] || p
}

function parentPrefix(p: string): string {
  const s = p.trim().replace(/\/+$/, '')
  if (!s) return ''
  const idx = s.lastIndexOf('/')
  if (idx <= 0) return ''
  return s.slice(0, idx)
}

function kindLabel(kind: string): string {
  if (kind === 'dir') return t('common.dir')
  if (kind === 'symlink') return t('common.symlink')
  if (kind === 'file') return t('common.file')
  return kind
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
  onFiltersChanged()
}

async function fetchPage(cursor: number, append: boolean): Promise<void> {
  const id = runId.value
  if (!id) return

  const params = new URLSearchParams()
  if (prefix.value.trim()) params.set('prefix', prefix.value.trim())
  params.set('cursor', String(cursor))
  params.set('limit', '200')
  if (searchApplied.value.trim()) params.set('q', searchApplied.value.trim())
  if (kindFilter.value !== 'all') params.set('kind', kindFilter.value)
  if (hideDotfiles.value) params.set('hide_dotfiles', 'true')
  if (typeSort.value !== 'dir_first') params.set('type_sort', typeSort.value)

  const mult = sizeUnitMultiplier(sizeUnitApplied.value)
  const minBytes =
    sizeMinApplied.value != null && Number.isFinite(sizeMinApplied.value)
      ? Math.max(0, Math.floor(sizeMinApplied.value * mult))
      : null
  const maxBytes =
    sizeMaxApplied.value != null && Number.isFinite(sizeMaxApplied.value)
      ? Math.max(0, Math.floor(sizeMaxApplied.value * mult))
      : null

  if (minBytes != null) params.set('min_size_bytes', String(minBytes))
  if (maxBytes != null) params.set('max_size_bytes', String(maxBytes))

  const url = `/api/runs/${encodeURIComponent(id)}/entries?${params.toString()}`

  const res = await apiFetch<RunEntriesResponse>(url)
  nextCursor.value = res.next_cursor ?? null
  entries.value = append ? [...entries.value, ...res.entries] : res.entries
}

async function refresh(): Promise<void> {
  if (!runId.value) return
  loading.value = true
  try {
    await fetchPage(0, false)
  } catch (error) {
    message.error(formatToastError(t('errors.runEntriesFailed'), error, t))
  } finally {
    loading.value = false
  }
}

async function loadMore(): Promise<void> {
  const cur = nextCursor.value
  if (cur == null) return
  loadingMore.value = true
  try {
    await fetchPage(cur, true)
  } catch (error) {
    message.error(formatToastError(t('errors.runEntriesFailed'), error, t))
  } finally {
    loadingMore.value = false
  }
}

function open(nextRunId: string): void {
  runId.value = nextRunId
  prefix.value = ''
  entries.value = []
  nextCursor.value = null
  searchDraft.value = ''
  searchApplied.value = ''
  kindFilter.value = 'all'
  hideDotfiles.value = false
  typeSort.value = 'dir_first'
  sizeMinDraft.value = null
  sizeMaxDraft.value = null
  sizeUnitDraft.value = 'MB'
  sizeMinApplied.value = null
  sizeMaxApplied.value = null
  sizeUnitApplied.value = 'MB'
  filtersPopoverOpen.value = false
  filtersDrawerOpen.value = false
  shortcutsPopoverOpen.value = false
  shortcutsDrawerOpen.value = false
  selectionPopoverOpen.value = false
  selectionDrawerOpen.value = false
  selected.value = new Map()
  lastRangeAnchorPath.value = null
  show.value = true
  void refresh()
}

function enterDir(p: string): void {
  prefix.value = p
  entries.value = []
  nextCursor.value = null
  void refresh()
}

function up(): void {
  prefix.value = parentPrefix(prefix.value)
  entries.value = []
  nextCursor.value = null
  void refresh()
}

function applySearch(): void {
  searchApplied.value = searchDraft.value.trim()
  entries.value = []
  nextCursor.value = null
  void refresh()
}

function onFiltersChanged(): void {
  entries.value = []
  nextCursor.value = null
  void refresh()
}

function clearSearch(): void {
  searchDraft.value = ''
  searchApplied.value = ''
  onFiltersChanged()
}

function clearKindFilter(): void {
  kindFilter.value = 'all'
  onFiltersChanged()
}

function clearDotfiles(): void {
  hideDotfiles.value = false
  onFiltersChanged()
}

function clearSizeFilter(): void {
  sizeMinDraft.value = null
  sizeMaxDraft.value = null
  sizeMinApplied.value = null
  sizeMaxApplied.value = null
  onFiltersChanged()
}

function resetTypeSort(): void {
  typeSort.value = 'dir_first'
  onFiltersChanged()
}

function resetAllFilters(): void {
  searchDraft.value = ''
  searchApplied.value = ''
  kindFilter.value = 'all'
  hideDotfiles.value = false
  sizeMinDraft.value = null
  sizeMaxDraft.value = null
  sizeUnitDraft.value = 'MB'
  sizeMinApplied.value = null
  sizeMaxApplied.value = null
  sizeUnitApplied.value = 'MB'
  typeSort.value = 'dir_first'
  onFiltersChanged()
}

function clearSelection(): void {
  selected.value = new Map()
  lastRangeAnchorPath.value = null
}

function selectAllLoadedRows(): void {
  const next = new Map(selected.value)
  for (const row of entries.value) {
    next.set(row.path, row.kind === 'dir' ? 'dir' : 'file')
  }
  selected.value = next
}

function invertLoadedRowsSelection(): void {
  const next = new Map(selected.value)
  for (const row of entries.value) {
    if (next.has(row.path)) next.delete(row.path)
    else next.set(row.path, row.kind === 'dir' ? 'dir' : 'file')
  }
  selected.value = next
}

function updateCheckedRowKeys(keys: Array<string | number>): void {
  const loaded = entries.value.map((e) => e.path)
  const loadedSet = new Set(loaded)
  const desiredLoaded = new Set(keys.map((k) => String(k)).filter((p) => loadedSet.has(p)))

  const prev = selected.value
  const next = new Map(prev)

  // Apply the desired selection state for loaded rows only; keep selection from other pages intact.
  for (const row of entries.value) {
    if (desiredLoaded.has(row.path)) next.set(row.path, row.kind === 'dir' ? 'dir' : 'file')
    else next.delete(row.path)
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
      selected.value = next
      return
    }
    const idxA = loaded.indexOf(a)
    const idxB = loaded.indexOf(b)
    if (idxA !== -1 && idxB !== -1) {
      const from = Math.min(idxA, idxB)
      const to = Math.max(idxA, idxB)
      for (const row of entries.value.slice(from, to + 1)) {
        next.set(row.path, row.kind === 'dir' ? 'dir' : 'file')
      }
    }
  }

  if (added.length === 1 && removed.length === 0) lastRangeAnchorPath.value = added[0] ?? null
  else if (removed.length === 1 && added.length === 0) lastRangeAnchorPath.value = removed[0] ?? null
  else if (next.size === 0) lastRangeAnchorPath.value = null

  selected.value = next
}

function pick(): void {
  const files: string[] = []
  const dirs: string[] = []
  for (const [p, k] of selected.value.entries()) {
    if (k === 'dir') dirs.push(p)
    else files.push(p)
  }
  if (files.length + dirs.length === 0) {
    message.error(t('errors.restoreSelectionRequired'))
    return
  }
  show.value = false
  emit('picked', { files, dirs })
}

const columns = computed<DataTableColumns<RunEntry>>(() => [
  {
    type: 'selection',
  },
  {
    title: t('common.name'),
    key: 'name',
    render(row) {
      const name = entryName(row.path)
      const label = row.kind === 'dir' ? `📁 ${name}` : row.kind === 'symlink' ? `🔗 ${name}` : `📄 ${name}`

      const nameNode =
        row.kind === 'dir'
          ? h(
              'button',
              {
                class: 'text-left w-full text-[var(--n-primary-color)] hover:underline truncate',
                onClick: () => enterDir(row.path),
              },
              label,
            )
          : h('div', { class: 'truncate' }, label)

      if (isDesktop.value) return nameNode

      const parts: string[] = []
      parts.push(kindLabel(row.kind))
      if (row.kind === 'file' || row.kind === 'symlink') parts.push(formatBytes(row.size))
      const meta = parts.join(' · ')

      return h('div', { class: 'space-y-1 min-w-0' }, [
        h('div', { class: 'min-w-0' }, [nameNode]),
        h('div', { class: 'text-xs opacity-70 truncate' }, meta),
      ])
    },
  },
  ...(isDesktop.value
    ? ([
        {
          title: t('common.type'),
          key: 'kind',
          width: 110,
          render: (row: RunEntry) => kindLabel(row.kind),
        },
        {
          title: t('common.size'),
          key: 'size',
          width: 120,
          align: 'right',
          render: (row: RunEntry) => (row.kind === 'dir' ? '-' : formatBytes(row.size)),
        },
      ] as const)
    : []),
])

function isShortcutBlocked(): boolean {
  return (
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
  if (!isTableFocused()) return false
  if (selected.value.size !== 1) return false

  const first = selected.value.entries().next().value as [string, 'file' | 'dir'] | undefined
  const p = first?.[0]
  const kind = first?.[1]
  if (!p || kind !== 'dir') return false

  enterDir(p)
  return true
}

usePickerKeyboardShortcuts(show, {
  onEscape: () => {
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
      prefixBar.value?.focus?.()
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

defineExpose<RunEntriesPickerModalExpose>({ open })
</script>

<template>
  <PickerModalCard
    v-model:show="show"
    :style="modalStyle"
    :title="t('restore.pick.title')"
  >
    <div class="flex flex-col gap-3 flex-1 min-h-0">
      <PickerPathBarInput
        ref="prefixBar"
        v-model:value="prefix"
        :placeholder="t('restore.pick.currentPrefix')"
        :up-title="t('fsPicker.up')"
        :refresh-title="t('common.refresh')"
        :disabled-up="!prefix.trim()"
        @up="up"
        @refresh="refresh"
        @enter="refresh"
        @navigate="onPrefixNavigate"
      />

      <div class="flex items-center gap-2">
        <PickerSearchInput
          v-model:value="searchDraft"
          :placeholder="t('restore.pick.searchPlaceholder')"
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
              <n-select v-model:value="kindFilter" :options="kindOptions" @update:value="onFiltersChanged" />
            </n-form-item>
            <n-form-item :label="t('common.hideDotfiles')">
              <n-switch v-model:value="hideDotfiles" @update:value="onFiltersChanged" />
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
              <n-select v-model:value="typeSort" :options="typeSortOptions" @update:value="onFiltersChanged" />
            </n-form-item>
          </n-form>

          <template #popoverFooter>
            <div class="flex justify-end">
              <n-button size="tiny" tertiary @click="resetAllFilters">{{ t('common.clear') }}</n-button>
            </div>
          </template>

          <template #drawerFooter>
            <div class="flex justify-end gap-2 pt-2">
              <n-button tertiary @click="resetAllFilters">{{ t('common.clear') }}</n-button>
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

      <PickerActiveChipsRow :chips="activeChips" :clear-label="t('common.clear')" @clear="resetAllFilters" />

      <div class="flex flex-col gap-2 flex-1 min-h-0">
        <div ref="tableContainerEl" class="flex-1 min-h-0 overflow-hidden">
          <div class="h-full overflow-hidden rounded-lg app-border-subtle">
            <n-data-table
              class="app-picker-table"
              :bordered="false"
              :size="isDesktop ? 'medium' : 'small'"
              :row-class-name="rowClassName"
              :loading="loading"
              :columns="columns"
              :data="entries"
              :row-key="(row) => row.path"
              :checked-row-keys="checkedRowKeys"
              @update:checked-row-keys="updateCheckedRowKeys"
              :max-height="tableBodyMaxHeightPx || undefined"
            />
          </div>
        </div>

        <div v-if="nextCursor != null" class="flex justify-center shrink-0">
          <n-button size="small" :loading="loadingMore" @click="loadMore">{{ t('common.more') }}</n-button>
        </div>
      </div>
    </div>

    <template #footer>
      <PickerFooterRow>
        <template #left>
          <n-popover
            v-if="isDesktop"
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
                v-if="selectedPreviewItems.length > 0"
                class="max-h-[40vh] overflow-auto rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5 space-y-1"
              >
                <div v-for="it in selectedPreviewItems" :key="it.path" class="font-mono text-xs break-all">
                  {{ it.kind === 'dir' ? `📁 ${it.path}` : `📄 ${it.path}` }}
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
          v-if="!isDesktop"
          size="small"
          :title="t('fsPicker.selectedCount', { count: selectedCount })"
          :aria-label="t('fsPicker.selectedCount', { count: selectedCount })"
          @click="selectionDrawerOpen = true"
        >
          <template #icon>
            <n-icon><list-outline /></n-icon>
          </template>
        </n-button>
        <n-badge v-if="!isDesktop" :value="selectedCount" :show="selectedCount > 0">
          <n-button type="primary" :disabled="selectedCount === 0" @click="pick">
            {{ t('restore.pick.confirm') }}
          </n-button>
        </n-badge>
        <n-button v-else type="primary" :disabled="selectedCount === 0" @click="pick">
          {{ t('restore.pick.confirm') }}
        </n-button>
      </PickerFooterRow>
    </template>
  </PickerModalCard>

  <n-drawer v-if="!isDesktop" v-model:show="selectionDrawerOpen" placement="bottom" height="80vh">
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
          v-if="selectedPreviewItems.length > 0"
          class="max-h-[55vh] overflow-auto rounded-md bg-black/2 dark:bg-white/5 px-2 py-1.5 space-y-1"
        >
          <div v-for="it in selectedPreviewItems" :key="it.path" class="font-mono text-xs break-all">
            {{ it.kind === 'dir' ? `📁 ${it.path}` : `📄 ${it.path}` }}
          </div>
        </div>
        <div v-else class="text-xs opacity-60">
          {{ t('common.noSelection') }}
        </div>
      </div>
    </n-drawer-content>
  </n-drawer>
</template>
