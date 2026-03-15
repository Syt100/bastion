<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NForm, NFormItem, NInput, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import AppModalShell from '@/components/AppModalShell.vue'
import PageHeader from '@/components/PageHeader.vue'
import { MODAL_WIDTH } from '@/lib/modal'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { createClipboardCopyAction } from '@/lib/clipboardFeedback'
import { formatToastError } from '@/lib/errors'
import { buildJobsCollectionLocation } from '@/lib/jobsRoute'
import { scopeFromNodeId } from '@/lib/scope'
import { useAgentsStore } from '@/stores/agents'
import { useFleetStore, type FleetAgentDetailResponse } from '@/stores/fleet'
import { useUiStore } from '@/stores/ui'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const message = useMessage()

const ui = useUiStore()
const agents = useAgentsStore()
const fleet = useFleetStore()
const copyWithFeedback = createClipboardCopyAction(t, message)

const agentId = computed(() => (typeof route.params.agentId === 'string' ? route.params.agentId : ''))
const loading = ref<boolean>(false)
const detail = ref<FleetAgentDetailResponse | null>(null)
const rotateLoading = ref<boolean>(false)
const revokeLoading = ref<boolean>(false)
const syncLoading = ref<boolean>(false)
const rotateModalOpen = ref<boolean>(false)
const rotateKey = ref<string>('')

const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const statusTagType = computed<'success' | 'default' | 'error'>(() => {
  if (detail.value?.agent.status === 'online') return 'success'
  if (detail.value?.agent.status === 'revoked') return 'error'
  return 'default'
})

const syncTagType = computed<'success' | 'default' | 'warning' | 'error'>(() => {
  const state = detail.value?.sync.state
  if (state === 'synced') return 'success'
  if (state === 'error') return 'error'
  if (state === 'pending') return 'warning'
  return 'default'
})

function statusLabel(status: 'online' | 'offline' | 'revoked'): string {
  if (status === 'online') return t('agents.status.online')
  if (status === 'revoked') return t('agents.status.revoked')
  return t('agents.status.offline')
}

function syncStateLabel(state: 'synced' | 'pending' | 'error' | 'offline'): string {
  return t(`agents.configSyncStatus.${state}`)
}

async function refresh(): Promise<void> {
  if (!agentId.value) return
  loading.value = true
  try {
    detail.value = await fleet.get(agentId.value)
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentFailed'), error, t))
    detail.value = null
  } finally {
    loading.value = false
  }
}

function openJobs(): void {
  if (!detail.value) return
  void router.push(buildJobsCollectionLocation({ scope: scopeFromNodeId(detail.value.agent.id) }))
}

function openStorage(): void {
  if (!detail.value) return
  void router.push({
    path: '/integrations/storage',
    query: {
      scope: scopeFromNodeId(detail.value.agent.id),
    },
  })
}

async function syncNow(): Promise<void> {
  if (!detail.value?.capabilities.can_sync_now) return
  syncLoading.value = true
  try {
    const res = await agents.syncConfigNow(detail.value.agent.id)
    if (res.outcome === 'pending_offline') {
      message.info(t('messages.syncConfigPendingOffline'))
    } else if (res.outcome === 'unchanged') {
      message.success(t('messages.syncConfigUnchanged'))
    } else {
      message.success(t('messages.syncConfigSent'))
    }
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.syncConfigNowFailed'), error, t))
  } finally {
    syncLoading.value = false
  }
}

async function rotateKeyNow(): Promise<void> {
  if (!detail.value?.capabilities.can_rotate_key) return
  rotateLoading.value = true
  try {
    const res = await agents.rotateAgentKey(detail.value.agent.id)
    rotateKey.value = res.agent_key
    rotateModalOpen.value = true
    message.success(t('messages.agentKeyRotated'))
  } catch (error) {
    message.error(formatToastError(t('errors.rotateAgentKeyFailed'), error, t))
  } finally {
    rotateLoading.value = false
  }
}

