<script setup lang="ts">
import type { Component } from 'vue'
import { computed, h } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NButton,
  NDropdown,
  NIcon,
  NLayout,
  NLayoutHeader,
  NLayoutSider,
  NMenu,
  NTag,
  type MenuOption,
  useMessage,
} from 'naive-ui'
import { HomeOutline, SettingsOutline, PeopleOutline, ArchiveOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import type { SupportedLocale } from '@/i18n'
import { useSystemStore } from '@/stores/system'
import InsecureHttpBanner from '@/components/InsecureHttpBanner.vue'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const { t } = useI18n()

const ui = useUiStore()
const auth = useAuthStore()
const system = useSystemStore()

const activeKey = computed(() => route.path)

function icon(iconComponent: Component) {
  return () => h(NIcon, null, { default: () => h(iconComponent) })
}

const menuOptions = computed<MenuOption[]>(() => [
  { label: t('nav.dashboard'), key: '/', icon: icon(HomeOutline) },
  { label: t('nav.jobs'), key: '/jobs', icon: icon(ArchiveOutline) },
  { label: t('nav.agents'), key: '/agents', icon: icon(PeopleOutline) },
  { label: t('nav.settings'), key: '/settings', icon: icon(SettingsOutline) },
])

const languageOptions = computed(() => [
  { label: '简体中文', key: 'zh-CN' },
  { label: 'English', key: 'en-US' },
])

function onSelectLanguage(key: string | number): void {
  ui.setLocale(key as SupportedLocale)
}

async function onLogout(): Promise<void> {
  try {
    await auth.logout()
    await router.push('/login')
  } catch {
    message.error(t('errors.logoutFailed'))
  }
}
</script>

<template>
  <n-layout class="h-screen">
    <n-layout-header bordered class="px-4">
      <div class="h-14 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="font-semibold tracking-wide">{{ t('app.name') }}</div>
          <n-tag size="small" type="info" :bordered="false">{{ t('common.mvp') }}</n-tag>
        </div>
        <div class="flex items-center gap-2">
          <n-dropdown :options="languageOptions" trigger="click" @select="onSelectLanguage">
            <n-button quaternary>{{ t('common.language') }}</n-button>
          </n-dropdown>
          <n-button quaternary @click="ui.toggleDarkMode()">
            {{ ui.darkMode ? t('common.light') : t('common.dark') }}
          </n-button>
          <n-button quaternary type="error" @click="onLogout">{{ t('common.logout') }}</n-button>
        </div>
      </div>
    </n-layout-header>

    <n-layout has-sider class="h-[calc(100vh-56px)]">
      <n-layout-sider bordered collapse-mode="width" :collapsed-width="64" :width="220">
        <n-menu
          :value="activeKey"
          :options="menuOptions"
          @update:value="(key) => router.push(String(key))"
        />
      </n-layout-sider>
      <n-layout content-style="padding: 16px">
        <InsecureHttpBanner v-if="system.insecureHttp" class="mb-4" />
        <router-view />
      </n-layout>
    </n-layout>
  </n-layout>
</template>
