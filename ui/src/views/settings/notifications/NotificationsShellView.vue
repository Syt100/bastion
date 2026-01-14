<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NCard, NTabs, NTabPane } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { DEFAULT_NOTIFICATIONS_TAB_KEY, getNotificationsNavItems, isNotificationsTabKey } from '@/navigation/notifications'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const isDesktop = useMediaQuery(MQ.mdUp)

const isIndex = computed(() => route.path === '/settings/notifications')

const active = computed(() => {
  const match = route.path.match(/^\/settings\/notifications\/([^/]+)/)
  const key = match?.[1]
  if (key && isNotificationsTabKey(key)) return key
  return DEFAULT_NOTIFICATIONS_TAB_KEY
})

function go(key: unknown): void {
  if (typeof key !== 'string') return
  if (!isNotificationsTabKey(key)) return
  void router.push(`/settings/notifications/${key}`)
}

const tabs = computed(() => getNotificationsNavItems())
</script>

<template>
  <div class="space-y-4">
    <template v-if="isDesktop && !isIndex">
      <n-card class="app-card" :bordered="false">
        <n-tabs type="line" :value="active" :pane-style="{ display: 'none' }" @update:value="go">
          <n-tab-pane v-for="item in tabs" :key="item.key" :name="item.key" :tab="t(item.titleKey)" />
        </n-tabs>
      </n-card>
    </template>

    <router-view />
  </div>
</template>
