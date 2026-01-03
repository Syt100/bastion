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

const backTarget = computed(() => {
  if (route.path.startsWith('/settings/notifications/')) return '/settings/notifications'
  return '/settings'
})

function backToSettingsOverview(): void {
  void router.push(backTarget.value)
}

const showMobileBack = computed(() => !isDesktop.value && route.path !== '/settings')
</script>

<template>
  <div class="space-y-6">
    <PageHeader :title="t('settings.title')" :subtitle="t('settings.subtitle')">
      <template v-if="showMobileBack" #prefix>
        <n-button quaternary size="small" @click="backToSettingsOverview">
          <template #icon>
            <n-icon><ChevronBackOutline /></n-icon>
          </template>
          {{ t('common.return') }}
        </n-button>
      </template>
    </PageHeader>
    <router-view />
  </div>
</template>
