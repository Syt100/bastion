<script setup lang="ts">
import { computed, watchEffect } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NIcon } from 'naive-ui'
import { ChevronForwardOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

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
    <div class="divide-y divide-black/5 dark:divide-white/10">
      <button
        v-for="item in items"
        :key="item.key"
        type="button"
        class="app-list-row"
        @click="go(item.to)"
      >
        <div class="flex items-start gap-3 min-w-0">
          <div
            class="app-icon-tile"
          >
            <n-icon size="20">
              <component :is="item.icon" />
            </n-icon>
          </div>
        <div class="min-w-0">
            <div class="font-medium truncate">{{ t(item.titleKey) }}</div>
            <div class="text-xs opacity-70 mt-0.5 truncate">{{ t(item.descriptionKey) }}</div>
          </div>
        </div>

        <n-icon size="18" class="opacity-60 flex-shrink-0">
          <ChevronForwardOutline />
        </n-icon>
      </button>
    </div>
  </n-card>
</template>
