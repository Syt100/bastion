<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NCard, NMenu, NSelect, type MenuOption } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const isDesktop = useMediaQuery(MQ.mdUp)

const menuOptions = computed<MenuOption[]>(() => [
  { label: t('settings.menu.storage'), key: '/settings/storage' },
  { label: t('settings.menu.notifications'), key: '/settings/notifications' },
])

const activeKey = computed(() => {
  const path = route.path
  if (path.startsWith('/settings/notifications')) return '/settings/notifications'
  return '/settings/storage'
})

function navigate(key: unknown): void {
  if (typeof key !== 'string') return
  if (key === route.path) return
  void router.push(key)
}
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('settings.title')" :subtitle="t('settings.subtitle')" />

    <div class="grid grid-cols-1 md:grid-cols-[240px_1fr] gap-4">
      <div v-if="isDesktop">
        <n-card class="shadow-sm border border-black/5 dark:border-white/10" :bordered="false">
          <n-menu :value="activeKey" :options="menuOptions" @update:value="navigate" />
        </n-card>
      </div>
      <div v-else>
        <n-card class="shadow-sm border border-black/5 dark:border-white/10" :bordered="false">
          <n-select :value="activeKey" :options="menuOptions" @update:value="navigate" />
        </n-card>
      </div>

      <div>
        <router-view />
      </div>
    </div>
  </div>
</template>

