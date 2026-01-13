<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import {
  NButton,
  NCode,
  NDrawer,
  NDrawerContent,
  NModal,
  NSpin,
  NSpace,
  NSwitch,
  NTag,
  useMessage,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/stores/ui'
import { useJobsStore, type RunEvent } from '@/stores/jobs'
import { MODAL_WIDTH } from '@/lib/modal'
import { formatToastError } from '@/lib/errors'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { MQ } from '@/lib/breakpoints'
import { useMediaQuery } from '@/lib/media'

export type RunEventsModalExpose = {
  open: (runId: string) => Promise<void>
}

type WsStatus = 'disconnected' | 'connecting' | 'live' | 'reconnecting' | 'error'
type SummaryChip = { text: string; type: 'default' | 'warning' | 'error' | 'success' }

const { t } = useI18n()
const message = useMessage()
const ui = useUiStore()
const jobs = useJobsStore()

const show = ref<boolean>(false)
const loading = ref<boolean>(false)
const runId = ref<string | null>(null)
const events = ref<RunEvent[]>([])
const wsStatus = ref<WsStatus>('disconnected')

let lastSeq = 0
let socket: WebSocket | null = null
let allowReconnect = false
let reconnectTimer: number | null = null
let reconnectCountdownTimer: number | null = null
let nowTicker: number | null = null
let suppressAutoUnfollowUntil = 0

const isDesktop = useMediaQuery(MQ.mdUp)
const locale = computed(() => ui.locale)
const { formatUnixSeconds } = useUnixSecondsFormatter(locale)

const listTsFormatter = computed(() => {
  const options: Intl.DateTimeFormatOptions = isDesktop.value
    ? {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false,
      }
    : {
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
      }

  return new Intl.DateTimeFormat(locale.value, options)
})

function formatListUnixSeconds(ts: number | null): string {
  if (!ts) return '-'
  return listTsFormatter.value.format(new Date(ts * 1000))
}

const follow = ref<boolean>(true)
const followDisabledReason = ref<'auto' | 'manual' | null>(null)
const unseenCount = ref<number>(0)
const listEl = ref<HTMLElement | null>(null)

const showLatest = computed(() => !follow.value || unseenCount.value > 0)

const nowTick = ref<number>(Math.floor(Date.now() / 1000))

const reconnectAttempts = ref<number>(0)
const reconnectInSeconds = ref<number | null>(null)

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

function runEventAccentBorderClass(level: string): string {
  if (level === 'error') return 'border-red-500/80'
  if (level === 'warn' || level === 'warning') return 'border-amber-400/80'
  if (level === 'info') return 'border-emerald-400/80'
  if (level === 'debug') return 'border-slate-400/70'
  return 'border-zinc-400/60'
}

function wsStatusTagType(status: WsStatus): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'live') return 'success'
  if (status === 'error') return 'error'
  if (status === 'reconnecting') return 'warning'
  return 'default'
}

function closeSocket(): void {
  if (socket) {
    socket.close()
    socket = null
  }
  if (reconnectTimer !== null) {
    window.clearTimeout(reconnectTimer)
    reconnectTimer = null
  }
  if (reconnectCountdownTimer !== null) {
    window.clearInterval(reconnectCountdownTimer)
    reconnectCountdownTimer = null
  }
  reconnectInSeconds.value = null
  wsStatus.value = 'disconnected'
}

function scrollToLatest(): void {
  const el = listEl.value
  if (!el) return
  suppressAutoUnfollowUntil = Date.now() + 300
  el.scrollTop = el.scrollHeight
  unseenCount.value = 0
}

function setFollowEnabled(enabled: boolean, reason: 'auto' | 'manual' | null = null): void {
  follow.value = enabled
  followDisabledReason.value = enabled ? null : reason
}

function handleFollowUpdate(value: boolean): void {
  setFollowEnabled(value, value ? null : 'manual')
}

function jumpToLatest(): void {
  setFollowEnabled(true)
  nextTick().then(scrollToLatest)
}

function handleListScroll(e: Event): void {
  const el = e.target as HTMLElement | null
  if (!el) return
  const atBottom = isAtBottom(el)
  if (unseenCount.value > 0 && atBottom) unseenCount.value = 0
  if (!follow.value && atBottom && followDisabledReason.value === 'auto') {
    setFollowEnabled(true)
  } else if (follow.value && !atBottom && Date.now() > suppressAutoUnfollowUntil) {
    setFollowEnabled(false, 'auto')
  }
}

