<script setup lang="ts">
import { computed, h, ref } from 'vue'
import { NButton, NDataTable, NInput, NModal, NSpace, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { apiFetch } from '@/lib/api'
import { MODAL_WIDTH } from '@/lib/modal'
import { formatToastError } from '@/lib/errors'

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

const show = ref<boolean>(false)
const loading = ref<boolean>(false)
const loadingMore = ref<boolean>(false)

const runId = ref<string | null>(null)
const prefix = ref<string>('')
const entries = ref<RunEntry[]>([])
const nextCursor = ref<number | null>(null)

const selected = ref<Map<string, 'file' | 'dir'>>(new Map())
const checkedRowKeys = computed<string[]>(() => Array.from(selected.value.keys()))

const selectedCount = computed(() => selected.value.size)

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

async function fetchPage(cursor: number, append: boolean): Promise<void> {
  const id = runId.value
  if (!id) return

  const url =
    `/api/runs/${encodeURIComponent(id)}/entries?` +
    `prefix=${encodeURIComponent(prefix.value)}&cursor=${encodeURIComponent(String(cursor))}&limit=${encodeURIComponent('200')}`

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
      if (row.kind === 'dir') {
        return h(
          'button',
          {
            class: 'text-left w-full text-[var(--n-primary-color)] hover:underline',
            onClick: () => enterDir(row.path),
          },
          label,
        )
      }
      return h('span', null, label)
    },
  },
  {
    title: t('common.type'),
    key: 'kind',
    width: 110,
    render(row) {
      if (row.kind === 'dir') return t('common.dir')
      if (row.kind === 'symlink') return t('common.symlink')
      if (row.kind === 'file') return t('common.file')
      return row.kind
    },
  },
])

defineExpose<RunEntriesPickerModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('restore.pick.title')">
    <div class="space-y-3">
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex items-center gap-2">
          <n-button size="small" @click="up">{{ t('fsPicker.up') }}</n-button>
          <n-button size="small" @click="refresh">{{ t('common.refresh') }}</n-button>
        </div>
        <div class="text-xs opacity-70">{{ t('fsPicker.selectedCount', { count: selectedCount }) }}</div>
      </div>

      <div class="space-y-2">
        <div class="text-xs opacity-70">{{ t('restore.pick.currentPrefix') }}</div>
        <n-input v-model:value="prefix" @keyup.enter="refresh" />
      </div>

      <n-data-table
        :loading="loading"
        :columns="columns"
        :data="entries"
        :row-key="(row) => row.path"
        :checked-row-keys="checkedRowKeys"
        @update:checked-row-keys="updateCheckedRowKeys"
        :max-height="420"
      />

      <div v-if="nextCursor != null" class="flex justify-center">
        <n-button size="small" :loading="loadingMore" @click="loadMore">{{ t('common.more') }}</n-button>
      </div>

      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
        <n-button type="primary" :disabled="selectedCount === 0" @click="pick">{{ t('restore.pick.confirm') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>
