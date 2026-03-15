<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCard, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import { formatToastError } from '@/lib/errors'
import { useIntegrationsStore, type IntegrationsSummaryResponse } from '@/stores/integrations'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()
const integrations = useIntegrationsStore()

const loading = ref<boolean>(false)
const summary = ref<IntegrationsSummaryResponse | null>(null)

async function refresh(): Promise<void> {
  loading.value = true
  try {
    summary.value = await integrations.getSummary()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void refresh()
})
</script>

<template>
  <AppEmptyState v-if="loading && !summary" :title="t('common.loading')" loading />

  <div v-else class="space-y-4">
    <n-card class="app-card" :bordered="false">
      <div class="flex items-start justify-between gap-3">
        <div>
          <div class="text-sm font-semibold">{{ t('integrations.distribution.title') }}</div>
          <div class="app-meta-text mt-1">{{ t('integrations.distribution.subtitle') }}</div>
        </div>
        <n-tag
          v-if="summary"
          :type="summary.distribution.state === 'degraded' ? 'warning' : summary.distribution.state === 'ready' ? 'success' : 'default'"
          size="small"
        >
          {{ t(`integrations.states.${summary.distribution.state}`) }}
        </n-tag>
      </div>

      <div v-if="summary" class="mt-4 grid gap-3 md:grid-cols-3">
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('integrations.distribution.coverage') }}</div>
          <div class="mt-2 text-2xl font-semibold">{{ summary.distribution.summary.coverage_total }}</div>
        </div>
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('integrations.distribution.drifted') }}</div>
          <div class="mt-2 text-2xl font-semibold">{{ summary.distribution.summary.drifted_total }}</div>
        </div>
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('integrations.distribution.failed') }}</div>
          <div class="mt-2 text-2xl font-semibold">{{ summary.distribution.summary.failed_total }}</div>
        </div>
      </div>

      <div class="mt-4 flex flex-wrap gap-2">
        <n-button @click="router.push('/fleet')">{{ t('integrations.distribution.openFleet') }}</n-button>
        <n-button @click="router.push('/system/bulk-operations')">{{ t('integrations.distribution.openBulkOperations') }}</n-button>
      </div>
    </n-card>
  </div>
</template>
