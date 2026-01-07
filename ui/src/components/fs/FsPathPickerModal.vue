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

import { ApiError, apiFetch } from '@/lib/api'
import { MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatBytes } from '@/lib/format'
import { formatToastError } from '@/lib/errors'
import { formatUnixSecondsYmdHms } from '@/lib/datetime'

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
  open: (nodeId: 'hub' | string, initialPath?: string) => void
}

const emit = defineEmits<{
  (e: 'picked', paths: string[]): void
}>()

const { t } = useI18n()
const message = useMessage()
const isDesktop = useMediaQuery(MQ.mdUp)

const show = ref<boolean>(false)
const loading = ref<boolean>(false)
const nodeId = ref<'hub' | string>('hub')
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

const selectedCount = computed(() => checked.value.length)

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

const tableMaxHeight = computed(() => (isDesktop.value ? 420 : 'calc(100vh - 390px)'))
const modalStyle = computed(() =>
  isDesktop.value
    ? { width: MODAL_WIDTH.lg }
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
  try {
    const url = (path: string) =>
      `/api/nodes/${encodeURIComponent(nodeId.value)}/fs/list?path=${encodeURIComponent(path)}`

    const res = await apiFetch<FsListResponse>(url(p))
    currentPath.value = res.path
    entries.value = res.entries
    saveLastDir(nodeId.value, currentPath.value)
  } catch (error) {
    const code = error instanceof ApiError ? error.body?.error : undefined
    const msg = (error instanceof ApiError ? error.body?.message || error.message : '') || ''
    const shouldFallback =
      code === 'not_directory' || (code === 'agent_fs_list_failed' && msg.toLowerCase().includes('not a directory'))

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

function open(nextNodeId: 'hub' | string, initialPath?: string): void {
  nodeId.value = nextNodeId
  const remembered = loadLastDir(nextNodeId)
  currentPath.value = remembered ?? (initialPath?.trim() || '/')
  entries.value = []
  checked.value = []
  resetAllFilters()
  filtersPopoverOpen.value = false
  filtersDrawerOpen.value = false
  show.value = true
  void refresh()
}

function addCurrentDirToSelection(): void {
  const p = normalizePath(currentPath.value)
  if (!p) return
  if (!checked.value.includes(p)) checked.value = [...checked.value, p]
}

function up(): void {
  currentPath.value = computeParentPath(currentPath.value)
  void refresh()
}

function pick(): void {
  const unique = Array.from(new Set(checked.value.map((v) => v.trim()).filter((v) => v.length > 0)))
  if (unique.length === 0) {
    message.error(t('errors.sourcePathsRequired'))
    return
  }
  show.value = false
  emit('picked', unique)
}

const columns = computed<DataTableColumns<FsListEntry>>(() => [
  {
    type: 'selection',
  },
  {
    title: t('common.name'),
    key: 'name',
    render(row) {
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
  },
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
])

defineExpose<FsPathPickerModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="modalStyle" :title="t('fsPicker.title')">
    <div class="space-y-3">
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex items-center gap-2">
          <n-button size="small" @click="up">{{ t('fsPicker.up') }}</n-button>
          <n-button size="small" @click="refresh">{{ t('common.refresh') }}</n-button>
          <n-button size="small" @click="addCurrentDirToSelection">{{ t('fsPicker.selectCurrentDir') }}</n-button>
        </div>
        <div class="text-xs opacity-70">{{ t('fsPicker.selectedCount', { count: selectedCount }) }}</div>
      </div>

      <div class="space-y-2">
        <div class="text-xs opacity-70">{{ t('fsPicker.currentPath') }}</div>
        <n-input v-model:value="currentPath" @keyup.enter="refresh" />
      </div>

      <div class="flex items-center gap-2">
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

      <div v-if="activeChips.length > 0" class="flex flex-wrap gap-2 items-center">
        <n-tag v-for="chip in activeChips" :key="chip.key" size="small" closable @close="chip.onClose">
          {{ chip.label }}
        </n-tag>
        <n-button size="tiny" tertiary @click="resetAllFilters">{{ t('common.clear') }}</n-button>
      </div>

      <n-drawer v-model:show="filtersDrawerOpen" placement="bottom" height="80vh">
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

      <n-data-table
        :loading="loading"
        :columns="columns"
        :data="visibleEntries"
        :row-key="(row) => row.path"
        v-model:checked-row-keys="checked"
        :max-height="tableMaxHeight"
      />

      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
        <n-button type="primary" :disabled="checked.length === 0" @click="pick">{{ t('fsPicker.addSelected') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>
