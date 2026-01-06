<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import {
  NButton,
  NCode,
  NModal,
  NSpin,
  NSpace,
  NSwitch,
  NTag,
  NVirtualList,
  useMessage,
  type VirtualListInst,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import { useJobsStore, type RunEvent } from '@/stores/jobs'
import { MODAL_WIDTH } from '@/lib/modal'
import { formatToastError } from '@/lib/errors'
import { useUnixSecondsFormatter } from '@/lib/datetime'

export type RunEventsModalExpose = {
  open: (runId: string) => Promise<void>
}

const { t } = useI18n()
const message = useMessage()
const ui = useUiStore()
const jobs = useJobsStore()

const show = ref<boolean>(false)
const loading = ref<boolean>(false)
const runId = ref<string | null>(null)
const events = ref<RunEvent[]>([])
const wsStatus = ref<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')

let lastSeq = 0
let socket: WebSocket | null = null

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const follow = ref<boolean>(true)
const hasUnseen = ref<boolean>(false)
const listRef = ref<VirtualListInst | null>(null)

const detailShow = ref<boolean>(false)
const detailEvent = ref<RunEvent | null>(null)

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function wsUrl(path: string): string {
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${proto}//${window.location.host}${path}`
}

function isAtBottom(el: HTMLElement): boolean {
  const remaining = el.scrollHeight - el.scrollTop - el.clientHeight
  return remaining <= 16
}

function runEventLevelTagType(level: string): 'success' | 'error' | 'warning' | 'default' {
  if (level === 'error') return 'error'
  if (level === 'warn' || level === 'warning') return 'warning'
  if (level === 'info') return 'success'
  return 'default'
}

function closeSocket(): void {
  if (socket) {
    socket.close()
    socket = null
  }
  wsStatus.value = 'disconnected'
}

function scrollToLatest(): void {
  listRef.value?.scrollTo({ position: 'bottom' })
  hasUnseen.value = false
}

function handleListScroll(e: Event): void {
  const el = e.target as HTMLElement | null
  if (!el) return
  if (hasUnseen.value && isAtBottom(el)) {
    hasUnseen.value = false
  }
}

function openEventDetails(e: RunEvent): void {
  detailEvent.value = e
  detailShow.value = true
}

function connectWs(id: string, afterSeq: number): void {
  closeSocket()
  wsStatus.value = 'connecting'

  const nextSocket = new WebSocket(
    wsUrl(`/api/runs/${encodeURIComponent(id)}/events/ws?after_seq=${encodeURIComponent(String(afterSeq))}`),
  )
  socket = nextSocket

  nextSocket.onopen = () => {
    wsStatus.value = 'connected'
  }

  nextSocket.onmessage = async (evt: MessageEvent) => {
    let parsed: unknown
    try {
      parsed = JSON.parse(String(evt.data)) as unknown
    } catch {
      return
    }

    if (!parsed || typeof parsed !== 'object') return
    const e = parsed as RunEvent
    if (typeof e.seq !== 'number' || typeof e.ts !== 'number') return
    if (e.seq <= lastSeq) return

    lastSeq = e.seq
    events.value.push(e)
    await nextTick()
    if (follow.value) {
      scrollToLatest()
    } else {
      hasUnseen.value = true
    }
  }

  nextSocket.onerror = () => {
    wsStatus.value = 'error'
  }

  nextSocket.onclose = () => {
    wsStatus.value = 'disconnected'
  }
}

async function open(id: string): Promise<void> {
  show.value = true
  runId.value = id
  loading.value = true
  events.value = []
  lastSeq = 0
  follow.value = true
  hasUnseen.value = false

  try {
    const initial = await jobs.listRunEvents(id)
    events.value = initial
    lastSeq = initial.reduce((m, e) => Math.max(m, e.seq), 0)
    await nextTick()
    scrollToLatest()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchRunEventsFailed'), error, t))
  } finally {
    loading.value = false
  }

  connectWs(id, lastSeq)
}

watch(show, (open) => {
  if (!open) {
    closeSocket()
    detailShow.value = false
    detailEvent.value = null
  }
})

onBeforeUnmount(() => {
  closeSocket()
})

watch(follow, (enabled) => {
  if (enabled) {
    nextTick().then(scrollToLatest)
  }
})

defineExpose<RunEventsModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('runEvents.title')">
    <div class="space-y-3">
      <div class="text-sm opacity-70 flex flex-wrap items-center gap-2 justify-between">
        <div class="flex items-center gap-2 min-w-0">
          <span class="truncate">{{ runId }}</span>
          <n-tag
            size="small"
            :type="wsStatus === 'connected' ? 'success' : wsStatus === 'error' ? 'error' : 'default'"
          >
            {{ t(`runEvents.ws.${wsStatus}`) }}
          </n-tag>
          <n-tag v-if="hasUnseen" size="small" type="warning">
            {{ t('runEvents.badges.new') }}
          </n-tag>
        </div>

        <div class="flex items-center gap-2">
          <div class="flex items-center gap-2">
            <span class="text-xs opacity-80">{{ t('runEvents.actions.follow') }}</span>
            <n-switch v-model:value="follow" size="small" />
          </div>
          <n-button v-if="hasUnseen" size="small" @click="scrollToLatest">{{ t('runEvents.actions.latest') }}</n-button>
        </div>
      </div>

      <n-spin v-if="loading" size="small" />

      <div v-if="events.length === 0" class="text-sm opacity-70">{{ t('runEvents.noEvents') }}</div>

      <n-virtual-list
        v-else
        ref="listRef"
        :items="events"
        :item-size="28"
        key-field="seq"
        class="max-h-96 border rounded-md bg-[var(--n-color)]"
        @scroll="handleListScroll"
      >
        <template #default="{ item }">
          <div
            data-testid="run-event-row"
            class="h-7 px-2 border-b last:border-b-0 font-mono text-xs opacity-90 flex items-center gap-2"
          >
            <span class="opacity-70 shrink-0 tabular-nums w-28">{{ formatUnixSeconds(item.ts) }}</span>
            <n-tag class="shrink-0" size="tiny" :type="runEventLevelTagType(item.level)">{{ item.level }}</n-tag>
            <span class="opacity-70 shrink-0 w-24 truncate">{{ item.kind }}</span>
            <span class="min-w-0 flex-1 truncate">{{ item.message }}</span>
            <n-button
              v-if="item.fields"
              size="tiny"
              secondary
              @click="openEventDetails(item)"
            >
              {{ t('runEvents.actions.details') }}
            </n-button>
          </div>
        </template>
      </n-virtual-list>

      <n-modal
        v-model:show="detailShow"
        preset="card"
        :style="{ width: MODAL_WIDTH.md }"
        :title="t('runEvents.details.title')"
      >
        <div v-if="detailEvent" class="space-y-3">
          <div class="text-sm opacity-70 flex flex-wrap items-center gap-2">
            <span class="tabular-nums">{{ formatUnixSeconds(detailEvent.ts) }}</span>
            <n-tag size="small" :type="runEventLevelTagType(detailEvent.level)">{{ detailEvent.level }}</n-tag>
            <span class="opacity-70">{{ detailEvent.kind }}</span>
          </div>
          <div class="font-mono text-sm whitespace-pre-wrap break-words">{{ detailEvent.message }}</div>
          <n-code
            v-if="detailEvent.fields"
            :code="formatJson(detailEvent.fields)"
            language="json"
          />
          <n-space justify="end">
            <n-button @click="detailShow = false">{{ t('common.close') }}</n-button>
          </n-space>
        </div>
      </n-modal>

      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.close') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>
