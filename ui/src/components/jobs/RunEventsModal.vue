<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { NButton, NCode, NModal, NSpin, NSpace, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import { useJobsStore, type RunEvent } from '@/stores/jobs'
import { MODAL_WIDTH } from '@/lib/modal'

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

const scrollEl = ref<HTMLElement | null>(null)

let lastSeq = 0
let socket: WebSocket | null = null

const dateFormatter = computed(
  () =>
    new Intl.DateTimeFormat(ui.locale, {
      dateStyle: 'medium',
      timeStyle: 'medium',
    }),
)

function formatUnixSeconds(ts: number | null): string {
  if (!ts) return '-'
  return dateFormatter.value.format(new Date(ts * 1000))
}

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

function connectWs(id: string): void {
  closeSocket()
  wsStatus.value = 'connecting'

  const nextSocket = new WebSocket(wsUrl(`/api/runs/${encodeURIComponent(id)}/events/ws`))
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
    if (scrollEl.value) scrollEl.value.scrollTop = scrollEl.value.scrollHeight
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

  try {
    const initial = await jobs.listRunEvents(id)
    events.value = initial
    lastSeq = initial.reduce((m, e) => Math.max(m, e.seq), 0)
    await nextTick()
    if (scrollEl.value) scrollEl.value.scrollTop = scrollEl.value.scrollHeight
  } catch {
    message.error(t('errors.fetchRunEventsFailed'))
  } finally {
    loading.value = false
  }

  connectWs(id)
}

watch(show, (open) => {
  if (!open) closeSocket()
})

onBeforeUnmount(() => {
  closeSocket()
})

defineExpose<RunEventsModalExpose>({ open })
</script>

<template>
  <n-modal v-model:show="show" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('runEvents.title')">
    <div class="space-y-3">
      <div class="text-sm opacity-70 flex items-center gap-2">
        <span>{{ runId }}</span>
        <n-tag
          size="small"
          :type="wsStatus === 'connected' ? 'success' : wsStatus === 'error' ? 'error' : 'default'"
        >
          {{ t(`runEvents.ws.${wsStatus}`) }}
        </n-tag>
      </div>

      <n-spin v-if="loading" size="small" />

      <div
        ref="scrollEl"
        id="run-events-scroll"
        class="max-h-96 overflow-auto border rounded-md p-2 bg-[var(--n-color)]"
      >
        <div v-if="events.length === 0" class="text-sm opacity-70">{{ t('runEvents.noEvents') }}</div>
        <div v-for="e in events" :key="e.seq" class="font-mono text-xs py-1 border-b last:border-b-0 opacity-90">
          <div class="flex flex-wrap gap-2 items-center">
            <span class="opacity-70">{{ formatUnixSeconds(e.ts) }}</span>
            <n-tag size="tiny" :type="runEventLevelTagType(e.level)">{{ e.level }}</n-tag>
            <span class="opacity-70">{{ e.kind }}</span>
            <span>{{ e.message }}</span>
          </div>
          <n-code v-if="e.fields" class="mt-1" :code="formatJson(e.fields)" language="json" />
        </div>
      </div>

      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.close') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>
