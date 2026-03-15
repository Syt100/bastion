<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import { formatToastError } from '@/lib/errors'
import { scopeFromNodeId } from '@/lib/scope'
import { useUnixSecondsFormatter } from '@/lib/datetime'
import { useAgentsStore } from '@/stores/agents'
import { useUiStore } from '@/stores/ui'
import { useIntegrationsStore, type DistributionDetailsResponse } from '@/stores/integrations'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()
const agents = useAgentsStore()
const integrations = useIntegrationsStore()
const ui = useUiStore()
const { formatUnixSeconds } = useUnixSecondsFormatter(computed(() => ui.locale))

const loading = ref<boolean>(false)
const syncNowLoading = ref<string | null>(null)
const distribution = ref<DistributionDetailsResponse | null>(null)

async function refresh(): Promise<void> {
  loading.value = true
  try {
    distribution.value = await integrations.getDistribution()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

function distributionTagType(state: 'covered' | 'drifted' | 'failed'): 'success' | 'warning' | 'error' {
  if (state === 'covered') return 'success'
  if (state === 'failed') return 'error'
  return 'warning'
}

async function syncNow(agentId: string): Promise<void> {
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
  } catch (error) {
    message.error(formatToastError(t('errors.syncConfigNowFailed'), error, t))
  } finally {
    syncNowLoading.value = null
  }
}

onMounted(() => {
  void refresh()
})
</script>

<template>
  <AppEmptyState v-if="loading && !distribution" :title="t('common.loading')" loading />

  <div v-else class="space-y-4">
    <n-card class="app-card" :bordered="false">
      <div class="flex items-start justify-between gap-3">
        <div>
          <div class="text-sm font-semibold">{{ t('integrations.distribution.title') }}</div>
          <div class="app-meta-text mt-1">{{ t('integrations.distribution.subtitle') }}</div>
        </div>
        <n-tag
          v-if="distribution"
          :type="distribution.summary.drifted_total > 0 || distribution.summary.failed_total > 0 ? 'warning' : distribution.summary.coverage_total > 0 ? 'success' : 'default'"
          size="small"
        >
          {{
            distribution.summary.coverage_total === 0
              ? t('integrations.states.empty')
              : distribution.summary.drifted_total > 0 || distribution.summary.failed_total > 0
                ? t('integrations.states.degraded')
                : t('integrations.states.ready')
          }}
        </n-tag>
      </div>

      <div v-if="distribution" class="mt-4 grid gap-3 md:grid-cols-4">
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('integrations.distribution.coverage') }}</div>
          <div class="mt-2 text-2xl font-semibold">{{ distribution.summary.coverage_total }}</div>
        </div>
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('integrations.distribution.drifted') }}</div>
          <div class="mt-2 text-2xl font-semibold">{{ distribution.summary.drifted_total }}</div>
        </div>
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('integrations.distribution.failed') }}</div>
          <div class="mt-2 text-2xl font-semibold">{{ distribution.summary.failed_total }}</div>
        </div>
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('integrations.distribution.offline') }}</div>
          <div class="mt-2 text-2xl font-semibold">{{ distribution.summary.offline_total }}</div>
        </div>
      </div>

      <div class="mt-4 flex flex-wrap gap-2">
        <n-button @click="router.push('/fleet')">{{ t('integrations.distribution.openFleet') }}</n-button>
        <n-button @click="router.push('/system/bulk-operations')">{{ t('integrations.distribution.openBulkOperations') }}</n-button>
      </div>
    </n-card>

    <n-card v-if="distribution" class="app-card" :bordered="false">
      <div class="flex items-start justify-between gap-3">
        <div>
          <div class="text-sm font-semibold">{{ t('integrations.distribution.scopeTitle') }}</div>
          <div class="app-meta-text mt-1">{{ t('integrations.distribution.scopeSubtitle') }}</div>
        </div>
      </div>

      <div v-if="distribution.items.length === 0" class="mt-4 app-help-text">
        {{ t('common.noData') }}
      </div>

      <div v-else class="mt-4 space-y-3">
        <div
          v-for="item in distribution.items"
          :key="item.agent_id"
          class="rounded-2xl app-panel-inset px-4 py-3"
        >
          <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
            <div class="min-w-0 space-y-2">
              <div class="flex flex-wrap items-center gap-2">
                <div class="font-medium">{{ item.agent_name || item.agent_id }}</div>
                <n-tag size="small" :type="distributionTagType(item.distribution_state)">
                  {{ t(`integrations.distribution.state.${item.distribution_state}`) }}
                </n-tag>
                <n-tag size="small" :type="item.connection_status === 'online' ? 'success' : 'default'">
                  {{ t(`integrations.distribution.connection.${item.connection_status}`) }}
                </n-tag>
              </div>
              <div class="text-sm app-text-muted">
                {{
                  t('integrations.distribution.scopeMeta', {
                    pending: item.pending_tasks_total,
                    attemptedAt: formatUnixSeconds(item.last_attempt_at ?? null),
                  })
                }}
              </div>
              <div v-if="item.last_error" class="text-sm">
                {{ item.last_error }}
              </div>
            </div>

            <div class="flex flex-wrap gap-2">
              <n-button size="small" @click="router.push(`/fleet/${encodeURIComponent(item.agent_id)}`)">
                {{ t('integrations.distribution.openAgent') }}
              </n-button>
              <n-button
                size="small"
                :loading="syncNowLoading === item.agent_id"
                @click="syncNow(item.agent_id)"
              >
                {{ t('agents.actions.syncNow') }}
              </n-button>
              <n-button
                size="small"
                @click="
                  router.push({
                    path: '/integrations/storage',
                    query: { scope: scopeFromNodeId(item.agent_id) },
                  })
                "
              >
                {{ t('integrations.distribution.openStorage') }}
              </n-button>
            </div>
          </div>
        </div>
      </div>
    </n-card>
  </div>
</template>
