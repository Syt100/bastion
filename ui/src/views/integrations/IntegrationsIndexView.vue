<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NTag, useMessage } from 'naive-ui'
import { ChevronForwardOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import AppEmptyState from '@/components/AppEmptyState.vue'
import AppIcon from '@/components/AppIcon.vue'
import { formatToastError } from '@/lib/errors'
import { getSettingsOverviewItemsForDomain, type SettingsOverviewItem } from '@/navigation/settings'
import { useIntegrationsStore, type IntegrationsSummaryResponse, type IntegrationsDomainState } from '@/stores/integrations'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()
const integrations = useIntegrationsStore()

const loading = ref<boolean>(false)
const summary = ref<IntegrationsSummaryResponse | null>(null)

const items = computed<SettingsOverviewItem[]>(() => getSettingsOverviewItemsForDomain('integrations'))

function stateTagType(state: IntegrationsDomainState): 'success' | 'warning' | 'default' {
  if (state === 'ready') return 'success'
  if (state === 'degraded') return 'warning'
  return 'default'
}

function stateLabel(state: IntegrationsDomainState): string {
  return t(`integrations.states.${state}`)
}

async function refresh(): Promise<void> {
  loading.value = true
  try {
    summary.value = await integrations.getSummary()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchWebdavSecretsFailed'), error, t))
  } finally {
    loading.value = false
  }
}

function descriptionFor(item: SettingsOverviewItem): string {
  if (!summary.value) return t(item.descriptionKey)
  if (item.key === 'storage') {
    const s = summary.value.storage
    return t('integrations.overview.storageMeta', s.summary)
  }
  if (item.key === 'notifications') {
    const s = summary.value.notifications
    return t('integrations.overview.notificationsMeta', s.summary)
  }
  if (item.key === 'distribution') {
    const s = summary.value.distribution
    return t('integrations.overview.distributionMeta', s.summary)
  }
  return t(item.descriptionKey)
}

function stateFor(key: string): IntegrationsDomainState | null {
  if (!summary.value) return null
  if (key === 'storage') return summary.value.storage.state
  if (key === 'notifications') return summary.value.notifications.state
  if (key === 'distribution') return summary.value.distribution.state
  return null
}

function go(to: string): void {
  void router.push(to)
}

onMounted(() => {
  void refresh()
})
</script>

<template>
  <div class="space-y-4">
    <n-card class="app-card" :bordered="false">
      <div class="flex items-center justify-between gap-3">
        <div>
          <div class="text-sm font-semibold">{{ t('integrations.overview.title') }}</div>
          <div class="app-meta-text mt-1">{{ t('integrations.overview.subtitle') }}</div>
        </div>
      </div>
    </n-card>

    <AppEmptyState v-if="loading && !summary" :title="t('common.loading')" loading />

    <n-card v-else class="app-card" :bordered="false">
      <div class="app-divide-y">
        <button
          v-for="item in items"
          :key="item.key"
          type="button"
          class="app-list-row app-motion-soft"
          @click="go(item.to)"
        >
          <div class="flex items-start gap-3 min-w-0">
            <div class="app-icon-tile">
              <AppIcon :component="item.icon" size="lg" tone="primary" />
            </div>
            <div class="min-w-0">
              <div class="flex items-center gap-2 flex-wrap">
                <div class="font-medium truncate">{{ t(item.titleKey) }}</div>
                <n-tag
                  v-if="stateFor(item.key)"
                  :type="stateTagType(stateFor(item.key)!)"
                  size="small"
                >
                  {{ stateLabel(stateFor(item.key)!) }}
                </n-tag>
              </div>
              <div class="app-meta-text mt-0.5">{{ descriptionFor(item) }}</div>
            </div>
          </div>

          <AppIcon :component="ChevronForwardOutline" size="md" tone="muted" class="flex-shrink-0" />
        </button>
      </div>
    </n-card>
  </div>
</template>
