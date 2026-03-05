<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import {
  NButton,
  NSpin,
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
import {
  runEventDisplayMessage,
  runEventLevelTagType,
  runEventSummaryChips,
  RUN_EVENT_DETAIL_HEADER_META_FIELDS_WITH_IDENTIFIERS,
} from '@/lib/run_events'
import { useRunEventsStream, type RunEventsWsStatus } from '@/lib/runEventsStream'
import AppModalShell from '@/components/AppModalShell.vue'
import RunEventDetailDialog from '@/components/runs/RunEventDetailDialog.vue'

export type RunEventsModalExpose = {
  open: (runId: string) => Promise<void>
}

type SummaryChip = { text: string; type: 'default' | 'warning' | 'error' | 'success' }

const { t } = useI18n()
const message = useMessage()
const ui = useUiStore()
const jobs = useJobsStore()

const show = ref<boolean>(false)
const loading = ref<boolean>(false)
const runId = ref<string | null>(null)
const events = ref<RunEvent[]>([])
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
const runEventsStream = useRunEventsStream({
  buildUrl: (id, afterSeq) =>
    wsUrl(`/api/runs/${encodeURIComponent(id)}/events/ws?after_seq=${encodeURIComponent(String(afterSeq))}`),
  validateEvent: (event) => typeof event.ts === 'number',
  onEvent: async (event) => {
    events.value.push(event)
    await nextTick()
    if (follow.value) {
      scrollToLatest()
    } else {
      unseenCount.value += 1
    }
  },
})
const wsStatus = runEventsStream.status
const reconnectInSeconds = runEventsStream.reconnectInSeconds

const detailShow = ref<boolean>(false)
const detailEvent = ref<RunEvent | null>(null)
const detailHeaderMetaFields = RUN_EVENT_DETAIL_HEADER_META_FIELDS_WITH_IDENTIFIERS

function wsUrl(path: string): string {
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${proto}//${window.location.host}${path}`
}

function isAtBottom(el: HTMLElement): boolean {
  const remaining = el.scrollHeight - el.scrollTop - el.clientHeight
  return remaining <= 16
}

function runEventAccentBorderClass(level: string): string {
  if (level === 'error') return 'border-[color:var(--app-danger)]'
  if (level === 'warn' || level === 'warning') return 'border-[color:var(--app-warning)]'
  if (level === 'info') return 'border-[color:var(--app-info)]'
  return 'border-[color:var(--app-border)]'
}

function wsStatusTagType(status: RunEventsWsStatus): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'live') return 'success'
  if (status === 'error') return 'error'
  if (status === 'reconnecting') return 'warning'
  return 'default'
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

function manualReconnect(): void {
  if (!runId.value) return
  runEventsStream.reconnect(runId.value)
}

function eventDisplayMessage(e: RunEvent): string {
  return runEventDisplayMessage(e, t)
}

function formatRelativeSeconds(seconds: number): string {
  const abs = Math.abs(seconds)
  const unit =
    abs >= 86400 ? { n: Math.round(abs / 86400), s: 'd' } : abs >= 3600
      ? { n: Math.round(abs / 3600), s: 'h' }
      : abs >= 60
        ? { n: Math.round(abs / 60), s: 'm' }
        : { n: Math.round(abs), s: 's' }
  const value = `${unit.n}${t(`common.timeUnits.${unit.s}`)}`
  return seconds >= 0 ? t('common.relativeTime.in', { value }) : t('common.relativeTime.ago', { value })
}

function pickSummaryChips(e: RunEvent): SummaryChip[] {
  return runEventSummaryChips(e, t, { nowTs: nowTick.value, maxChips: 3 })
}

