<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import {
  NButton,
  NCard,
  NDataTable,
  NTag,
  NSpace,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import ListToolbar from '@/components/list/ListToolbar.vue'
import ListPageScaffold from '@/components/list/ListPageScaffold.vue'
import AppPagination from '@/components/list/AppPagination.vue'
import ListFilterSelectField from '@/components/list/ListFilterSelectField.vue'
import ListActiveFiltersRow from '@/components/list/ListActiveFiltersRow.vue'
import AppEmptyState from '@/components/AppEmptyState.vue'
import { useNotificationsStore, type NotificationChannel, type NotificationQueueItem } from '@/stores/notifications'
import { useUiStore } from '@/stores/ui'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { buildListRangeSummary } from '@/lib/listUi'
import { useLatestRequest } from '@/lib/latest'
import { usePersistentColumnWidths } from '@/lib/columnWidths'
import { createMultiSelectFilterField, parseRouteQueryList, useListFilters } from '@/lib/listFilters'

const { t } = useI18n()
const message = useMessage()
const route = useRoute()

const ui = useUiStore()
const notifications = useNotificationsStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const columnWidths = usePersistentColumnWidths('bastion.ui.tableColumns.settings.notifications.queue')

const loading = ref(false)

type QueueStatus = 'queued' | 'sending' | 'sent' | 'failed' | 'canceled'

const statusFilter = ref<QueueStatus[] | null>([])
const channelFilter = ref<NotificationChannel[] | null>([])
const page = ref(1)
const pageSize = ref(20)
const pageSizeOptions = [20, 50, 100]
const total = ref(0)
const items = ref<NotificationQueueItem[]>([])
const retryBusy = ref<Record<string, boolean>>({})
const cancelBusy = ref<Record<string, boolean>>({})

const latest = useLatestRequest()
const queueRangeSummary = computed(() => buildListRangeSummary(total.value, page.value, pageSize.value))
const queuePaginationLabel = computed(() => t('common.paginationRange', queueRangeSummary.value))

function isQueueStatus(value: string): value is QueueStatus {
  return value === 'queued' || value === 'sending' || value === 'sent' || value === 'failed' || value === 'canceled'
}

function isChannel(value: string): value is NotificationChannel {
  return value === 'wecom_bot' || value === 'email'
}

function applyRouteFilters(): void {
  const statuses = parseRouteQueryList(route.query.status).filter(isQueueStatus)
  const channels = parseRouteQueryList(route.query.channel).filter(isChannel)
  statusFilter.value = statuses
  channelFilter.value = channels
}

applyRouteFilters()

function isAbortError(error: unknown): boolean {
  if (!error || typeof error !== 'object') return false
  if (typeof DOMException !== 'undefined' && error instanceof DOMException) return error.name === 'AbortError'
  return 'name' in error && (error as { name?: unknown }).name === 'AbortError'
}

const statusOptions = computed(() => [
  { label: t('settings.notifications.status.queued'), value: 'queued' },
  { label: t('settings.notifications.status.sending'), value: 'sending' },
  { label: t('settings.notifications.status.sent'), value: 'sent' },
  { label: t('settings.notifications.status.failed'), value: 'failed' },
  { label: t('settings.notifications.status.canceled'), value: 'canceled' },
])

const channelOptions = computed(() => [
  { label: t('settings.notifications.channel.wecom'), value: 'wecom_bot' },
  { label: t('settings.notifications.channel.email'), value: 'email' },
])

const {
  hasActiveFilters,
  activeFilterChips,
  clearFilters: clearListFilters,
} = useListFilters([
  createMultiSelectFilterField({
    key: 'status',
    label: t('settings.notifications.queue.columns.status'),
    value: statusFilter,
    options: () => statusOptions.value,
  }),
  createMultiSelectFilterField({
    key: 'channel',
    label: t('settings.notifications.queue.columns.channel'),
    value: channelFilter,
    options: () => channelOptions.value,
  }),
])

const queueBaseEmpty = computed<boolean>(() => total.value === 0 && !hasActiveFilters.value)

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

function handleColumnResize(_resizedWidth: number, limitedWidth: number, column: unknown): void {
  if (!column || typeof column !== 'object') return
  if (!('key' in column)) return
  const key = (column as { key?: unknown }).key
  if (key === undefined || key === null) return
  columnWidths.setWidth(String(key), limitedWidth)
}

async function refresh(): Promise<void> {
  const req = latest.next()
  loading.value = true
  try {
    const statuses = statusFilter.value ?? []
    const channels = channelFilter.value ?? []
    const res = await notifications.listQueue({
      status: statuses.length ? statuses : undefined,
      channel: channels.length ? channels : undefined,
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
  if (retryBusy.value[id] === true) return
  retryBusy.value = { ...retryBusy.value, [id]: true }
  try {
    await notifications.retryNow(id)
    message.success(t('messages.notificationRetryScheduled'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.notificationRetryFailed'), e, t))
  } finally {
    const next = { ...retryBusy.value }
    delete next[id]
    retryBusy.value = next
  }
}

async function cancel(id: string): Promise<void> {
  if (cancelBusy.value[id] === true) return
  cancelBusy.value = { ...cancelBusy.value, [id]: true }
  try {
    await notifications.cancel(id)
    message.success(t('messages.notificationCanceled'))
    await refresh()
  } catch (e) {
    message.error(formatToastError(t('errors.notificationCancelFailed'), e, t))
  } finally {
    const next = { ...cancelBusy.value }
    delete next[id]
    cancelBusy.value = next
  }
}

function clearFilters(): void {
  clearListFilters()
}

watch(
  () => [route.query.status, route.query.channel],
  () => applyRouteFilters(),
)

watch([statusFilter, channelFilter], () => {
  page.value = 1
  void refresh()
})

function onUpdatePage(next: number): void {
  if (next === page.value) return
  page.value = next
  void refresh()
}

function onUpdatePageSize(next: number): void {
  if (next === pageSize.value) return
  pageSize.value = next
  page.value = 1
  void refresh()
}

onMounted(refresh)

const columns = computed<DataTableColumns<NotificationQueueItem>>(() => [
  {
    title: t('settings.notifications.queue.columns.job'),
    key: 'job_name',
    width: columnWidths.getWidth('job_name') ?? 180,
    minWidth: 140,
    resizable: true,
    render: (row) => h('div', { class: 'min-w-0 truncate', title: row.job_name }, row.job_name),
  },
  {
    title: t('settings.notifications.queue.columns.channel'),
    key: 'channel',
    width: 110,
    maxWidth: 110,
    render: (row) => {
      const label = formatChannel(row.channel)
      return h('div', { class: 'min-w-0 truncate', title: label }, label)
    },
  },
  {
    title: t('settings.notifications.queue.columns.destination'),
    key: 'destination',
    width: columnWidths.getWidth('destination'),
    minWidth: 200,
    resizable: true,
    render: (row) => h('div', { class: 'min-w-0 truncate', title: row.destination }, row.destination),
  },
  {
    title: t('settings.notifications.queue.columns.status'),
    key: 'status',
    width: 190,
    maxWidth: 190,
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
    width: 170,
    maxWidth: 170,
    render: (row) => (row.status === 'queued' ? formatUnixSeconds(row.next_attempt_at) : '-'),
  },
  {
    title: t('settings.notifications.queue.columns.attempts'),
    key: 'attempts',
    width: 84,
    maxWidth: 84,
  },
  {
    title: t('settings.notifications.queue.columns.updatedAt'),
    key: 'updated_at',
    width: 170,
    maxWidth: 170,
    render: (row) => formatUnixSeconds(row.updated_at),
  },
  {
    title: t('settings.notifications.queue.columns.lastError'),
    key: 'last_error',
    width: columnWidths.getWidth('last_error'),
    minWidth: 220,
    resizable: true,
    render: (row) => h('div', { class: 'min-w-0 truncate', title: row.last_error ?? '-' }, row.last_error ?? '-'),
  },
  {
    title: t('settings.notifications.queue.columns.actions'),
    key: 'actions',
    width: 200,
    maxWidth: 200,
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
                loading: retryBusy.value[row.id] === true,
                disabled: !(row.status === 'failed' || row.status === 'canceled') || retryBusy.value[row.id] === true,
                onClick: () => void retryNow(row.id),
              },
              { default: () => t('settings.notifications.queue.actions.retryNow') },
            ),
            h(
              NButton,
              {
                size: 'small',
                loading: cancelBusy.value[row.id] === true,
                disabled: row.status !== 'queued' || cancelBusy.value[row.id] === true,
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
  <n-card class="app-card" :bordered="false" :title="t('settings.notifications.queueTitle')">
    <ListPageScaffold>
      <template #toolbar>
        <ListToolbar embedded compact>
          <template #filters>
            <ListFilterSelectField
              v-model:value="statusFilter"
              multiple
              clearable
              max-tag-count="responsive"
              :placeholder="t('settings.notifications.status.all')"
              :options="statusOptions"
            />
            <ListFilterSelectField
              v-model:value="channelFilter"
              multiple
              clearable
              max-tag-count="responsive"
              :placeholder="t('settings.notifications.channel.all')"
              :options="channelOptions"
            />
          </template>

          <template #actions>
            <n-button size="small" class="w-full md:w-auto" :loading="loading" @click="refresh">{{ t('common.refresh') }}</n-button>
          </template>
        </ListToolbar>
      </template>

      <template #content>
        <ListActiveFiltersRow
          class="mb-3"
          :chips="activeFilterChips"
          :clear-label="t('common.clear')"
          @clear="clearFilters"
        />

        <AppEmptyState
          v-if="loading && items.length === 0"
          :title="t('common.loading')"
          :description="t('settings.notifications.queue.empty.loadingDescription')"
          loading
          variant="plain"
        />
        <AppEmptyState
          v-else-if="!loading && items.length === 0"
          :title="queueBaseEmpty ? t('settings.notifications.queue.empty.title') : t('settings.notifications.queue.empty.noResultsTitle')"
          :description="queueBaseEmpty ? t('settings.notifications.queue.empty.description') : t('settings.notifications.queue.empty.noResultsDescription')"
          variant="plain"
        >
          <template #actions>
            <n-button
              v-if="!queueBaseEmpty"
              size="small"
              @click="clearFilters"
            >
              {{ t('common.clear') }}
            </n-button>
            <n-button size="small" :loading="loading" @click="refresh">
              {{ t('common.refresh') }}
            </n-button>
          </template>
        </AppEmptyState>

        <div v-else-if="!isDesktop" class="space-y-3">
          <n-card
            v-for="row in items"
            :key="row.id"
            size="small"
            class="app-card"
            :bordered="false"
          >
            <template #header>
              <div class="flex items-center justify-between gap-3">
                <div class="font-medium truncate">{{ row.job_name }}</div>
                <n-tag size="small" :bordered="false">
                  {{ formatStatus(row.status) }}
                </n-tag>
              </div>
            </template>

            <div class="text-xs app-text-muted space-y-1">
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
              <n-button
                size="small"
                :loading="retryBusy[row.id] === true"
                :disabled="!(row.status === 'failed' || row.status === 'canceled') || retryBusy[row.id] === true"
                @click="retryNow(row.id)"
              >
                {{ t('settings.notifications.queue.actions.retryNow') }}
              </n-button>
              <n-button
                size="small"
                type="warning"
                tertiary
                :loading="cancelBusy[row.id] === true"
                :disabled="row.status !== 'queued' || cancelBusy[row.id] === true"
                @click="cancel(row.id)"
              >
                {{ t('settings.notifications.queue.actions.cancel') }}
              </n-button>
            </div>
          </n-card>
        </div>

        <div v-else class="overflow-x-auto">
          <n-data-table
            table-layout="fixed"
            :loading="loading"
            :columns="columns"
            :data="items"
            :on-unstable-column-resize="handleColumnResize"
          />
        </div>
      </template>

      <template #footer>
        <AppPagination
          :page="page"
          :page-size="pageSize"
          :item-count="total"
          :page-sizes="pageSizeOptions"
          :loading="loading"
          :total-label="queuePaginationLabel"
          @update:page="onUpdatePage"
          @update:page-size="onUpdatePageSize"
        />
      </template>
    </ListPageScaffold>
  </n-card>
</template>
