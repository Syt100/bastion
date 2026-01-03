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
  <n-card class="shadow-sm border border-black/5 dark:border-white/10" :bordered="false">
    <div class="divide-y divide-black/5 dark:divide-white/10">
      <button
        v-for="item in items"
        :key="item.key"
        type="button"
        class="w-full text-left px-3 py-3 hover:bg-black/5 dark:hover:bg-white/5 transition flex items-center justify-between gap-3"
        @click="go(item.to)"
      >
        <div class="flex items-start gap-3 min-w-0">
          <div
            class="w-10 h-10 rounded-lg bg-black/5 dark:bg-white/10 flex items-center justify-center flex-shrink-0"
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

