<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import {
  NButton,
  NCard,
  NDataTable,
  NSelect,
  NTag,
  NSpace,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useNotificationsStore, type NotificationChannel, type NotificationQueueItem } from '@/stores/notifications'
import { useUiStore } from '@/stores/ui'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { useLatestRequest } from '@/lib/latest'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const notifications = useNotificationsStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const loading = ref(false)

type StatusFilter = 'all' | 'queued' | 'sending' | 'sent' | 'failed' | 'canceled'
type ChannelFilter = 'all' | NotificationChannel

const statusFilter = ref<StatusFilter>('all')
const channelFilter = ref<ChannelFilter>('all')
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const items = ref<NotificationQueueItem[]>([])

const latest = useLatestRequest()

function isAbortError(error: unknown): boolean {
  if (!error || typeof error !== 'object') return false
  if (typeof DOMException !== 'undefined' && error instanceof DOMException) return error.name === 'AbortError'
  return 'name' in error && (error as { name?: unknown }).name === 'AbortError'
}

const statusOptions = computed(() => [
  { label: t('settings.notifications.status.all'), value: 'all' as const },
  { label: t('settings.notifications.status.queued'), value: 'queued' },
  { label: t('settings.notifications.status.sending'), value: 'sending' },
  { label: t('settings.notifications.status.sent'), value: 'sent' },
  { label: t('settings.notifications.status.failed'), value: 'failed' },
  { label: t('settings.notifications.status.canceled'), value: 'canceled' },
])

const channelOptions = computed(() => [
  { label: t('settings.notifications.channel.all'), value: 'all' as const },
  { label: t('settings.notifications.channel.wecom'), value: 'wecom_bot' },
  { label: t('settings.notifications.channel.email'), value: 'email' },
])

function formatChannel(channel: NotificationChannel): string {
  return channel === 'wecom_bot' ? t('settings.notifications.channel.wecom') : t('settings.notifications.channel.email')
}

function formatStatus(status: string): string {
  const map: Record<string, string> = {
    queued: t('settings.notifications.status.queued'),
    sending: t('settings.notifications.status.sending'),
    sent: t('settings.notifications.status.sent'),
    failed: t('settings.notifications.status.failed'),
    canceled: t('settings.notifications.status.canceled'),
  }
  return map[status] ?? status
}

async function refresh(): Promise<void> {
  const req = latest.next()
  loading.value = true
  try {
    const res = await notifications.listQueue({
      status: statusFilter.value === 'all' ? undefined : statusFilter.value,
      channel: channelFilter.value === 'all' ? undefined : channelFilter.value,
      page: page.value,
      pageSize: pageSize.value,
      signal: req.signal,
    })
    if (req.isStale()) return
    items.value = res.items
    total.value = res.total
    page.value = res.page
    pageSize.value = res.page_size
  } catch (e) {
    if (req.isStale() || isAbortError(e)) return
    message.error(formatToastError(t('errors.fetchNotificationQueueFailed'), e, t))
  } finally {
    if (req.isStale()) return
    loading.value = false
  }
}

