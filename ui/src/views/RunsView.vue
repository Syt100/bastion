<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter, type LocationQueryRaw } from 'vue-router'
import {
  NButton,
  NCard,
  NInput,
  NPagination,
  NSelect,
  NTag,
  useMessage,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import PageHeader from '@/components/PageHeader.vue'
import ListPageScaffold from '@/components/list/ListPageScaffold.vue'
import ListToolbar from '@/components/list/ListToolbar.vue'
import { useUiStore } from '@/stores/ui'
import { useRunsStore, type RunKind, type RunWorkspaceListItem } from '@/stores/runs'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { formatToastError } from '@/lib/errors'
import { COMMAND_CENTER_RANGE_OPTIONS, parseCommandCenterRangePreset, resolveRouteScope } from '@/lib/commandCenter'
import { formatCommandCenterScopeLabel } from '@/lib/commandCenterPresentation'
import { buildRunDetailLocation, runKindLabel, runStatusLabel } from '@/lib/runs'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const message = useMessage()

const ui = useUiStore()
const runsStore = useRunsStore()

const loading = ref(false)
const rows = ref<RunWorkspaceListItem[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const searchDraft = ref('')

const effectiveScope = computed(() => resolveRouteScope(route, ui.preferredScope))
const rangePreset = computed(() => parseCommandCenterRangePreset(route.query.range))
const selectedStatus = computed(() => {
  const value = Array.isArray(route.query.status) ? route.query.status[0] : route.query.status
  return typeof value === 'string' && value.length > 0 ? value : 'all'
})
const selectedKind = computed(() => {
  const value = Array.isArray(route.query.kind) ? route.query.kind[0] : route.query.kind
  return typeof value === 'string' && value.length > 0 ? value : 'all'
})
const selectedQuery = computed(() => {
  const value = Array.isArray(route.query.q) ? route.query.q[0] : route.query.q
  return typeof value === 'string' ? value : ''
})
const selectedPage = computed(() => {
  const value = Array.isArray(route.query.page) ? route.query.page[0] : route.query.page
  const parsed = Number.parseInt(typeof value === 'string' ? value : '1', 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 1
})
const scopeLabel = computed(() => formatCommandCenterScopeLabel(String(effectiveScope.value), t))
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

watch(
  selectedQuery,
  (value) => {
    searchDraft.value = value
  },
  { immediate: true },
)

const statusOptions = computed(() => [
  { label: t('runs.filters.all'), value: 'all' },
  { label: runStatusLabel(t, 'queued'), value: 'queued' },
  { label: runStatusLabel(t, 'running'), value: 'running' },
  { label: runStatusLabel(t, 'success'), value: 'success' },
  { label: runStatusLabel(t, 'failed'), value: 'failed' },
  { label: runStatusLabel(t, 'rejected'), value: 'rejected' },
  { label: runStatusLabel(t, 'canceled'), value: 'canceled' },
])

const kindOptions = computed(() => [
  { label: t('runs.filters.all'), value: 'all' },
  { label: runKindLabel(t, 'backup'), value: 'backup' },
  { label: runKindLabel(t, 'restore'), value: 'restore' },
  { label: runKindLabel(t, 'verify'), value: 'verify' },
  { label: runKindLabel(t, 'cleanup'), value: 'cleanup' },
])

function statusTagType(status: string): 'success' | 'error' | 'warning' | 'default' {
  if (status === 'success') return 'success'
  if (status === 'failed') return 'error'
  if (status === 'running' || status === 'queued' || status === 'rejected') return 'warning'
  return 'default'
}

function replaceQuery(patch: Record<string, string | undefined>): void {
  const next: LocationQueryRaw = { ...route.query }
  for (const [key, value] of Object.entries(patch)) {
    if (value == null || value === '') delete next[key]
    else next[key] = value
  }
  void router.replace({ query: next })
}

function setRange(value: '24h' | '7d' | '30d'): void {
  replaceQuery({
    range: value === '24h' ? undefined : value,
    page: undefined,
  })
}

function setStatus(value: string): void {
  replaceQuery({
    status: value === 'all' ? undefined : value,
    page: undefined,
  })
}

function setKind(value: string): void {
  replaceQuery({
    kind: value === 'all' ? undefined : value,
    page: undefined,
  })
}

function applySearch(): void {
  const next = searchDraft.value.trim()
  replaceQuery({
    q: next || undefined,
    page: undefined,
  })
}

function clearFilters(): void {
  searchDraft.value = ''
  void router.replace({
    query: {
      ...(route.query.scope ? { scope: route.query.scope } : {}),
      ...(rangePreset.value !== '24h' ? { range: rangePreset.value } : {}),
    },
  })
}

function goToPage(nextPage: number): void {
  replaceQuery({
    page: nextPage > 1 ? String(nextPage) : undefined,
  })
}

function openRun(row: RunWorkspaceListItem): void {
  void router.push(
    buildRunDetailLocation(row.id, {
      fromScope: effectiveScope.value,
      fromJob: row.job_id,
      fromSection: 'history',
    }),
  )
}

function openJob(row: RunWorkspaceListItem): void {
  void router.push({
    path: `/jobs/${encodeURIComponent(row.job_id)}/history`,
    query: { scope: effectiveScope.value },
  })
}

async function refresh(): Promise<void> {
  loading.value = true
  try {
    const response = await runsStore.listWorkspace({
      scope: effectiveScope.value,
      status: selectedStatus.value === 'all' ? 'all' : (selectedStatus.value as never),
      kind: selectedKind.value === 'all' ? 'all' : (selectedKind.value as RunKind),
      range: rangePreset.value,
      q: selectedQuery.value || undefined,
      page: selectedPage.value,
      pageSize: pageSize.value,
    })
    rows.value = response.items
    total.value = response.total
    page.value = response.page
    pageSize.value = response.page_size
  } catch (error) {
    message.error(formatToastError(t('errors.fetchRunsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

watch(
  [effectiveScope, rangePreset, selectedStatus, selectedKind, selectedQuery, selectedPage],
  () => {
    void refresh()
  },
  { immediate: true },
)
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('runs.title')" :subtitle="t('runs.subtitle')">
      <div class="flex items-center gap-2 flex-wrap">
        <div class="console-range-picker">
          <n-button
            v-for="option in COMMAND_CENTER_RANGE_OPTIONS"
            :key="option.value"
            size="small"
            :type="rangePreset === option.value ? 'primary' : 'default'"
            :secondary="rangePreset !== option.value"
            @click="setRange(option.value)"
          >
            {{ t(option.labelKey) }}
          </n-button>
        </div>
        <n-button size="small" :loading="loading" @click="refresh">
          {{ t('common.refresh') }}
        </n-button>
      </div>
    </PageHeader>

    <ListPageScaffold>
      <template #toolbar>
        <ListToolbar embedded>
          <template #search>
            <n-input
              v-model:value="searchDraft"
              clearable
              :placeholder="t('runs.filters.search')"
              @keyup.enter="applySearch"
              @clear="applySearch"
            />
          </template>

          <template #filters>
            <n-select
              :value="selectedStatus"
              :options="statusOptions"
              :placeholder="t('runs.filters.status')"
              class="min-w-[10rem]"
              @update:value="setStatus"
            />
            <n-select
              :value="selectedKind"
              :options="kindOptions"
              :placeholder="t('runs.filters.kind')"
              class="min-w-[10rem]"
              @update:value="setKind"
            />
            <n-button size="small" type="primary" @click="applySearch">
              {{ t('common.search') }}
            </n-button>
          </template>

          <template #actions>
            <n-button size="small" quaternary @click="clearFilters">
              {{ t('common.clear') }}
            </n-button>
          </template>
        </ListToolbar>
      </template>

      <template #content>
        <n-card class="app-card console-panel" :bordered="false">
          <div class="flex items-center justify-between gap-3 flex-wrap">
            <div>
              <div class="console-kicker">{{ t('runs.list.kicker') }}</div>
              <div class="mt-1 text-base font-semibold">{{ t('runs.list.title') }}</div>
              <p class="mt-1 app-text-muted">
                {{ t('runs.list.subtitle', { scope: scopeLabel }) }}
              </p>
            </div>
            <div class="text-sm app-text-muted">
              {{ t('runs.list.summary', { shown: rows.length, total, page }) }}
            </div>
          </div>

          <AppEmptyState
            v-if="loading && rows.length === 0"
            class="mt-4"
            :title="t('common.loading')"
            loading
            variant="inset"
          />

          <AppEmptyState
            v-else-if="!loading && rows.length === 0"
            class="mt-4"
            :title="t('runs.list.emptyTitle')"
            :description="t('runs.list.emptyDescription')"
            variant="inset"
          />

          <div v-else class="mt-4 grid grid-cols-1 gap-3 xl:grid-cols-2">
            <article
              v-for="row in rows"
              :key="row.id"
              class="rounded-2xl app-border-subtle app-motion-soft p-4 md:p-5"
              :style="{ background: 'var(--app-surface-1)' }"
            >
              <div class="flex items-start justify-between gap-3 flex-wrap">
                <div class="min-w-0 space-y-2">
                  <div class="flex items-center gap-2 flex-wrap">
                    <button
                      type="button"
                      class="text-left text-base font-semibold hover:underline"
                      @click="openRun(row)"
                    >
                      {{ row.job_name }}
                    </button>
                    <n-tag size="small" :bordered="false" :type="statusTagType(row.status)">
                      {{ runStatusLabel(t, row.status) }}
                    </n-tag>
                    <n-tag size="small" :bordered="false">
                      {{ runKindLabel(t, row.kind) }}
                    </n-tag>
                  </div>

                  <div class="flex items-center gap-2 flex-wrap text-xs app-text-muted">
                    <span>{{ formatCommandCenterScopeLabel(String(row.scope), t) }}</span>
                    <span>·</span>
                    <span class="font-mono">{{ row.id }}</span>
                  </div>

                  <p class="text-sm">
                    {{ row.failure_title || row.error || t('runs.list.noFailureSummary') }}
                  </p>

                  <div class="grid grid-cols-1 gap-1 text-sm app-text-muted sm:grid-cols-2">
                    <div>{{ t('runs.columns.startedAt') }}: <span class="font-mono tabular-nums">{{ formatUnixSeconds(row.started_at) }}</span></div>
                    <div>{{ t('runs.columns.endedAt') }}: <span class="font-mono tabular-nums">{{ formatUnixSeconds(row.ended_at) }}</span></div>
                  </div>
                </div>

                <div class="flex items-center gap-2 flex-wrap justify-end">
                  <n-button size="small" type="primary" @click="openRun(row)">
                    {{ t('runs.actions.openRun') }}
                  </n-button>
                  <n-button size="small" quaternary @click="openJob(row)">
                    {{ t('runs.actions.openJob') }}
                  </n-button>
                </div>
              </div>
            </article>
          </div>
        </n-card>
      </template>

      <template #footer>
        <div v-if="total > pageSize" class="flex justify-end">
          <n-pagination
            :page="page"
            :page-size="pageSize"
            :item-count="total"
            @update:page="goToPage"
          />
        </div>
      </template>
    </ListPageScaffold>
  </div>
</template>
