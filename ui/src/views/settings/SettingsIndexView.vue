<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NCard, NIcon } from 'naive-ui'
import { ChevronForwardOutline, CloudOutline, NotificationsOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const router = useRouter()

type SettingsItem = {
  key: string
  title: string
  description: string
  to: string
  icon: typeof CloudOutline
}

const items: SettingsItem[] = [
  {
    key: 'storage',
    title: t('settings.menu.storage'),
    description: t('settings.overview.storageDesc'),
    to: '/settings/storage',
    icon: CloudOutline,
  },
  {
    key: 'notifications',
    title: t('settings.menu.notifications'),
    description: t('settings.overview.notificationsDesc'),
    to: '/settings/notifications',
    icon: NotificationsOutline,
  },
]

function go(to: string): void {
  void router.push(to)
}
</script>

<template>
  <n-card class="app-card" :bordered="false">
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
            <div class="font-medium truncate">{{ item.title }}</div>
            <div class="text-xs opacity-70 mt-0.5 truncate">{{ item.description }}</div>
          </div>
        </div>

        <n-icon size="18" class="opacity-60 flex-shrink-0">
          <ChevronForwardOutline />
        </n-icon>
      </button>
    </div>
  </n-card>
</template>