function openEventDetails(e: RunEvent): void {
  detailEvent.value = e
  detailShow.value = true
}

function startNowTicker(): void {
  stopNowTicker()
  nowTick.value = Math.floor(Date.now() / 1000)
  nowTicker = window.setInterval(() => {
    nowTick.value = Math.floor(Date.now() / 1000)
  }, 5000)
}

function stopNowTicker(): void {
  if (nowTicker !== null) {
    window.clearInterval(nowTicker)
    nowTicker = null
  }
}

function reconnectDelaySeconds(attempt: number): number {
  // 1s, 2s, 4s, 8s, ... capped.
  const cappedAttempt = Math.max(0, Math.min(10, attempt))
  return Math.min(30, Math.max(1, 1 << cappedAttempt))
}

function scheduleReconnect(id: string): void {
  if (!allowReconnect) return
  reconnectAttempts.value += 1
  const delay = reconnectDelaySeconds(reconnectAttempts.value - 1)
  reconnectInSeconds.value = delay
  wsStatus.value = 'reconnecting'

  if (reconnectCountdownTimer !== null) window.clearInterval(reconnectCountdownTimer)
  reconnectCountdownTimer = window.setInterval(() => {
    if (reconnectInSeconds.value == null) return
    reconnectInSeconds.value = Math.max(0, reconnectInSeconds.value - 1)
  }, 1000)

  if (reconnectTimer !== null) window.clearTimeout(reconnectTimer)
  reconnectTimer = window.setTimeout(() => {
    reconnectTimer = null
    if (!allowReconnect) return
    connectWs(id, lastSeq, true)
  }, delay * 1000)
}

function connectWs(id: string, afterSeq: number, isReconnect: boolean): void {
  closeSocket()
  wsStatus.value = isReconnect ? 'reconnecting' : 'connecting'

  const nextSocket = new WebSocket(
    wsUrl(`/api/runs/${encodeURIComponent(id)}/events/ws?after_seq=${encodeURIComponent(String(afterSeq))}`),
  )
  socket = nextSocket

  nextSocket.onopen = () => {
    wsStatus.value = 'live'
    reconnectAttempts.value = 0
    reconnectInSeconds.value = null
    if (reconnectCountdownTimer !== null) {
      window.clearInterval(reconnectCountdownTimer)
      reconnectCountdownTimer = null
    }
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
      unseenCount.value += 1
    }
  }

  nextSocket.onerror = () => {
    wsStatus.value = 'error'
  }

  nextSocket.onclose = () => {
    socket = null
    if (!allowReconnect) {
      wsStatus.value = 'disconnected'
      return
    }
    scheduleReconnect(id)
  }
}

function manualReconnect(): void {
  if (!runId.value) return
  reconnectAttempts.value = 0
  reconnectInSeconds.value = null
  connectWs(runId.value, lastSeq, false)
}

function normalizeFields(fields: unknown | null): Record<string, unknown> | null {
  if (!fields || typeof fields !== 'object') return null
  if (Array.isArray(fields)) return null
  return fields as Record<string, unknown>
}

function toNumber(value: unknown): number | null {
  if (typeof value !== 'number' || !Number.isFinite(value)) return null
  return value
}

function toString(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const v = value.trim()
  return v ? v : null
}

function shortId(value: string): string {
  if (value.length <= 10) return value
  return `${value.slice(0, 8)}…`
}

function formatDurationMs(ms: number): string {
  if (ms < 1000) return `${Math.max(0, Math.floor(ms))}ms`
  const secs = ms / 1000
  if (secs < 10) return `${secs.toFixed(1)}s`
  return `${Math.round(secs)}s`
}

function formatRelativeSeconds(seconds: number): string {
  const abs = Math.abs(seconds)
  const unit =
    abs >= 86400 ? { n: Math.round(abs / 86400), s: 'd' } : abs >= 3600
      ? { n: Math.round(abs / 3600), s: 'h' }
      : abs >= 60
        ? { n: Math.round(abs / 60), s: 'm' }
        : { n: Math.round(abs), s: 's' }

  const v = `${unit.n}${unit.s}`
  const isZh = locale.value.startsWith('zh')
  if (seconds >= 0) return isZh ? `${v}后` : `in ${v}`
  return isZh ? `${v}前` : `${v} ago`
}

