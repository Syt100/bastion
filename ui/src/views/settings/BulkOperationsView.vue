<script setup lang="ts">
import { computed, h, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NDataTable, NModal, NRadioButton, NRadioGroup, NSpace, NTag, useMessage, type DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import { bulkOperationItemStatusLabel, bulkOperationKindLabel, bulkOperationStatusLabel, filterBulkOperationItems, type BulkOperationItemFilter } from '@/lib/bulkOperations'
import { MODAL_WIDTH } from '@/lib/modal'
import { formatToastError } from '@/lib/errors'
import { useUiStore } from '@/stores/ui'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { useBulkOperationsStore, type BulkOperationDetail, type BulkOperationListItem, type BulkOperationStatus, type BulkOperationItemDetail } from '@/stores/bulkOperations'

const { t } = useI18n()
const message = useMessage()

const ui = useUiStore()
const bulk = useBulkOperationsStore()
const route = useRoute()
const router = useRouter()

const loading = ref<boolean>(false)
const items = ref<BulkOperationListItem[]>([])

const detailOpen = ref<boolean>(false)
const detailLoading = ref<boolean>(false)
const detail = ref<BulkOperationDetail | null>(null)
const detailOpId = ref<string | null>(null)
const detailItemFilter = ref<BulkOperationItemFilter>('all')
const visibleDetailItems = computed(() => filterBulkOperationItems(detail.value?.items ?? [], detailItemFilter.value))

let autoRefreshTimer: number | null = null

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

function statusTagType(status: BulkOperationStatus): 'default' | 'success' | 'warning' | 'error' {
  if (status === 'done') return 'success'
  if (status === 'running') return 'warning'
  if (status === 'canceled') return 'default'
  return 'default'
}

function itemStatusTagType(status: BulkOperationItemDetail['status']): 'default' | 'success' | 'warning' | 'error' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'running') return 'warning'
  return 'default'
}