async function retryNow(id: string): Promise<void> {
  try {
    await notifications.retryNow(id)
    message.success(t('messages.notificationRetryScheduled'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.notificationRetryFailed'), e, t))
  }
}

async function cancel(id: string): Promise<void> {
  try {
    await notifications.cancel(id)
    message.success(t('messages.notificationCanceled'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.notificationCancelFailed'), e, t))
  }
}

watch([statusFilter, channelFilter], () => {
  page.value = 1
  void refresh()
})

onMounted(refresh)

const columns = computed<DataTableColumns<NotificationQueueItem>>(() => [
  { title: t('settings.notifications.queue.columns.job'), key: 'job_name' },
  {
    title: t('settings.notifications.queue.columns.channel'),
    key: 'channel',
    render: (row) => formatChannel(row.channel),
  },
  { title: t('settings.notifications.queue.columns.destination'), key: 'destination' },
  {
    title: t('settings.notifications.queue.columns.status'),
    key: 'status',
    render: (row) =>
      h(
        NSpace,
        { size: 6 },
        {
          default: () => [
            h(
              NTag,
              {
                size: 'small',
                type:
                  row.status === 'sent'
                    ? 'success'
                    : row.status === 'failed'
                      ? 'error'
                      : row.status === 'canceled'
                        ? 'warning'
                        : 'default',
                bordered: false,
              },
              { default: () => formatStatus(row.status) },
            ),
            row.destination_deleted
              ? h(
                  NTag,
                  { size: 'small', type: 'warning', bordered: false },
                  { default: () => t('settings.notifications.destinationDeleted') },
                )
              : null,
            !row.destination_enabled && !row.destination_deleted
              ? h(
                  NTag,
                  { size: 'small', type: 'warning', bordered: false },
                  { default: () => t('settings.notifications.destinationDisabled') },
                )
              : null,
          ],
        },
      ),
  },
  {
    title: t('settings.notifications.queue.columns.nextAttempt'),
    key: 'next_attempt_at',
    render: (row) => (row.status === 'queued' ? formatUnixSeconds(row.next_attempt_at) : '-'),
  },
  { title: t('settings.notifications.queue.columns.attempts'), key: 'attempts' },
  {
    title: t('settings.notifications.queue.columns.updatedAt'),
    key: 'updated_at',
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('settings.notifications.queue.columns.lastError'),
    key: 'last_error',
    render: (row) => row.last_error ?? '-',
  },
  {
    title: t('settings.notifications.queue.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NButton,
              {
                size: 'small',
                disabled: !(row.status === 'failed' || row.status === 'canceled'),
                onClick: () => void retryNow(row.id),
              },
              { default: () => t('settings.notifications.queue.actions.retryNow') },
            ),
            h(
              NButton,
              {
                size: 'small',
                disabled: row.status !== 'queued',
                type: 'warning',
                tertiary: true,
                onClick: () => void cancel(row.id),
              },
              { default: () => t('settings.notifications.queue.actions.cancel') },
            ),
          ],
        },
      ),
  },
])
</script>

<template>
  <n-card class="app-card" :title="t('settings.notifications.queueTitle')">
    <div class="space-y-4">
      <div class="flex flex-wrap items-center gap-2">
        <n-select v-model:value="statusFilter" :options="statusOptions" class="w-48" />
        <n-select v-model:value="channelFilter" :options="channelOptions" class="w-48" />
        <n-button :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
      </div>

      <div v-if="!isDesktop" class="space-y-3">
        <n-card
          v-for="row in items"
          :key="row.id"
          size="small"
          class="app-card"
        >
          <template #header>
            <div class="flex items-center justify-between gap-3">
              <div class="font-medium truncate">{{ row.job_name }}</div>
              <n-tag size="small" :bordered="false">
                {{ formatStatus(row.status) }}
              </n-tag>
            </div>
          </template>

          <div class="text-xs opacity-70 space-y-1">
            <div>{{ t('settings.notifications.queue.columns.channel') }}: {{ formatChannel(row.channel) }}</div>
            <div>{{ t('settings.notifications.queue.columns.destination') }}: {{ row.destination }}</div>
            <div v-if="row.status === 'queued'">
              {{ t('settings.notifications.queue.columns.nextAttempt') }}: {{ formatUnixSeconds(row.next_attempt_at) }}
            </div>
            <div>{{ t('settings.notifications.queue.columns.attempts') }}: {{ row.attempts }}</div>
            <div>{{ t('settings.notifications.queue.columns.updatedAt') }}: {{ formatUnixSeconds(row.updated_at) }}</div>
            <div v-if="row.last_error">{{ t('settings.notifications.queue.columns.lastError') }}: {{ row.last_error }}</div>
            <div v-if="row.destination_deleted">{{ t('settings.notifications.destinationDeleted') }}</div>
            <div v-else-if="!row.destination_enabled">{{ t('settings.notifications.destinationDisabled') }}</div>
          </div>

          <div class="mt-3 flex items-center justify-end gap-2">
            <n-button size="small" :disabled="!(row.status === 'failed' || row.status === 'canceled')" @click="retryNow(row.id)">
              {{ t('settings.notifications.queue.actions.retryNow') }}
            </n-button>
            <n-button size="small" type="warning" tertiary :disabled="row.status !== 'queued'" @click="cancel(row.id)">
              {{ t('settings.notifications.queue.actions.cancel') }}
            </n-button>
          </div>
        </n-card>
      </div>

      <div v-else class="overflow-x-auto">
        <n-data-table :loading="loading" :columns="columns" :data="items" />
      </div>

      <div class="flex items-center justify-between text-sm">
        <div class="opacity-70">{{ t('settings.notifications.queue.total', { total }) }}</div>
        <div class="flex items-center gap-2">
          <n-button size="small" :disabled="page <= 1" @click="page -= 1; refresh()">{{ t('common.back') }}</n-button>
          <div class="text-xs opacity-70">{{ page }}</div>
          <n-button
            size="small"
            :disabled="page * pageSize >= total"
            @click="page += 1; refresh()"
          >
            {{ t('common.next') }}
          </n-button>
        </div>
      </div>
    </div>
  </n-card>
</template>
