<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import {
  NButton,
  NCard,
  NCheckbox,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NPopconfirm,
  NRadioButton,
  NRadioGroup,
  NSelect,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { useAgentsStore, type AgentDetail, type AgentListItem, type AgentsLabelsMode, type EnrollmentToken } from '@/stores/agents'
import { useBulkOperationsStore, type BulkSelectorRequest } from '@/stores/bulkOperations'
import { useUiStore } from '@/stores/ui'
import PageHeader from '@/components/PageHeader.vue'
import { MODAL_WIDTH } from '@/lib/modal'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { copyText } from '@/lib/clipboard'
import { formatToastError } from '@/lib/errors'
import AppEmptyState from '@/components/AppEmptyState.vue'

const { t } = useI18n()
const message = useMessage()
const router = useRouter()

const ui = useUiStore()
const agents = useAgentsStore()
const bulkOps = useBulkOperationsStore()
const isDesktop = useMediaQuery(MQ.mdUp)

const tokenModalOpen = ref<boolean>(false)
const tokenCreating = ref<boolean>(false)
const tokenResult = ref<EnrollmentToken | null>(null)
const ttlSeconds = ref<number>(60 * 60)
const remainingUses = ref<number | null>(null)

const rotateModalOpen = ref<boolean>(false)
const rotateRotating = ref<boolean>(false)
const rotateResult = ref<{ agent_id: string; agent_key: string } | null>(null)

const labelIndexLoading = ref<boolean>(false)
const labelIndex = ref<{ label: string; count: number }[]>([])
const selectedLabels = ref<string[]>([])
const labelsMode = ref<AgentsLabelsMode>('and')
const selectedAgentIds = ref<string[]>([])

const labelsModalOpen = ref<boolean>(false)
const labelsSaving = ref<boolean>(false)
const labelsAgent = ref<AgentListItem | null>(null)
const labelsValue = ref<string[]>([])

const bulkLabelsModalOpen = ref<boolean>(false)
const bulkLabelsSaving = ref<boolean>(false)
const bulkLabelsAction = ref<'agent_labels_add' | 'agent_labels_remove'>('agent_labels_add')
const bulkLabelsTarget = ref<'selected' | 'label_filter'>('selected')
const bulkLabelsValue = ref<string[]>([])

const bulkSyncModalOpen = ref<boolean>(false)
const bulkSyncSaving = ref<boolean>(false)
const bulkSyncTarget = ref<'selected' | 'label_filter'>('selected')

const detailModalOpen = ref<boolean>(false)
const detailLoading = ref<boolean>(false)
const detail = ref<AgentDetail | null>(null)

const syncNowLoading = ref<string | null>(null)

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

function shortId(value: string): string {
  if (value.length <= 12) return value
  return `${value.slice(0, 8)}…${value.slice(-4)}`
}

const labelOptions = computed(() =>
  labelIndex.value.map((it) => ({
    label: `${it.label} (${it.count})`,
    value: it.label,
  })),
)

async function refresh(): Promise<void> {
  try {
    await agents.refresh({ labels: selectedLabels.value, labelsMode: labelsMode.value })
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  }
}

async function refreshLabelIndex(): Promise<void> {
  labelIndexLoading.value = true
  try {
    labelIndex.value = await agents.listLabelIndex()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentLabelsFailed'), error, t))
  } finally {
    labelIndexLoading.value = false
  }
}

function openTokenModal(): void {
  tokenResult.value = null
  ttlSeconds.value = 60 * 60
  remainingUses.value = null
  tokenModalOpen.value = true
}

async function createToken(): Promise<void> {
  tokenCreating.value = true
  try {
    tokenResult.value = await agents.createEnrollmentToken({
      ttlSeconds: ttlSeconds.value,
      remainingUses: remainingUses.value,
    })
    message.success(t('messages.enrollmentTokenCreated'))
  } catch (error) {
    message.error(formatToastError(t('errors.createEnrollmentTokenFailed'), error, t))
  } finally {
    tokenCreating.value = false
  }
}

async function copyToClipboard(value: string): Promise<void> {
  const ok = await copyText(value)
  if (ok) {
    message.success(t('messages.copied'))
  } else {
    message.error(t('errors.copyFailed'))
  }
}

async function revokeAgent(agentId: string): Promise<void> {
  try {
    await agents.revokeAgent(agentId)
    await refresh()
    message.success(t('messages.agentRevoked'))
  } catch (error) {
    message.error(formatToastError(t('errors.revokeAgentFailed'), error, t))
  }
}

async function rotateAgentKey(agentId: string): Promise<void> {
  rotateRotating.value = true
  try {
    rotateResult.value = await agents.rotateAgentKey(agentId)
    rotateModalOpen.value = true
    message.success(t('messages.agentKeyRotated'))
  } catch (error) {
    message.error(formatToastError(t('errors.rotateAgentKeyFailed'), error, t))
  } finally {
    rotateRotating.value = false
  }
}

function openLabelsModal(agent: AgentListItem): void {
  labelsAgent.value = agent
  labelsValue.value = [...(agent.labels ?? [])]
  labelsModalOpen.value = true
}

function setAgentSelected(agentId: string, checked: boolean): void {
  const next = new Set(selectedAgentIds.value)
  if (checked) next.add(agentId)
  else next.delete(agentId)
  selectedAgentIds.value = [...next]
}

function openBulkLabelsModal(): void {
  bulkLabelsValue.value = []
  bulkLabelsAction.value = 'agent_labels_add'

  if (selectedAgentIds.value.length > 0) bulkLabelsTarget.value = 'selected'
  else bulkLabelsTarget.value = 'label_filter'

  bulkLabelsModalOpen.value = true
}

async function createBulkLabelsOperation(): Promise<void> {
  bulkLabelsSaving.value = true
  try {
    const labels = Array.from(
      new Set(bulkLabelsValue.value.map((v) => v.trim()).filter((v) => v.length > 0)),
    )
    if (labels.length === 0) {
      message.error(t('errors.formInvalid'))
      return
    }

    let selector: BulkSelectorRequest
    if (bulkLabelsTarget.value === 'selected') {
      const nodeIds = Array.from(new Set(selectedAgentIds.value))
      if (nodeIds.length === 0) {
        message.error(t('errors.formInvalid'))
        return
      }
      selector = { node_ids: nodeIds }
    } else {
      const labels = selectedLabels.value
      if (labels.length === 0) {
        message.error(t('errors.formInvalid'))
        return
      }
      selector = { labels, labels_mode: labelsMode.value }
    }

    const opId = await bulkOps.create({
      kind: bulkLabelsAction.value,
      selector,
      payload: { labels },
    })

    message.success(t('messages.bulkOperationCreated'))
    bulkLabelsModalOpen.value = false
    await router.push({ path: '/settings/bulk-operations', query: { open: opId } })
  } catch (error) {
    message.error(formatToastError(t('errors.createBulkOperationFailed'), error, t))
  } finally {
    bulkLabelsSaving.value = false
  }
}

function openBulkSyncModal(): void {
  if (selectedAgentIds.value.length > 0) bulkSyncTarget.value = 'selected'
  else bulkSyncTarget.value = 'label_filter'

  bulkSyncModalOpen.value = true
}

async function createBulkSyncOperation(): Promise<void> {
  bulkSyncSaving.value = true
  try {
    let selector: BulkSelectorRequest
    if (bulkSyncTarget.value === 'selected') {
      const nodeIds = Array.from(new Set(selectedAgentIds.value))
      if (nodeIds.length === 0) {
        message.error(t('errors.formInvalid'))
        return
      }
      selector = { node_ids: nodeIds }
    } else {
      const labels = selectedLabels.value
      if (labels.length === 0) {
        message.error(t('errors.formInvalid'))
        return
      }
      selector = { labels, labels_mode: labelsMode.value }
    }

    const opId = await bulkOps.create({
      kind: 'sync_config_now',
      selector,
    })

    message.success(t('messages.bulkOperationCreated'))
    bulkSyncModalOpen.value = false
    await router.push({ path: '/settings/bulk-operations', query: { open: opId } })
  } catch (error) {
    message.error(formatToastError(t('errors.createBulkOperationFailed'), error, t))
  } finally {
    bulkSyncSaving.value = false
  }
}

function configSyncStatusLabel(status: AgentListItem['config_sync_status']): string {
  return t(`agents.configSyncStatus.${status}`)
}

function configSyncStatusTagType(
  status: AgentListItem['config_sync_status'],
): 'default' | 'success' | 'warning' | 'error' {
  if (status === 'synced') return 'success'
  if (status === 'pending') return 'warning'
  if (status === 'error') return 'error'
  return 'default'
}

function configSyncTitle(row: AgentListItem): string {
  const desired = row.desired_config_snapshot_id ?? '-'
  const applied = row.applied_config_snapshot_id ?? '-'
  const err = row.last_config_sync_error ?? '-'
  return `desired: ${desired}\napplied: ${applied}\nerror: ${err}`
}

async function openAgentDetail(agentId: string): Promise<void> {
  detailModalOpen.value = true
  detailLoading.value = true
  detail.value = null
  try {
    detail.value = await agents.getAgent(agentId)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentFailed'), error, t))
  } finally {
    detailLoading.value = false
  }
}

function closeAgentDetail(): void {
  detailModalOpen.value = false
  detail.value = null
}

async function syncConfigNow(agentId: string): Promise<void> {
  syncNowLoading.value = agentId
  try {
    const res = await agents.syncConfigNow(agentId)
    if (res.outcome === 'pending_offline') {
      message.info(t('messages.syncConfigPendingOffline'))
    } else if (res.outcome === 'unchanged') {
      message.success(t('messages.syncConfigUnchanged'))
    } else {
      message.success(t('messages.syncConfigSent'))
    }
    await refresh()
    if (detail.value?.id === agentId) {
      detail.value = await agents.getAgent(agentId)
    }
  } catch (error) {
    message.error(formatToastError(t('errors.syncConfigNowFailed'), error, t))
  } finally {
    syncNowLoading.value = null
  }
}

async function saveAgentLabels(): Promise<void> {
  if (!labelsAgent.value) return

  labelsSaving.value = true
  try {
    await agents.setAgentLabels(labelsAgent.value.id, labelsValue.value)
    await refreshLabelIndex()
    await refresh()
    message.success(t('messages.agentLabelsUpdated'))
    labelsModalOpen.value = false
  } catch (error) {
    message.error(formatToastError(t('errors.updateAgentLabelsFailed'), error, t))
  } finally {
    labelsSaving.value = false
  }
}

const columns = computed<DataTableColumns<AgentListItem>>(() => [
  ...(isDesktop.value ? [{ type: 'selection' as const }] : []),
  {
    title: t('agents.columns.name'),
    key: 'name',
    render: (row) => row.name ?? '-',
  },
  {
    title: t('agents.columns.id'),
    key: 'id',
    render: (row) =>
      h('div', { class: 'flex items-center gap-2' }, [
        h('span', { class: 'font-mono text-xs' }, shortId(row.id)),
        h(
          NButton,
          { quaternary: true, size: 'small', onClick: () => copyToClipboard(row.id) },
          { default: () => t('agents.actions.copy') },
        ),
      ]),
  },
  {
    title: t('agents.columns.labels'),
    key: 'labels',
    render: (row) => {
      if (!row.labels?.length) return '-'
      return h(
        'div',
        { class: 'flex flex-wrap gap-1' },
        row.labels.map((label) => h(NTag, { size: 'small' }, { default: () => label })),
      )
    },
  },
  {
    title: t('agents.columns.status'),
    key: 'status',
    render: (row) => {
      const conn = row.revoked
        ? h(NTag, { type: 'error', size: 'small' }, { default: () => t('agents.status.revoked') })
        : row.online
          ? h(NTag, { type: 'success', size: 'small' }, { default: () => t('agents.status.online') })
          : h(NTag, { size: 'small' }, { default: () => t('agents.status.offline') })

      const cfg = h(
        NTag,
        {
          type: configSyncStatusTagType(row.config_sync_status),
          size: 'small',
          title: configSyncTitle(row),
        },
        { default: () => configSyncStatusLabel(row.config_sync_status) },
      )

      return h('div', { class: 'flex flex-wrap gap-1' }, [conn, cfg])
    },
  },
  {
    title: t('agents.columns.lastSeen'),
    key: 'last_seen_at',
    render: (row) => formatUnixSeconds(row.last_seen_at),
  },
  {
    title: t('agents.columns.actions'),
    key: 'actions',
    render: (row) =>
      h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NButton,
              { tertiary: true, size: 'small', onClick: () => openAgentDetail(row.id) },
              { default: () => t('agents.actions.details') },
            ),
            h(
              NButton,
              {
                tertiary: true,
                size: 'small',
                loading: syncNowLoading.value === row.id,
                disabled: row.revoked,
                onClick: () => syncConfigNow(row.id),
              },
              { default: () => t('agents.actions.syncNow') },
            ),
            h(
              NButton,
              { tertiary: true, size: 'small', onClick: () => openLabelsModal(row) },
              { default: () => t('agents.actions.labels') },
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => rotateAgentKey(row.id),
                positiveText: t('agents.actions.rotateKey'),
                negativeText: t('common.cancel'),
                disabled: row.revoked,
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    {
                      tertiary: true,
                      size: 'small',
                      type: 'warning',
                      loading: rotateRotating.value,
                      disabled: row.revoked,
                    },
                    { default: () => t('agents.actions.rotateKey') },
                  ),
                default: () => t('agents.rotateConfirm'),
              },
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => revokeAgent(row.id),
                positiveText: t('agents.actions.revoke'),
                negativeText: t('common.cancel'),
                disabled: row.revoked,
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { tertiary: true, size: 'small', type: 'error', disabled: row.revoked },
                    { default: () => t('agents.actions.revoke') },
                  ),
                default: () => t('agents.revokeConfirm'),
              },
            ),
          ],
        },
      ),
  },
])

