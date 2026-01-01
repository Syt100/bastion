<script setup lang="ts">
import type { Component } from 'vue'
import { computed, h, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NButton,
  NCard,
  NDrawer,
  NDrawerContent,
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
import {
  ArchiveOutline,
  HomeOutline,
  MenuOutline,
  PeopleOutline,
  SettingsOutline,
} from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import type { SupportedLocale } from '@/i18n'
import { useSystemStore } from '@/stores/system'
import InsecureHttpBanner from '@/components/InsecureHttpBanner.vue'
import AppLogo from '@/components/AppLogo.vue'
import { useMediaQuery } from '@/lib/media'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const { t } = useI18n()

const ui = useUiStore()
const auth = useAuthStore()
const system = useSystemStore()

const activeKey = computed(() => route.path)
const mobileMenuOpen = ref(false)
const isDesktop = useMediaQuery('(min-width: 768px)')

watch(isDesktop, (value) => {
  if (value) {
    mobileMenuOpen.value = false
  }
})

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
  <n-layout has-sider class="min-h-screen bg-transparent">
    <n-layout-sider
      v-if="isDesktop"
      bordered
      class="bg-white/70 dark:bg-[#0b1220]/60 backdrop-blur"
      collapse-mode="width"
      :collapsed-width="64"
      :width="220"
    >
      <div class="h-14 px-4 flex items-center gap-3 border-b border-black/5 dark:border-white/10">
        <AppLogo />
        <n-tag size="small" type="info" :bordered="false">{{ t('common.mvp') }}</n-tag>
      </div>

      <n-menu :value="activeKey" :options="menuOptions" @update:value="(key) => router.push(String(key))" />
    </n-layout-sider>

    <n-layout class="bg-transparent">
      <n-layout-header
        bordered
        class="bg-white/70 dark:bg-[#0b1220]/60 backdrop-blur px-4 sticky top-0 z-50"
      >
        <div class="h-14 flex items-center justify-between max-w-7xl mx-auto">
          <div class="flex items-center gap-3">
            <n-button v-if="!isDesktop" quaternary @click="mobileMenuOpen = true">
              <template #icon>
                <n-icon><MenuOutline /></n-icon>
              </template>
            </n-button>

            <template v-if="!isDesktop">
              <AppLogo />
              <n-tag size="small" type="info" :bordered="false">{{ t('common.mvp') }}</n-tag>
            </template>
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

      <n-layout content-style="padding: 16px" class="bg-transparent">
        <div class="max-w-7xl mx-auto">
          <InsecureHttpBanner v-if="system.insecureHttp" class="mb-4" />
          <router-view />
        </div>
      </n-layout>
    </n-layout>
  </n-layout>

  <n-drawer v-if="!isDesktop" v-model:show="mobileMenuOpen" placement="left" :width="280">
    <n-drawer-content>
      <n-card class="mb-3" :bordered="false">
        <div class="flex items-center justify-between">
          <AppLogo size="sm" />
          <n-tag size="small" type="info" :bordered="false">{{ t('common.mvp') }}</n-tag>
        </div>
      </n-card>

      <n-menu
        :value="activeKey"
        :options="menuOptions"
        @update:value="
          (key) => {
            mobileMenuOpen = false
            router.push(String(key))
          }
        "
      />
    </n-drawer-content>
  </n-drawer>
</template>