async function revoke(): Promise<void> {
  if (!detail.value?.capabilities.can_revoke) return
  revokeLoading.value = true
  try {
    await agents.revokeAgent(detail.value.agent.id)
    message.success(t('messages.agentRevoked'))
    await refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.revokeAgentFailed'), error, t))
  } finally {
    revokeLoading.value = false
  }
}

async function copy(value: string): Promise<void> {
  await copyWithFeedback(value)
}

onMounted(() => {
  void refresh()
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      :title="detail?.agent.name || detail?.agent.id || t('fleet.detail.title')"
      :subtitle="t('fleet.detail.subtitle')"
    >
      <n-button @click="refresh">{{ t('common.refresh') }}</n-button>
      <n-button @click="openJobs">{{ t('agents.actions.jobs') }}</n-button>
      <n-button @click="openStorage">{{ t('agents.actions.storage') }}</n-button>
      <n-button
        :loading="syncLoading"
        :disabled="!detail?.capabilities.can_sync_now"
        type="primary"
        @click="syncNow"
      >
        {{ t('agents.actions.syncNow') }}
      </n-button>
    </PageHeader>

    <AppEmptyState v-if="loading && !detail" :title="t('common.loading')" loading />
    <AppEmptyState v-else-if="!detail" :title="t('common.noData')" />

    <template v-else>
      <div class="grid gap-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(0,0.85fr)]">
        <n-card class="app-card" :bordered="false">
          <div class="flex flex-wrap items-center gap-2">
            <n-tag :type="statusTagType" size="small">{{ statusLabel(detail.agent.status) }}</n-tag>
            <n-tag :type="syncTagType" size="small">{{ syncStateLabel(detail.sync.state) }}</n-tag>
          </div>

          <n-form label-placement="top" class="mt-4">
            <div class="grid gap-4 md:grid-cols-2">
              <n-form-item :label="t('agents.detailModal.id')">
                <n-input :value="detail.agent.id" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.name')">
                <n-input :value="detail.agent.name ?? '-'" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.lastSeen')">
                <n-input :value="formatUnixSeconds(detail.agent.last_seen_at ?? null)" readonly />
              </n-form-item>
              <n-form-item :label="t('fleet.detail.createdAt')">
                <n-input :value="formatUnixSeconds(detail.agent.created_at)" readonly />
              </n-form-item>
            </div>
          </n-form>

          <div class="mt-4">
            <div class="app-meta-text">{{ t('agents.columns.labels') }}</div>
            <div class="mt-2 flex flex-wrap gap-1">
              <n-tag v-for="label in detail.agent.labels" :key="label" size="small">{{ label }}</n-tag>
              <span v-if="detail.agent.labels.length === 0" class="app-meta-text">-</span>
            </div>
          </div>
        </n-card>

        <n-card class="app-card" :bordered="false">
          <div class="text-sm font-semibold">{{ t('fleet.detail.actionsTitle') }}</div>
          <div class="app-meta-text mt-1">{{ t('fleet.detail.actionsSubtitle') }}</div>
          <div class="mt-4 flex flex-wrap gap-2">
            <n-button
              :loading="rotateLoading"
              :disabled="!detail.capabilities.can_rotate_key"
              @click="rotateKeyNow"
            >
              {{ t('agents.actions.rotateKey') }}
            </n-button>
            <n-button
              type="error"
              ghost
              :loading="revokeLoading"
              :disabled="!detail.capabilities.can_revoke"
              @click="revoke"
            >
              {{ t('agents.actions.revoke') }}
            </n-button>
          </div>
        </n-card>
      </div>

      <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <n-card class="app-card" :bordered="false" :title="t('fleet.detail.syncTitle')">
          <n-form label-placement="top">
            <div class="grid gap-4 md:grid-cols-2">
              <n-form-item :label="t('agents.detailModal.desiredSnapshot')">
                <n-input :value="detail.sync.desired_snapshot_id ?? '-'" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.appliedSnapshot')">
                <n-input :value="detail.sync.applied_snapshot_id ?? '-'" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.desiredAt')">
                <n-input :value="formatUnixSeconds(detail.sync.desired_snapshot_at ?? null)" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.appliedAt')">
                <n-input :value="formatUnixSeconds(detail.sync.applied_snapshot_at ?? null)" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.lastAttemptAt')">
                <n-input :value="formatUnixSeconds(detail.sync.last_attempt_at ?? null)" readonly />
              </n-form-item>
              <n-form-item :label="t('agents.detailModal.lastErrorKind')">
                <n-input :value="detail.sync.last_error_kind ?? '-'" readonly />
              </n-form-item>
            </div>
            <n-form-item :label="t('agents.detailModal.lastError')">
              <n-input
                :value="detail.sync.last_error ?? '-'"
                readonly
                type="textarea"
                :autosize="{ minRows: 2, maxRows: 6 }"
              />
            </n-form-item>
          </n-form>
        </n-card>

        <n-card class="app-card" :bordered="false" :title="t('fleet.detail.relatedJobsTitle')">
          <div v-if="detail.related_jobs.length" class="app-divide-y">
            <button
              v-for="job in detail.related_jobs"
              :key="job.id"
              type="button"
              class="app-list-row app-motion-soft text-left"
              @click="router.push(`/jobs/${encodeURIComponent(job.id)}/overview`)"
            >
              <div class="min-w-0">
                <div class="font-medium truncate">{{ job.name }}</div>
                <div class="app-meta-text mt-1">
                  {{ t('fleet.detail.jobMeta', { schedule: job.schedule || '-', updatedAt: formatUnixSeconds(job.updated_at) }) }}
                </div>
              </div>
            </button>
          </div>
          <AppEmptyState
            v-else
            :title="t('fleet.detail.relatedJobsEmptyTitle')"
            :description="t('fleet.detail.relatedJobsEmptyDescription')"
          />
        </n-card>
      </div>

      <n-card class="app-card" :bordered="false" :title="t('fleet.detail.activityTitle')">
        <div v-if="detail.recent_activity.length" class="app-divide-y">
          <div v-for="item in detail.recent_activity" :key="item.run_id" class="app-list-row">
            <div class="min-w-0">
              <div class="font-medium truncate">{{ item.job_name }}</div>
              <div class="app-meta-text mt-1">
                {{ t('fleet.detail.activityMeta', { status: item.status, startedAt: formatUnixSeconds(item.started_at ?? null), endedAt: formatUnixSeconds(item.ended_at ?? null) }) }}
              </div>
            </div>
            <div class="flex gap-2">
              <n-button size="small" @click="copy(item.run_id)">{{ t('agents.actions.copy') }}</n-button>
              <n-button size="small" tertiary @click="router.push(`/runs/${encodeURIComponent(item.run_id)}`)">
                {{ t('fleet.detail.openRun') }}
              </n-button>
            </div>
          </div>
        </div>
        <AppEmptyState
          v-else
          :title="t('fleet.detail.activityEmptyTitle')"
          :description="t('fleet.detail.activityEmptyDescription')"
        />
      </n-card>
    </template>

    <AppModalShell
      v-model:show="rotateModalOpen"
      :width="MODAL_WIDTH.md"
      :title="t('agents.rotateModal.title')"
    >
      <div class="text-sm app-text-muted">{{ t('agents.rotateModal.help') }}</div>
      <n-form label-placement="top" class="mt-4">
        <n-form-item :label="t('agents.rotateModal.agentKey')">
          <n-input :value="rotateKey" readonly />
        </n-form-item>
      </n-form>

      <template #footer>
        <n-button @click="rotateModalOpen = false">{{ t('common.close') }}</n-button>
        <n-button type="primary" @click="copy(rotateKey)">{{ t('agents.actions.copyKey') }}</n-button>
      </template>
    </AppModalShell>
  </div>
</template>
