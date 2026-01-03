<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NIcon } from 'naive-ui'
import { ChevronBackOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const isDesktop = useMediaQuery(MQ.mdUp)

const showBackToOverview = computed(() => {
  if (isDesktop.value) return false
  if (route.path === '/settings') return false
  // Avoid stacking two back bars on notification subpages (they already have "back to notifications list").
  if (route.path.startsWith('/settings/notifications/')) return false
  return true
})

const mobileSectionTitle = computed(() => {
  if (route.path.startsWith('/settings/storage')) return t('settings.menu.storage')
  if (route.path.startsWith('/settings/notifications')) return t('settings.menu.notifications')
  return t('settings.title')
})

function backToSettingsOverview(): void {
  void router.push('/settings')
}
</script>

<template>
  <div class="space-y-6">
    <template v-if="showBackToOverview">
      <div class="flex items-center gap-2">
        <n-button quaternary size="small" @click="backToSettingsOverview">
          <template #icon>
            <n-icon><ChevronBackOutline /></n-icon>
          </template>
          {{ t('common.back') }}
        </n-button>
        <div class="text-sm font-medium truncate">{{ mobileSectionTitle }}</div>
      </div>
    </template>

    <PageHeader :title="t('settings.title')" :subtitle="t('settings.subtitle')" />
    <router-view />
  </div>
</template>