async function open(id: string): Promise<void> {
  show.value = true
  runId.value = id
  loading.value = true
  events.value = []
  runEventsStream.setLastSeq(0)
  setFollowEnabled(true)
  unseenCount.value = 0
  runEventsStream.stop()
  startNowTicker()

  try {
    const initial = await jobs.listRunEvents(id)
    events.value = initial
    const maxSeq = initial.reduce((m, e) => Math.max(m, e.seq), 0)
    runEventsStream.setLastSeq(maxSeq)
    await nextTick()
    scrollToLatest()
    runEventsStream.start(id, maxSeq)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchRunEventsFailed'), error, t))
    runEventsStream.stop()
  } finally {
    loading.value = false
  }
}

watch(show, (open) => {
  if (!open) {
    runEventsStream.stop()
    stopNowTicker()
    detailShow.value = false
    detailEvent.value = null
  }
})

onBeforeUnmount(() => {
  runEventsStream.stop()
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
  <AppModalShell v-model:show="show" :width="MODAL_WIDTH.lg" :title="t('runEvents.title')">
    <div class="text-sm app-text-muted flex flex-wrap items-center gap-2 justify-between">
      <div class="flex items-center gap-2 min-w-0">
        <span class="truncate">{{ runId }}</span>
        <n-tag
          size="small"
          :type="wsStatusTagType(wsStatus)"
        >
          {{ t(`runEvents.ws.${wsStatus}`) }}
        </n-tag>
        <span v-if="wsStatus === 'reconnecting' && reconnectInSeconds != null" class="text-xs app-text-muted tabular-nums">
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
          <span class="text-xs app-text-muted">{{ t('runEvents.actions.follow') }}</span>
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

    <div v-if="events.length === 0" class="text-sm app-text-muted">{{ t('runEvents.noEvents') }}</div>

    <div
      v-else
      ref="listEl"
      data-testid="run-events-list"
      class="max-h-[65vh] overflow-auto rounded-md p-1 space-y-1 bg-[var(--n-color)] ring-1 ring-[color:var(--app-border)]"
      @scroll="handleListScroll"
    >
      <div
        v-for="item in events"
        :key="item.seq"
        data-testid="run-event-row"
        class="px-2 py-1 rounded-md border-l-2 font-mono text-xs cursor-pointer transition-colors hover:bg-[var(--app-hover)]"
        :class="runEventAccentBorderClass(item.level)"
        @click="openEventDetails(item)"
      >
        <template v-if="isDesktop">
          <div class="flex items-center gap-1.5">
            <span
              class="app-text-muted shrink-0 tabular-nums whitespace-nowrap leading-4"
              :title="formatUnixSeconds(item.ts)"
            >
              {{ formatListUnixSeconds(item.ts) }}
            </span>
            <n-tag class="shrink-0 w-16 inline-flex justify-center" size="tiny" :type="runEventLevelTagType(item.level)">
              <span class="block w-full truncate text-center">{{ item.level }}</span>
            </n-tag>
            <span class="app-text-muted shrink-0 w-24 truncate">{{ item.kind }}</span>
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
            <span class="min-w-0 flex-1 truncate">{{ eventDisplayMessage(item) }}</span>
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
          <div class="flex items-center gap-1.5">
            <span
              class="app-text-muted shrink-0 tabular-nums whitespace-nowrap leading-4"
              :title="formatUnixSeconds(item.ts)"
            >
              {{ formatListUnixSeconds(item.ts) }}
            </span>
            <n-tag class="shrink-0 w-16 inline-flex justify-center" size="tiny" :type="runEventLevelTagType(item.level)">
              <span class="block w-full truncate text-center">{{ item.level }}</span>
            </n-tag>
            <span class="min-w-0 flex-1 truncate">{{ eventDisplayMessage(item) }}</span>
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
            <span class="app-text-muted truncate max-w-[40%]">{{ item.kind }}</span>
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

    <RunEventDetailDialog
      v-model:show="detailShow"
      :event="detailEvent"
      :is-desktop="isDesktop"
      :title="t('runEvents.details.title')"
      :close-label="t('common.close')"
      :header-meta-fields="detailHeaderMetaFields"
    />

    <template #footer>
      <n-button @click="show = false">{{ t('common.close') }}</n-button>
    </template>
  </AppModalShell>
</template>
