<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NCard, NIcon, NTabs, NTabPane } from 'naive-ui'
import { ChevronBackOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const isDesktop = useMediaQuery(MQ.mdUp)

const isIndex = computed(() => route.path === '/settings/notifications')

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

function back(): void {
  void router.push('/settings/notifications')
}
</script>

<template>
  <div class="space-y-4">
    <template v-if="isDesktop && !isIndex">
      <n-card class="shadow-sm border border-black/5 dark:border-white/10" :bordered="false">
        <n-tabs type="line" :value="active" :pane-style="{ display: 'none' }" @update:value="go">
          <n-tab-pane name="channels" :tab="t('settings.notifications.tabs.channels')" />
          <n-tab-pane name="destinations" :tab="t('settings.notifications.tabs.destinations')" />
          <n-tab-pane name="templates" :tab="t('settings.notifications.tabs.templates')" />
          <n-tab-pane name="queue" :tab="t('settings.notifications.tabs.queue')" />
        </n-tabs>
      </n-card>
    </template>
    <template v-else-if="!isDesktop && !isIndex">
      <div class="flex items-center gap-2">
        <n-button quaternary size="small" @click="back">
          <template #icon>
            <n-icon><ChevronBackOutline /></n-icon>
          </template>
          {{ t('common.back') }}
        </n-button>
        <div class="text-sm font-medium truncate">{{ t('settings.menu.notifications') }}</div>
      </div>
    </template>

    <router-view />
  </div>
</template>