function pickSummaryChips(e: RunEvent): SummaryChip[] {
  const fields = normalizeFields(e.fields)
  if (!fields) return []

  const out: SummaryChip[] = []
  const push = (chip: SummaryChip): void => {
    if (out.length >= 2) return
    out.push(chip)
  }

  const errorKind = toString(fields.error_kind) ?? toString(fields.last_error_kind)
  if (errorKind) {
    const type: SummaryChip['type'] = errorKind === 'auth' || errorKind === 'config' ? 'error' : 'warning'
    push({ text: errorKind, type })
  }

  const attempt = toNumber(fields.attempt) ?? toNumber(fields.attempts)
  if (attempt != null) {
    push({ text: `#${Math.max(0, Math.floor(attempt))}`, type: 'default' })
  }

  const nextAttemptAt = toNumber(fields.next_attempt_at)
  if (nextAttemptAt != null) {
    push({ text: formatRelativeSeconds(nextAttemptAt - nowTick.value), type: 'default' })
  }

  const durationMs = toNumber(fields.duration_ms)
  if (durationMs != null) {
    push({ text: formatDurationMs(durationMs), type: 'default' })
  }

  const errorsTotal = toNumber(fields.errors_total)
  const warningsTotal = toNumber(fields.warnings_total)
  if (errorsTotal != null || warningsTotal != null) {
    const errors = Math.max(0, Math.floor(errorsTotal ?? 0))
    const warnings = Math.max(0, Math.floor(warningsTotal ?? 0))
    push({ text: `E${errors}/W${warnings}`, type: errors > 0 ? 'error' : warnings > 0 ? 'warning' : 'default' })
  }

  const ok = typeof fields.ok === 'boolean' ? fields.ok : null
  if (ok != null) {
    push({ text: ok ? 'OK' : 'FAIL', type: ok ? 'success' : 'error' })
  }

  const channel = toString(fields.channel)
  if (channel) push({ text: channel, type: 'default' })

  const source = toString(fields.source)
  if (source) push({ text: source, type: 'default' })

  const executedOffline = typeof fields.executed_offline === 'boolean' ? fields.executed_offline : null
  if (executedOffline === true) push({ text: t('runs.badges.offline'), type: 'default' })

  const agentId = toString(fields.agent_id)
  if (agentId) push({ text: shortId(agentId), type: 'default' })

  const secretName = toString(fields.secret_name)
  if (secretName) push({ text: secretName, type: 'default' })

  return out
}

async function open(id: string): Promise<void> {
  show.value = true
  runId.value = id
  loading.value = true
  events.value = []
  lastSeq = 0
  setFollowEnabled(true)
  unseenCount.value = 0
  reconnectAttempts.value = 0
  reconnectInSeconds.value = null
  allowReconnect = true
  startNowTicker()

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

  connectWs(id, lastSeq, false)
}

watch(show, (open) => {
  if (!open) {
    allowReconnect = false
    closeSocket()
    stopNowTicker()
    detailShow.value = false
    detailEvent.value = null
  }
})