watch([selectedLabels, labelsMode], refresh, { deep: true })
watch(detailModalOpen, (open) => {
  if (open) return
  detailLoading.value = false
  detail.value = null
})

onMounted(async () => {
  await refreshLabelIndex()
  await refresh()
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('agents.title')" :subtitle="t('agents.subtitle')">
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button
        :disabled="selectedAgentIds.length === 0 && selectedLabels.length === 0"
        @click="openBulkLabelsModal"
      >
        {{ t('agents.bulkLabels') }}
      </n-button>
      <n-button
        :disabled="selectedAgentIds.length === 0 && selectedLabels.length === 0"
        @click="openBulkSyncModal"
      >
        {{ t('agents.bulkSync') }}
      </n-button>
      <n-button type="primary" @click="openTokenModal">{{ t('agents.newToken') }}</n-button>
    </PageHeader>

    <n-card class="app-card">
      <div class="flex flex-col gap-3 md:flex-row md:items-end">
        <div class="flex-1 min-w-0">
          <div class="text-sm opacity-70 mb-1">{{ t('agents.filters.labels') }}</div>
          <n-select
            v-model:value="selectedLabels"
            multiple
            filterable
            clearable
            :loading="labelIndexLoading"
            :options="labelOptions"
            :placeholder="t('agents.filters.labelsPlaceholder')"
          />
        </div>
        <div class="shrink-0">
          <div class="text-sm opacity-70 mb-1">{{ t('agents.filters.mode') }}</div>
          <n-radio-group v-model:value="labelsMode" size="small">
            <n-radio-button value="and">{{ t('common.and') }}</n-radio-button>
            <n-radio-button value="or">{{ t('common.or') }}</n-radio-button>
          </n-radio-group>
        </div>
        <div class="shrink-0 flex justify-end">
          <n-button @click="selectedLabels = []">{{ t('common.clear') }}</n-button>
        </div>
      </div>
    </n-card>

    <div v-if="!isDesktop" class="space-y-3">
      <AppEmptyState v-if="agents.loading && agents.items.length === 0" :title="t('common.loading')" loading />
      <AppEmptyState v-else-if="!agents.loading && agents.items.length === 0" :title="t('common.noData')" />

      <n-card
        v-for="agent in agents.items"
        :key="agent.id"
        size="small"
        class="app-card"
      >
        <template #header>
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-2 min-w-0">
              <n-checkbox
                :checked="selectedAgentIds.includes(agent.id)"
                @update:checked="(v) => setAgentSelected(agent.id, v)"
              />
              <div class="font-medium truncate">{{ agent.name ?? '-' }}</div>
            </div>
            <div class="flex flex-wrap justify-end gap-1">
              <n-tag v-if="agent.revoked" type="error" size="small">{{ t('agents.status.revoked') }}</n-tag>
              <n-tag v-else-if="agent.online" type="success" size="small">{{ t('agents.status.online') }}</n-tag>
              <n-tag v-else size="small">{{ t('agents.status.offline') }}</n-tag>
              <n-tag
                :type="configSyncStatusTagType(agent.config_sync_status)"
                size="small"
                :title="configSyncTitle(agent)"
              >
                {{ configSyncStatusLabel(agent.config_sync_status) }}
              </n-tag>
            </div>
          </div>
        </template>

        <div class="text-sm space-y-2">
          <div v-if="agent.labels?.length" class="flex flex-wrap gap-1">
            <n-tag v-for="label in agent.labels" :key="label" size="small">{{ label }}</n-tag>
          </div>
          <div class="flex items-center justify-between gap-3">
            <div class="opacity-70">{{ t('agents.columns.id') }}</div>
            <div class="flex items-center gap-2">
              <span class="font-mono text-xs">{{ shortId(agent.id) }}</span>
              <n-button quaternary size="small" @click="copyToClipboard(agent.id)">{{ t('agents.actions.copy') }}</n-button>
            </div>
          </div>
          <div class="flex items-center justify-between gap-3">
            <div class="opacity-70">{{ t('agents.columns.lastSeen') }}</div>
            <div class="text-right">{{ formatUnixSeconds(agent.last_seen_at) }}</div>
          </div>
        </div>

        <template #footer>
          <div class="flex flex-wrap justify-end gap-2">
            <n-button size="small" tertiary @click="openAgentDetail(agent.id)">{{ t('agents.actions.details') }}</n-button>
            <n-button
              size="small"
              tertiary
              :loading="syncNowLoading === agent.id"
              :disabled="agent.revoked"
              @click="syncConfigNow(agent.id)"
            >
              {{ t('agents.actions.syncNow') }}
            </n-button>
            <n-button size="small" tertiary @click="openLabelsModal(agent)">{{ t('agents.actions.labels') }}</n-button>

            <n-popconfirm
              :positive-text="t('agents.actions.rotateKey')"
              :negative-text="t('common.cancel')"
              :disabled="agent.revoked"
              @positive-click="rotateAgentKey(agent.id)"
            >
              <template #trigger>
                <n-button size="small" type="warning" tertiary :loading="rotateRotating" :disabled="agent.revoked">
                  {{ t('agents.actions.rotateKey') }}
                </n-button>
              </template>
              {{ t('agents.rotateConfirm') }}
            </n-popconfirm>

            <n-popconfirm
              :positive-text="t('agents.actions.revoke')"
              :negative-text="t('common.cancel')"
              :disabled="agent.revoked"
              @positive-click="revokeAgent(agent.id)"
            >
              <template #trigger>
                <n-button size="small" type="error" tertiary :disabled="agent.revoked">
                  {{ t('agents.actions.revoke') }}
                </n-button>
              </template>
              {{ t('agents.revokeConfirm') }}
            </n-popconfirm>
          </div>
        </template>
      </n-card>
    </div>

    <div v-else>
      <n-card class="app-card">
        <div class="overflow-x-auto">
          <n-data-table
            v-model:checked-row-keys="selectedAgentIds"
            :row-key="(row) => row.id"
            :loading="agents.loading"
            :columns="columns"
            :data="agents.items"
          />
        </div>
      </n-card>
    </div>

    <n-modal v-model:show="tokenModalOpen" preset="card" :style="{ width: MODAL_WIDTH.md }" :title="t('agents.tokenModal.title')">
      <div class="space-y-4">
        <n-form label-placement="top">
          <n-form-item :label="t('agents.tokenModal.ttl')">
            <n-input-number v-model:value="ttlSeconds" :min="60" class="w-full" />
          </n-form-item>
          <n-form-item :label="t('agents.tokenModal.remainingUses')">
            <n-input-number v-model:value="remainingUses" :min="1" clearable class="w-full" />
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="tokenModalOpen = false">{{ t('common.close') }}</n-button>
          <n-button type="primary" :loading="tokenCreating" @click="createToken">
            {{ t('agents.tokenModal.create') }}
          </n-button>
        </n-space>

        <div v-if="tokenResult" class="space-y-2">
          <div class="text-sm opacity-70">{{ t('agents.tokenModal.help') }}</div>

          <n-form label-placement="top">
            <n-form-item :label="t('agents.tokenModal.token')">
              <n-input :value="tokenResult.token" readonly />
            </n-form-item>
            <n-form-item :label="t('agents.tokenModal.expiresAt')">
              <n-input :value="formatUnixSeconds(tokenResult.expires_at)" readonly />
            </n-form-item>
          </n-form>

          <n-space>
            <n-button @click="copyToClipboard(tokenResult.token)">{{ t('agents.actions.copyToken') }}</n-button>
          </n-space>
        </div>
      </div>
    </n-modal>

    <n-modal v-model:show="rotateModalOpen" preset="card" :style="{ width: MODAL_WIDTH.md }" :title="t('agents.rotateModal.title')">
      <div class="space-y-4">
        <div class="text-sm opacity-70">{{ t('agents.rotateModal.help') }}</div>

        <n-form v-if="rotateResult" label-placement="top">
          <n-form-item :label="t('agents.rotateModal.agentKey')">
            <n-input :value="rotateResult.agent_key" readonly />
          </n-form-item>
        </n-form>

        <n-space v-if="rotateResult">
          <n-button @click="copyToClipboard(rotateResult.agent_key)">{{ t('agents.actions.copyKey') }}</n-button>
        </n-space>

        <n-space justify="end">
          <n-button @click="rotateModalOpen = false">{{ t('common.close') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal
      v-model:show="detailModalOpen"
      preset="card"
      :style="{ width: MODAL_WIDTH.md }"
      :title="t('agents.detailModal.title')"
    >
      <div class="space-y-4">
        <AppEmptyState v-if="detailLoading" :title="t('common.loading')" loading />
        <AppEmptyState v-else-if="!detail" :title="t('common.noData')" />

        <div v-else class="space-y-4">
          <div class="flex flex-wrap gap-1">
            <n-tag v-if="detail.revoked" type="error" size="small">{{ t('agents.status.revoked') }}</n-tag>
            <n-tag v-else-if="detail.online" type="success" size="small">{{ t('agents.status.online') }}</n-tag>
            <n-tag v-else size="small">{{ t('agents.status.offline') }}</n-tag>
            <n-tag :type="configSyncStatusTagType(detail.config_sync_status)" size="small">
              {{ configSyncStatusLabel(detail.config_sync_status) }}
            </n-tag>
          </div>

          <n-form label-placement="top">
            <n-form-item :label="t('agents.detailModal.id')">
              <n-input :value="detail.id" readonly />
            </n-form-item>
            <n-form-item :label="t('agents.detailModal.name')">
              <n-input :value="detail.name ?? '-'" readonly />
            </n-form-item>
            <n-form-item :label="t('agents.detailModal.lastSeen')">
              <n-input :value="formatUnixSeconds(detail.last_seen_at)" readonly />
            </n-form-item>
          </n-form>

          <n-card size="small" class="app-card" :title="t('agents.detailModal.configSyncTitle')">
            <n-form label-placement="top">
              <n-form-item :label="t('agents.detailModal.desiredSnapshot')">
                <n-input :value="detail.desired_config_snapshot_id ?? '-'" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.desiredAt')">
                <n-input :value="formatUnixSeconds(detail.desired_config_snapshot_at)" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.appliedSnapshot')">
                <n-input :value="detail.applied_config_snapshot_id ?? '-'" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.appliedAt')">
                <n-input :value="formatUnixSeconds(detail.applied_config_snapshot_at)" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.lastAttemptAt')">
                <n-input :value="formatUnixSeconds(detail.last_config_sync_attempt_at)" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.lastErrorKind')">
                <n-input :value="detail.last_config_sync_error_kind ?? '-'" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.lastError')">
                <n-input
                  :value="detail.last_config_sync_error ?? '-'"
                  readonly
                  type="textarea"
                  :autosize="{ minRows: 2, maxRows: 6 }"
                />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.lastErrorAt')">
                <n-input :value="formatUnixSeconds(detail.last_config_sync_error_at)" readonly />
              </n-form-item>
            </n-form>
          </n-card>
        </div>

        <n-space justify="end">
          <n-button @click="closeAgentDetail">{{ t('common.close') }}</n-button>
          <n-button
            type="primary"
            :loading="syncNowLoading === detail?.id"
            :disabled="detail?.revoked ?? true"
            @click="detail?.id && syncConfigNow(detail.id)"
          >
            {{ t('agents.actions.syncNow') }}
          </n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal v-model:show="labelsModalOpen" preset="card" :style="{ width: MODAL_WIDTH.md }" :title="t('agents.labelsModal.title')">
      <div class="space-y-4">
        <div class="text-sm opacity-70">{{ t('agents.labelsModal.help') }}</div>
        <div v-if="labelsAgent" class="text-sm">
          <span class="opacity-70">{{ t('agents.columns.id') }}:</span>
          <span class="font-mono ml-2">{{ labelsAgent.id }}</span>
        </div>

        <n-form label-placement="top">
          <n-form-item :label="t('agents.labelsModal.labels')">
            <n-select
              v-model:value="labelsValue"
              multiple
              filterable
              tag
              clearable
              :options="labelOptions"
              :placeholder="t('agents.labelsModal.placeholder')"
            />
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="labelsModalOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="labelsSaving" @click="saveAgentLabels">{{ t('common.save') }}</n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal
      v-model:show="bulkSyncModalOpen"
      preset="card"
      :style="{ width: MODAL_WIDTH.md }"
      :title="t('agents.bulkSyncModal.title')"
    >
      <div class="space-y-4">
        <div class="text-sm opacity-70">{{ t('agents.bulkSyncModal.help') }}</div>

        <n-form label-placement="top">
          <n-form-item :label="t('agents.bulkSyncModal.target')">
            <n-radio-group v-model:value="bulkSyncTarget" size="small">
              <n-radio-button value="selected" :disabled="selectedAgentIds.length === 0">
                {{ t('agents.bulkSyncModal.targetSelected', { count: selectedAgentIds.length }) }}
              </n-radio-button>
              <n-radio-button value="label_filter" :disabled="selectedLabels.length === 0">
                {{ t('agents.bulkSyncModal.targetLabelFilter') }}
              </n-radio-button>
            </n-radio-group>
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="bulkSyncModalOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="bulkSyncSaving" @click="createBulkSyncOperation">
            {{ t('common.apply') }}
          </n-button>
        </n-space>
      </div>
    </n-modal>

    <n-modal
      v-model:show="bulkLabelsModalOpen"
      preset="card"
      :style="{ width: MODAL_WIDTH.md }"
      :title="t('agents.bulkLabelsModal.title')"
    >
      <div class="space-y-4">
        <div class="text-sm opacity-70">{{ t('agents.bulkLabelsModal.help') }}</div>

        <n-form label-placement="top">
          <n-form-item :label="t('agents.bulkLabelsModal.target')">
            <n-radio-group v-model:value="bulkLabelsTarget" size="small">
              <n-radio-button value="selected" :disabled="selectedAgentIds.length === 0">
                {{ t('agents.bulkLabelsModal.targetSelected', { count: selectedAgentIds.length }) }}
              </n-radio-button>
              <n-radio-button value="label_filter" :disabled="selectedLabels.length === 0">
                {{ t('agents.bulkLabelsModal.targetLabelFilter') }}
              </n-radio-button>
            </n-radio-group>
          </n-form-item>

          <n-form-item :label="t('agents.bulkLabelsModal.action')">
            <n-radio-group v-model:value="bulkLabelsAction" size="small">
              <n-radio-button value="agent_labels_add">{{ t('agents.bulkLabelsModal.actionAdd') }}</n-radio-button>
              <n-radio-button value="agent_labels_remove">{{ t('agents.bulkLabelsModal.actionRemove') }}</n-radio-button>
            </n-radio-group>
          </n-form-item>

          <n-form-item :label="t('agents.bulkLabelsModal.labels')">
            <n-select
              v-model:value="bulkLabelsValue"
              multiple
              filterable
              tag
              clearable
              :options="labelOptions"
              :placeholder="t('agents.bulkLabelsModal.labelsPlaceholder')"
            />
          </n-form-item>
        </n-form>

        <n-space justify="end">
          <n-button @click="bulkLabelsModalOpen = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" :loading="bulkLabelsSaving" @click="createBulkLabelsOperation">
            {{ t('common.apply') }}
          </n-button>
        </n-space>
      </div>
    </n-modal>
  </div>
</template>
