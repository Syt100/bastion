<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

import PageHeader from '@/components/PageHeader.vue'
import MobileTopBar from '@/components/MobileTopBar.vue'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'

const { t } = useI18n()
const route = useRoute()

const isDesktop = useMediaQuery(MQ.mdUp)

type MobileTopBarMeta = {
  titleKey: string
  backTo?: string | null
}

const mobileTopBarMeta = computed<MobileTopBarMeta>(() => {
  for (const record of [...route.matched].reverse()) {
    const raw = record.meta?.mobileTopBar as unknown
    if (!raw || typeof raw !== 'object') continue
    const meta = raw as { titleKey?: unknown; backTo?: unknown }
    if (typeof meta.titleKey !== 'string' || meta.titleKey.trim().length === 0) continue
    return {
      titleKey: meta.titleKey,
      backTo: typeof meta.backTo === 'string' ? meta.backTo : null,
    }
  }
  return { titleKey: 'settings.title', backTo: null }
})

const mobileTitle = computed(() => t(mobileTopBarMeta.value.titleKey))
</script>

<template>
  <div class="space-y-6">
    <MobileTopBar v-if="!isDesktop" :title="mobileTitle" :back-to="mobileTopBarMeta.backTo" />
    <PageHeader v-else :title="t('settings.title')" :subtitle="t('settings.subtitle')" />
    <router-view />
  </div>
</template>
