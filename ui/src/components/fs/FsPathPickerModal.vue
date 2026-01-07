<script setup lang="ts">
import { computed, h, ref } from 'vue'
import {
  NButton,
  NDataTable,
  NInput,
  NModal,
  NSelect,
  NSpace,
  NSwitch,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { ApiError, apiFetch } from '@/lib/api'
import { MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { formatBytes } from '@/lib/format'
import { formatToastError } from '@/lib/errors'
import { useUiStore } from '@/stores/ui'

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
const ui = useUiStore()
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

const selectedCount = computed(() => checked.value.length)

const hasSearchDraftChanges = computed(() => searchDraft.value.trim() !== searchApplied.value)

const kindOptions = computed(() => [
  { label: t('fsPicker.kindAll'), value: 'all' as const },
  { label: t('common.dir'), value: 'dir' as const },
  { label: t('common.file'), value: 'file' as const },
  { label: t('common.symlink'), value: 'symlink' as const },
])

const visibleEntries = computed(() => {
  const needle = searchApplied.value.trim().toLowerCase()
  return entries.value.filter((e) => {
    if (hideDotfiles.value && e.name.startsWith('.')) return false
    if (kindFilter.value !== 'all' && e.kind !== kindFilter.value) return false
    if (needle) return e.name.toLowerCase().includes(needle)
    return true
  })
})

const timeFormatDesktop = computed(
  () => new Intl.DateTimeFormat(ui.locale, { dateStyle: 'medium', timeStyle: 'medium' }),
)
const timeFormatMobile = computed(
  () => new Intl.DateTimeFormat(ui.locale, { dateStyle: 'medium', timeStyle: 'short' }),
)

function formatMtimeDesktop(ts?: number | null): string {
  if (!Number.isFinite(ts as number) || !ts) return '-'
  return timeFormatDesktop.value.format(new Date(ts * 1000))
}

function formatMtimeMobile(ts?: number | null): string {
  if (!Number.isFinite(ts as number) || !ts) return '-'
  return timeFormatMobile.value.format(new Date(ts * 1000))
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
  searchDraft.value = ''
  searchApplied.value = ''
  kindFilter.value = 'all'
  hideDotfiles.value = false
  show.value = true
  void refresh()
}

function applySearch(): void {
  searchApplied.value = searchDraft.value.trim()
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

      <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:flex-1">
          <div class="flex items-center gap-2 sm:flex-1">
            <n-input
              v-model:value="searchDraft"
              :placeholder="t('fsPicker.searchPlaceholder')"
              @keyup.enter="applySearch"
            />
            <n-button size="small" :disabled="!hasSearchDraftChanges" @click="applySearch">
              {{ t('fsPicker.search') }}
            </n-button>
          </div>
          <div class="flex items-center gap-2">
            <n-select v-model:value="kindFilter" size="small" :options="kindOptions" />
            <div class="flex items-center gap-2">
              <n-switch v-model:value="hideDotfiles" size="small" />
              <div class="text-xs opacity-70">{{ t('common.hideDotfiles') }}</div>
            </div>
          </div>
        </div>
      </div>

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
