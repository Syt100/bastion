<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NCard, NIcon } from 'naive-ui'
import { ChevronForwardOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const router = useRouter()

import type { SettingsOverviewItem } from '@/navigation/settings'
import { getSettingsOverviewItems } from '@/navigation/settings'

const items: SettingsOverviewItem[] = getSettingsOverviewItems()

function go(to: string): void {
  void router.push(to)
}
</script>

<template>
  <n-card class="app-card" :bordered="false">
    <div class="app-divide-y">
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
            <div class="text-xs app-text-muted mt-0.5 truncate">{{ t(item.descriptionKey) }}</div>
          </div>
        </div>

        <n-icon size="18" class="app-text-muted flex-shrink-0">
          <ChevronForwardOutline />
        </n-icon>
      </button>
    </div>
  </n-card>
</template>