async function refreshList(): Promise<void> {
  loading.value = true
  try {
    items.value = await bulk.list()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchBulkOperationsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

async function refreshDetail(): Promise<void> {
  if (!detailOpId.value) return
  detailLoading.value = true
  try {
    detail.value = await bulk.get(detailOpId.value)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchBulkOperationFailed'), error, t))
  } finally {
    detailLoading.value = false
  }
}

async function openDetail(opId: string): Promise<void> {
  detailOpId.value = opId
  detailItemFilter.value = 'all'
  detailOpen.value = true
  await refreshDetail()
}

function closeDetail(): void {
  stopAutoRefresh()
  detailOpen.value = false
  detailOpId.value = null
  detail.value = null
  const q = { ...route.query }
  delete q.open
  router.replace({ query: q })
}

function stopAutoRefresh(): void {
  if (autoRefreshTimer === null) return
  window.clearInterval(autoRefreshTimer)
  autoRefreshTimer = null
}

function startAutoRefresh(): void {
  if (autoRefreshTimer !== null) return

  autoRefreshTimer = window.setInterval(async () => {
    if (!detailOpen.value) return
    if (!detailOpId.value) return
    if (detail.value?.status !== 'running') {
      stopAutoRefresh()
      return
    }
    if (detailLoading.value) return

    try {
      await Promise.all([refreshDetail(), refreshList()])
    } catch {
      // Best-effort; users can manually refresh.
      stopAutoRefresh()
    }
  }, 2000)
}

async function retryFailed(): Promise<void> {
  if (!detailOpId.value) return
  try {
    await bulk.retryFailed(detailOpId.value)
    await refreshDetail()
    await refreshList()
    message.success(t('messages.bulkRetryScheduled'))
  } catch (error) {
    message.error(formatToastError(t('errors.retryBulkOperationFailed'), error, t))
  }
}

async function cancelOp(): Promise<void> {
  if (!detailOpId.value) return
  try {
    await bulk.cancel(detailOpId.value)
    await refreshDetail()
    await refreshList()
    message.success(t('messages.bulkOperationCanceled'))
  } catch (error) {
    message.error(formatToastError(t('errors.cancelBulkOperationFailed'), error, t))
  }
}

const columns = computed<DataTableColumns<BulkOperationListItem>>(() => [
  {
    title: t('bulk.columns.kind'),
    key: 'kind',
    render: (row) => bulkOperationKindLabel(t, row.kind),
  },
  {
    title: t('bulk.columns.status'),
    key: 'status',
    render: (row) =>
      h(NTag, { type: statusTagType(row.status), size: 'small' }, { default: () => bulkOperationStatusLabel(t, row.status) }),
  },
  {
    title: t('bulk.columns.progress'),
    key: 'progress',
    render: (row) => `${row.success + row.failed + row.canceled}/${row.total}`,
  },
  {
    title: t('bulk.columns.createdAt'),
    key: 'created_at',
    render: (row) => formatUnixSeconds(row.created_at),
  },
  {
    title: t('bulk.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NButton,
        { tertiary: true, size: 'small', onClick: () => openDetail(row.id) },
        { default: () => t('common.browse') },
      ),
  },
])

const itemColumns = computed<DataTableColumns<BulkOperationItemDetail>>(() => [
  {
    title: t('bulk.columns.agent'),
    key: 'agent',
    render: (row) => row.agent_name ?? row.agent_id,
  },
  {
    title: t('bulk.columns.status'),
    key: 'status',
    render: (row) =>
      h(NTag, { type: itemStatusTagType(row.status), size: 'small' }, { default: () => bulkOperationItemStatusLabel(t, row.status) }),
  },
  {
    title: t('bulk.columns.attempts'),
    key: 'attempts',
    render: (row) => row.attempts,
  },
  {
    title: t('bulk.columns.lastError'),
    key: 'last_error',
    render: (row) => row.last_error ?? '-',
  },
])

const canRetryFailed = computed(() => (detail.value?.failed ?? 0) > 0 && detail.value?.status !== 'canceled')
const canCancel = computed(() => detail.value != null && detail.value.status !== 'canceled' && detail.value.status !== 'done')

watch(
  [detailOpen, () => detail.value?.status],
  ([open, status]) => {
    if (!open || status !== 'running') {
      stopAutoRefresh()
      return
    }
    startAutoRefresh()
  },
  { immediate: true },
)

onMounted(async () => {
  await refreshList()

  const open = route.query.open
  if (typeof open === 'string' && open.trim().length > 0) {
    await openDetail(open)
  }
})

onBeforeUnmount(() => {
  stopAutoRefresh()
})

watch(
  () => route.query.open,
  async (value) => {
    if (typeof value === 'string' && value.trim().length > 0) {
      await openDetail(value)
    }
  },
)
</script>

<template>
  <n-card class="app-card" :bordered="false" :title="t('bulk.title')">
    <template #header-extra>
      <n-button size="small" :loading="loading" @click="refreshList">{{ t('common.refresh') }}</n-button>
    </template>

    <div class="space-y-4">
      <div class="app-help-text">{{ t('bulk.subtitle') }}</div>

      <AppEmptyState v-if="loading && items.length === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!loading && items.length === 0" :title="t('common.noData')" />

      <div v-else class="overflow-x-auto">
        <n-data-table :columns="columns" :data="items" :loading="loading" />
      </div>
    </div>
  </n-card>

  <n-modal v-model:show="detailOpen" preset="card" :style="{ width: MODAL_WIDTH.lg }" :title="t('bulk.detailTitle')">
    <div class="space-y-4">
      <div v-if="detail" class="space-y-2">
        <div class="text-sm">
          <span class="app-text-muted">{{ t('bulk.columns.kind') }}:</span>
          <span class="ml-2">{{ bulkOperationKindLabel(t, detail.kind) }}</span>
        </div>
        <div class="text-sm">
          <span class="app-text-muted">{{ t('bulk.columns.status') }}:</span>
          <span class="ml-2">
            <n-tag :type="statusTagType(detail.status)" size="small">{{ bulkOperationStatusLabel(t, detail.status) }}</n-tag>
          </span>
        </div>
        <div class="text-sm">
          <span class="app-text-muted">{{ t('bulk.columns.progress') }}:</span>
          <span class="ml-2">{{ detail.success + detail.failed + detail.canceled }}/{{ detail.total }}</span>
        </div>
      </div>

      <n-space justify="end">
        <n-button @click="refreshDetail">{{ t('common.refresh') }}</n-button>
        <n-button :disabled="!canRetryFailed" @click="retryFailed">{{ t('bulk.actions.retryFailed') }}</n-button>
        <n-button type="warning" :disabled="!canCancel" @click="cancelOp">{{ t('bulk.actions.cancel') }}</n-button>
        <n-button @click="closeDetail">{{ t('common.close') }}</n-button>
      </n-space>

      <n-card size="small" class="app-card" :bordered="false">
        <div class="flex items-center justify-between gap-3 flex-wrap mb-3">
          <div class="text-sm app-text-muted">{{ t('bulk.filters.items') }}</div>
          <n-radio-group v-model:value="detailItemFilter" size="small">
            <n-radio-button value="all">{{ t('bulk.filters.all') }}</n-radio-button>
            <n-radio-button value="failed">
              {{ t('bulk.filters.failedOnly', { count: detail?.failed ?? 0 }) }}
            </n-radio-button>
          </n-radio-group>
        </div>
        <div class="overflow-x-auto">
          <n-data-table :columns="itemColumns" :data="visibleDetailItems" :loading="detailLoading" />
        </div>
      </n-card>
    </div>
  </n-modal>
</template>
