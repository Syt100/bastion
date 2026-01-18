<script setup lang="ts">
import { computed, h, ref } from 'vue'
import {
  NBadge,
  NButton,
  NDataTable,
  NForm,
  NFormItem,
  NInputNumber,
  NSelect,
  NSwitch,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

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
import { usePickerTableBodyMaxHeightPx } from '@/components/pickers/usePickerTableBodyMaxHeightPx'

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

const selectedCount = computed(() => selected.value.size)
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
  selected.value = new Map()
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

function updateCheckedRowKeys(keys: Array<string | number>): void {
  const current = selected.value
  const nextKeys = new Set(keys.map((k) => String(k)))

  // Remove unchecked keys.
  const next = new Map<string, 'file' | 'dir'>()
  for (const [p, k] of current.entries()) {
    if (nextKeys.has(p)) next.set(p, k)
  }

  // Add newly checked keys with kind from current page (fallback to file).
  for (const raw of keys) {
    const p = String(raw)
    if (next.has(p)) continue
    const row = entries.value.find((e) => e.path === p)
    const kind = row?.kind === 'dir' ? 'dir' : 'file'
    next.set(p, kind)
  }

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
      </div>

      <PickerActiveChipsRow :chips="activeChips" :clear-label="t('common.clear')" @clear="resetAllFilters" />

      <div class="flex flex-col gap-2 flex-1 min-h-0">
        <div ref="tableContainerEl" class="flex-1 min-h-0 overflow-hidden">
          <n-data-table
            :loading="loading"
            :columns="columns"
            :data="entries"
            :row-key="(row) => row.path"
            :checked-row-keys="checkedRowKeys"
            @update:checked-row-keys="updateCheckedRowKeys"
            :max-height="tableBodyMaxHeightPx || undefined"
          />
        </div>

        <div v-if="nextCursor != null" class="flex justify-center shrink-0">
          <n-button size="small" :loading="loadingMore" @click="loadMore">{{ t('common.more') }}</n-button>
        </div>
      </div>
    </div>

    <template #footer>
      <PickerFooterRow>
        <template #left>
          <div v-if="isDesktop" class="text-xs opacity-70">
            {{ t('fsPicker.selectedCount', { count: selectedCount }) }}
          </div>
        </template>

        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
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
</template>
