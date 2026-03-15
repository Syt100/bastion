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
import { useHubRuntimeConfigStore } from '@/stores/hubRuntimeConfig'
import { useSystemStore } from '@/stores/system'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()
const system = useSystemStore()
const hubRuntimeConfig = useHubRuntimeConfigStore()

const loading = ref<boolean>(false)
const publicBaseUrl = ref<string | null>(null)
const items = computed<SettingsOverviewItem[]>(() => getSettingsOverviewItemsForDomain('system'))

async function refresh(): Promise<void> {
  loading.value = true
  try {
    await system.refresh()
    const runtime = await hubRuntimeConfig.get()
    publicBaseUrl.value = runtime.effective.public_base_url ?? null
  } catch (error) {
    message.error(formatToastError(t('errors.fetchHubRuntimeConfigFailed'), error, t))
  } finally {
    loading.value = false
  }
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
      <div class="grid gap-3 md:grid-cols-2">
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('system.overview.publicBaseUrl') }}</div>
          <div class="mt-2 font-medium break-all">
            {{ publicBaseUrl || t('system.overview.publicBaseUrlMissing') }}
          </div>
          <n-tag class="mt-3" :type="publicBaseUrl ? 'success' : 'warning'" size="small">
            {{ publicBaseUrl ? t('system.overview.configured') : t('system.overview.setupRequired') }}
          </n-tag>
        </div>
        <div class="rounded-2xl app-panel-inset px-4 py-3">
          <div class="app-meta-text">{{ t('system.overview.version') }}</div>
          <div class="mt-2 font-medium">{{ system.version || '-' }}</div>
          <div class="app-meta-text mt-3">{{ t('system.overview.restartHint') }}</div>
        </div>
      </div>
    </n-card>

    <AppEmptyState v-if="loading && !system.version && !publicBaseUrl" :title="t('common.loading')" loading />

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
              <div class="font-medium truncate">{{ t(item.titleKey) }}</div>
              <div class="app-meta-text mt-0.5 truncate">{{ t(item.descriptionKey) }}</div>
            </div>
          </div>

          <AppIcon :component="ChevronForwardOutline" size="md" tone="muted" class="flex-shrink-0" />
        </button>
      </div>
    </n-card>
  </div>
</template>
