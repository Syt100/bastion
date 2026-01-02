<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NCard, NSelect, NTabs, NTabPane } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const isDesktop = useMediaQuery(MQ.mdUp)

const tabs = computed(() => [
  { label: t('settings.notifications.tabs.channels'), value: 'channels' },
  { label: t('settings.notifications.tabs.destinations'), value: 'destinations' },
  { label: t('settings.notifications.tabs.templates'), value: 'templates' },
  { label: t('settings.notifications.tabs.queue'), value: 'queue' },
])

const active = computed(() => {
  const path = route.path
  if (path.includes('/settings/notifications/channels')) return 'channels'
  if (path.includes('/settings/notifications/templates')) return 'templates'
  if (path.includes('/settings/notifications/queue')) return 'queue'
  return 'destinations'
})

function go(key: unknown): void {
  if (typeof key !== 'string') return
  void router.push(`/settings/notifications/${key}`)
}
</script>

<template>
  <div class="space-y-4">
    <n-card class="shadow-sm border border-black/5 dark:border-white/10" :bordered="false">
      <template v-if="isDesktop">
        <n-tabs
          type="line"
          :value="active"
          :pane-style="{ display: 'none' }"
          @update:value="go"
        >
          <n-tab-pane name="channels" :tab="t('settings.notifications.tabs.channels')" />
          <n-tab-pane name="destinations" :tab="t('settings.notifications.tabs.destinations')" />
          <n-tab-pane name="templates" :tab="t('settings.notifications.tabs.templates')" />
          <n-tab-pane name="queue" :tab="t('settings.notifications.tabs.queue')" />
        </n-tabs>
      </template>
      <template v-else>
        <n-select :value="active" :options="tabs" @update:value="go" />
      </template>
    </n-card>

    <router-view />
  </div>
</template>

