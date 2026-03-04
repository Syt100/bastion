<script setup lang="ts">
import { computed, watchEffect } from 'vue'
import { useRouter } from 'vue-router'
import { NCard } from 'naive-ui'
import { ChevronForwardOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import AppIcon from '@/components/AppIcon.vue'

import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { DEFAULT_NOTIFICATIONS_TAB_KEY, getNotificationsNavItems } from '@/navigation/notifications'

const { t } = useI18n()
const router = useRouter()
const isDesktop = useMediaQuery(MQ.mdUp)

watchEffect(() => {
  if (!isDesktop.value) return
  const items = getNotificationsNavItems()
  const fallback = items.find((i) => i.key === DEFAULT_NOTIFICATIONS_TAB_KEY) ?? items[0]
  if (!fallback) return
  void router.replace(fallback.to)
})

const items = computed(() => getNotificationsNavItems())

function go(to: string): void {
  void router.push(to)
}
</script>

<template>
  <n-card v-if="!isDesktop" class="app-card" :bordered="false">
    <div class="app-divide-y">
      <button
        v-for="item in items"
        :key="item.key"
        type="button"
        class="app-list-row app-motion-soft"
        @click="go(item.to)"
      >
        <div class="flex items-start gap-3 min-w-0">
          <div
            class="app-icon-tile"
          >
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
</template>