onBeforeUnmount(() => {
  allowReconnect = false
  closeSocket()
  stopNowTicker()
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
            :type="wsStatusTagType(wsStatus)"
          >
            {{ t(`runEvents.ws.${wsStatus}`) }}
          </n-tag>
          <span v-if="wsStatus === 'reconnecting' && reconnectInSeconds != null" class="text-xs opacity-70 tabular-nums">
            {{ formatRelativeSeconds(reconnectInSeconds) }}
          </span>
          <n-tag v-if="unseenCount > 0" size="small" type="warning">
            {{ t('runEvents.badges.newCount', { count: unseenCount }) }}
          </n-tag>
        </div>

        <div class="flex items-center gap-2 ml-auto">
          <n-button
            v-if="wsStatus === 'disconnected' || wsStatus === 'reconnecting' || wsStatus === 'error'"
            size="small"
            secondary
            @click="manualReconnect"
          >
            {{ t('runEvents.actions.reconnect') }}
          </n-button>
          <div class="flex items-center gap-2">
            <span class="text-xs opacity-80">{{ t('runEvents.actions.follow') }}</span>
            <n-switch :value="follow" size="small" @update:value="handleFollowUpdate" />
          </div>
          <span data-testid="run-events-latest">
            <n-button size="small" :disabled="!showLatest" @click="jumpToLatest">
              {{ t('runEvents.actions.latest') }}
            </n-button>
          </span>
        </div>
      </div>

      <n-spin v-if="loading" size="small" />

      <div v-if="events.length === 0" class="text-sm opacity-70">{{ t('runEvents.noEvents') }}</div>

      <div
        v-else
        ref="listEl"
        data-testid="run-events-list"
        class="max-h-[65vh] overflow-auto rounded-md p-1 space-y-1 bg-[var(--n-color)] ring-1 ring-black/5 dark:ring-white/10"
        @scroll="handleListScroll"
      >
        <div
          v-for="item in events"
          :key="item.seq"
          data-testid="run-event-row"
          class="px-2 py-1 rounded-md border-l-2 font-mono text-xs opacity-90 cursor-pointer transition-colors bg-black/2 hover:bg-black/5 dark:bg-white/5 dark:hover:bg-white/10"
          :class="runEventAccentBorderClass(item.level)"
          @click="openEventDetails(item)"
        >
          <template v-if="isDesktop">
            <div class="flex items-center gap-2">
              <span
                class="opacity-70 shrink-0 tabular-nums whitespace-nowrap leading-4 w-32"
                :title="formatUnixSeconds(item.ts)"
              >
                {{ formatListUnixSeconds(item.ts) }}
              </span>
              <n-tag class="shrink-0" size="tiny" :type="runEventLevelTagType(item.level)">{{ item.level }}</n-tag>
              <span class="opacity-70 shrink-0 w-24 truncate">{{ item.kind }}</span>
              <div class="shrink-0 flex items-center gap-1 min-w-0">
                <n-tag
                  v-for="(chip, idx) in pickSummaryChips(item)"
                  :key="`${item.seq}-chip-${idx}`"
                  size="tiny"
                  :bordered="false"
                  :type="chip.type === 'success' ? 'success' : chip.type === 'error' ? 'error' : chip.type === 'warning' ? 'warning' : 'default'"
                >
                  <span class="inline-block max-w-28 truncate align-bottom">{{ chip.text }}</span>
                </n-tag>
              </div>
              <span class="min-w-0 flex-1 truncate">{{ item.message }}</span>
              <n-button
                v-if="item.fields"
                size="tiny"
                secondary
                @click.stop="openEventDetails(item)"
              >
                {{ t('runEvents.actions.details') }}
              </n-button>
            </div>
          </template>
          <template v-else>
            <div class="flex items-center gap-2">
              <span
                class="opacity-70 shrink-0 tabular-nums whitespace-nowrap leading-4 w-14"
                :title="formatUnixSeconds(item.ts)"
              >
                {{ formatListUnixSeconds(item.ts) }}
              </span>
              <n-tag class="shrink-0" size="tiny" :type="runEventLevelTagType(item.level)">{{ item.level }}</n-tag>
              <span class="min-w-0 flex-1 truncate">{{ item.message }}</span>
              <n-button
                v-if="item.fields"
                size="tiny"
                secondary
                @click.stop="openEventDetails(item)"
              >
                {{ t('runEvents.actions.details') }}
              </n-button>
            </div>

            <div class="mt-0.5 flex items-center gap-2 min-w-0">
              <span class="opacity-70 truncate max-w-[40%]">{{ item.kind }}</span>
              <div class="flex items-center gap-1 min-w-0 overflow-hidden">
                <n-tag
                  v-for="(chip, idx) in pickSummaryChips(item)"
                  :key="`${item.seq}-chip-${idx}`"
                  size="tiny"
                  :bordered="false"
                  :type="chip.type === 'success' ? 'success' : chip.type === 'error' ? 'error' : chip.type === 'warning' ? 'warning' : 'default'"
                >
                  <span class="inline-block max-w-24 truncate align-bottom">{{ chip.text }}</span>
                </n-tag>
              </div>
            </div>
          </template>
        </div>
      </div>

      <n-modal
        v-if="isDesktop"
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

      <n-drawer v-else v-model:show="detailShow" placement="bottom" height="70vh">
        <n-drawer-content :title="t('runEvents.details.title')" closable>
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
          </div>
        </n-drawer-content>
      </n-drawer>

      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.close') }}</n-button>
      </n-space>
    </div>
  </n-modal>
</template>
