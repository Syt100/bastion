<script setup lang="ts">
import type { Component } from 'vue'
import { computed, h, ref, watch, watchEffect } from 'vue'
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
  EllipsisHorizontal,
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
import { MQ } from '@/lib/breakpoints'
import { LAYOUT } from '@/lib/layout'
import { formatToastError } from '@/lib/errors'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const { t } = useI18n()

const ui = useUiStore()
const auth = useAuthStore()
const system = useSystemStore()

const activeKey = computed(() => {
  const path = route.path
  const ordered = [...menuRouteKeys].sort((a, b) => b.length - a.length)
  for (const key of ordered) {
    if (key === '/') {
      if (path === '/') return '/'
      continue
    }
    if (path === key || path.startsWith(`${key}/`)) return key
  }
  return path
})
const mobileMenuOpen = ref(false)
const isDesktop = useMediaQuery(MQ.mdUp)

watch(isDesktop, (value) => {
  if (value) {
    mobileMenuOpen.value = false
  }
})

function icon(iconComponent: Component) {
  return () => h(NIcon, null, { default: () => h(iconComponent) })
}

const menuRouteKeys = ['/', '/jobs', '/agents', '/settings', '/settings/storage', '/settings/notifications'] as const
const menuRouteKeySet = new Set<string>(menuRouteKeys)

const settingsParentKey = 'settings'
const expandedKeys = ref<string[]>([])

watchEffect(() => {
  if (!isDesktop.value) return
  if (!route.path.startsWith('/settings')) return
  if (expandedKeys.value.includes(settingsParentKey)) return
  expandedKeys.value = [...expandedKeys.value, settingsParentKey]
})

const menuOptions = computed<MenuOption[]>(() => [
  { label: t('nav.dashboard'), key: '/', icon: icon(HomeOutline) },
  { label: t('nav.jobs'), key: '/jobs', icon: icon(ArchiveOutline) },
  { label: t('nav.agents'), key: '/agents', icon: icon(PeopleOutline) },
  ...(isDesktop.value
    ? [
        {
          label: t('nav.settings'),
          key: settingsParentKey,
          icon: icon(SettingsOutline),
          children: [
            { label: t('settings.menu.overview'), key: '/settings' },
            { label: t('settings.menu.storage'), key: '/settings/storage' },
            { label: t('settings.menu.notifications'), key: '/settings/notifications' },
          ],
        } satisfies MenuOption,
      ]
    : [{ label: t('nav.settings'), key: '/settings', icon: icon(SettingsOutline) }]),
])

const languageOptions = computed(() => [
  { label: '简体中文', key: 'zh-CN' },
  { label: 'English', key: 'en-US' },
])

function onSelectLanguage(key: string | number): void {
  ui.setLocale(key as SupportedLocale)
}

function navigateMenu(key: unknown): void {
  if (typeof key !== 'string') return
  if (!menuRouteKeySet.has(key)) return
  if (key === activeKey.value) return
  void router.push(key)
}

function onUpdateExpandedKeys(keys: string[]): void {
  expandedKeys.value = keys
}

const mobileActions = computed(() => [
  { label: '简体中文', key: 'zh-CN' },
  { label: 'English', key: 'en-US' },
  { type: 'divider', key: '__d1' },
  { label: ui.darkMode ? t('common.light') : t('common.dark'), key: 'toggle_theme' },
  { type: 'divider', key: '__d2' },
  { label: t('common.logout'), key: 'logout' },
])

function onSelectMobileAction(key: string | number): void {
  if (key === 'toggle_theme') {
    ui.toggleDarkMode()
    return
  }
  if (key === 'logout') {
    void onLogout()
    return
  }
  onSelectLanguage(key)
}

async function onLogout(): Promise<void> {
  try {
    await auth.logout()
    await router.push('/login')
  } catch (error) {
    message.error(formatToastError(t('errors.logoutFailed'), error, t))
  }
}
</script>

<template>
  <n-layout has-sider class="min-h-screen bg-transparent">
    <n-layout-sider
      v-if="isDesktop"
      bordered
      class="app-glass"
      collapse-mode="width"
      :collapsed-width="LAYOUT.siderCollapsedWidth"
      :width="LAYOUT.siderWidth"
    >
      <div class="h-14 px-4 flex items-center gap-3 border-b border-black/5 dark:border-white/10">
        <AppLogo />
        <n-tag size="small" type="info" :bordered="false">{{ t('common.beta') }}</n-tag>
      </div>

      <n-menu
        :value="activeKey"
        :options="menuOptions"
        :expanded-keys="expandedKeys"
        @update:value="navigateMenu"
        @update:expanded-keys="onUpdateExpandedKeys"
      />
    </n-layout-sider>

    <n-layout class="bg-transparent">
      <n-layout-header
        bordered
        class="app-glass px-4 sticky top-0 z-50"
      >
        <div class="h-14 flex items-center justify-between max-w-7xl mx-auto">
          <div class="flex items-center gap-3">
            <n-button v-if="!isDesktop" quaternary :aria-label="t('common.openMenu')" @click="mobileMenuOpen = true">
              <template #icon>
                <n-icon><MenuOutline /></n-icon>
              </template>
            </n-button>

            <template v-if="!isDesktop">
              <AppLogo />
              <n-tag size="small" type="info" :bordered="false">{{ t('common.beta') }}</n-tag>
            </template>
          </div>

          <div class="flex items-center gap-2">
            <template v-if="isDesktop">
              <n-dropdown :options="languageOptions" trigger="click" @select="onSelectLanguage">
                <n-button quaternary>{{ t('common.language') }}</n-button>
              </n-dropdown>
              <n-button quaternary @click="ui.toggleDarkMode()">
                {{ ui.darkMode ? t('common.light') : t('common.dark') }}
              </n-button>
              <n-button quaternary type="error" @click="onLogout">{{ t('common.logout') }}</n-button>
            </template>
            <template v-else>
              <n-dropdown :options="mobileActions" trigger="click" @select="onSelectMobileAction">
                <n-button quaternary :aria-label="t('common.more')">
                  <template #icon>
                    <n-icon><EllipsisHorizontal /></n-icon>
                  </template>
                </n-button>
              </n-dropdown>
            </template>
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

  <n-drawer v-if="!isDesktop" v-model:show="mobileMenuOpen" placement="left" :width="LAYOUT.mobileDrawerWidth">
    <n-drawer-content>
      <n-card class="mb-3" :bordered="false">
        <div class="flex items-center justify-between">
          <AppLogo size="sm" />
          <n-tag size="small" type="info" :bordered="false">{{ t('common.beta') }}</n-tag>
        </div>
      </n-card>

      <n-menu
        :value="activeKey"
        :options="menuOptions"
        @update:value="
          (key) => {
            mobileMenuOpen = false
            navigateMenu(key)
          }
        "
      />
    </n-drawer-content>
  </n-drawer>
</template>
