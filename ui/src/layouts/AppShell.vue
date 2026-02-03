<script setup lang="ts">
import type { Component } from 'vue'
import { computed, h, onMounted, ref, watch, watchEffect } from 'vue'
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
  NSelect,
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
import { useAgentsStore } from '@/stores/agents'
import { useUiStore } from '@/stores/ui'
import type { SupportedLocale } from '@/i18n'
import { useSystemStore } from '@/stores/system'
import InsecureHttpBanner from '@/components/InsecureHttpBanner.vue'
import AppLogo from '@/components/AppLogo.vue'
import { useMediaQuery } from '@/lib/media'
import { MQ } from '@/lib/breakpoints'
import { LAYOUT } from '@/lib/layout'
import { formatToastError } from '@/lib/errors'
import { getSettingsMenuRouteKeys, getSettingsSidebarItems } from '@/navigation/settings'
import { getLocaleDropdownOptions } from '@/i18n/language'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const { t } = useI18n()

const ui = useUiStore()
const auth = useAuthStore()
const agents = useAgentsStore()
const system = useSystemStore()

const versionTag = computed(() => (system.version ? `v${system.version}` : null))

const nodeIdParam = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : null))
const menuPath = computed(() => (route.path.startsWith('/n/') ? route.path.replace(/^\/n\/[^/]+/, '') || '/' : route.path))
const nodeSuffix = computed(() => {
  if (!route.path.startsWith('/n/')) return null
  const suffix = route.path.replace(/^\/n\/[^/]+/, '')
  // Job-scoped URLs include job/run ids which are not meaningful across nodes.
  // When switching node from within a job context, return to the node-scoped jobs workspace.
  if (suffix.startsWith('/jobs/')) return '/jobs'
  return suffix === '' ? '/jobs' : suffix
})
const selectedNodeId = computed({
  get: () => nodeIdParam.value ?? ui.preferredNodeId ?? 'hub',
  set: (value: string) => void onSelectNode(value),
})

const isNodeScoped = computed(() => route.path.startsWith('/n/'))
const nodePickerLabel = computed(() =>
  isNodeScoped.value ? t('nav.nodePicker.currentLabel') : t('nav.nodePicker.preferredLabel'),
)
const nodePickerHint = computed(() =>
  isNodeScoped.value ? t('nav.nodePicker.hintCurrent') : t('nav.nodePicker.hintPreferred'),
)

const activeKey = computed(() => {
  const ordered = [...menuRouteKeys].sort((a, b) => b.length - a.length)
  for (const key of ordered) {
    if (key === '/') {
      if (menuPath.value === '/') return '/'
      continue
    }
    if (menuPath.value === key || menuPath.value.startsWith(`${key}/`)) return key
  }
  return menuPath.value
})
const mobileMenuOpen = ref(false)
const isDesktop = useMediaQuery(MQ.mdUp)
const isJobsWorkbench = computed(
  () =>
    isDesktop.value &&
    route.path.startsWith('/n/') &&
    (menuPath.value === '/jobs' || menuPath.value.startsWith('/jobs/')),
)

watch(isDesktop, (value) => {
  if (value) {
    mobileMenuOpen.value = false
  }
})

function icon(iconComponent: Component) {
  return () => h(NIcon, null, { default: () => h(iconComponent) })
}

const menuRouteKeys: string[] = ['/', '/jobs', '/agents', ...getSettingsMenuRouteKeys()]
const menuRouteKeySet = new Set<string>(menuRouteKeys)

const settingsParentKey = 'settings'
const expandedKeys = ref<string[]>([])

watchEffect(() => {
  if (!isDesktop.value) return
  if (!menuPath.value.startsWith('/settings')) return
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
            ...getSettingsSidebarItems().map((item) => ({
              label: t(item.titleKey),
              key: item.to,
            })),
          ],
        } satisfies MenuOption,
      ]
    : [{ label: t('nav.settings'), key: '/settings', icon: icon(SettingsOutline) }]),
])

const languageOptions = computed(() => getLocaleDropdownOptions())

function onSelectLanguage(key: string | number): void {
  ui.setLocale(key as SupportedLocale)
}

function openDocs(): void {
  // Open in a new tab so users can keep their current UI state.
  const docsRoot = ui.locale === 'zh-CN' ? '/docs/zh/' : '/docs/'
  window.open(docsRoot, '_blank', 'noopener')
}

function navigateMenu(key: unknown): void {
  if (typeof key !== 'string') return
  if (!menuRouteKeySet.has(key)) return
  if (key === activeKey.value) return
  if (key === '/jobs') {
    void router.push(`/n/${encodeURIComponent(selectedNodeId.value)}/jobs`)
    return
  }
  if (key === '/settings/storage') {
    void router.push(`/n/${encodeURIComponent(selectedNodeId.value)}/settings/storage`)
    return
  }
  void router.push(key)
}

function onUpdateExpandedKeys(keys: string[]): void {
  expandedKeys.value = keys
}

const mobileActions = computed(() => [
  ...getLocaleDropdownOptions(),
  { type: 'divider', key: '__d1' },
  { label: t('common.help'), key: 'docs' },
  { label: ui.darkMode ? t('common.light') : t('common.dark'), key: 'toggle_theme' },
  { type: 'divider', key: '__d2' },
  { label: t('common.logout'), key: 'logout' },
])

function onSelectMobileAction(key: string | number): void {
  if (key === 'docs') {
    openDocs()
    return
  }
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

async function onSelectNode(nodeId: string): Promise<void> {
  ui.setPreferredNodeId(nodeId)

  // On global pages, the node picker sets the preferred node only.
  if (!route.path.startsWith('/n/')) return

  const suffix = nodeSuffix.value ?? '/jobs'
  const target = `/n/${encodeURIComponent(nodeId)}/${suffix.replace(/^\/+/, '')}`
  if (route.path === target) return
  void router.push(target)
}

function formatNodeLabel(agentId: string | null): string {
  if (!agentId) return t('jobs.nodes.hub')
  const agent = agents.items.find((a) => a.id === agentId)
  if (!agent) return agentId
  const status = agent.revoked
    ? t('agents.status.revoked')
    : agent.online
      ? t('agents.status.online')
      : t('agents.status.offline')
  return agent.name ? `${agent.name} — ${status}` : `${agent.id} — ${status}`
}

const nodeOptions = computed(() => [
  { label: formatNodeLabel(null), value: 'hub' },
  ...agents.items.map((a) => ({
    label: formatNodeLabel(a.id),
    value: a.id,
    disabled: a.revoked,
  })),
])

onMounted(async () => {
  try {
    await agents.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  }
})

watch(nodeIdParam, (value) => {
  if (!value) return
  ui.setPreferredNodeId(value)
})
</script>

<template>
  <n-layout
    has-sider
    class="bg-transparent"
    :class="isDesktop ? 'h-screen overflow-hidden' : 'min-h-screen'"
  >
    <n-layout-sider
      v-if="isDesktop"
      bordered
      class="app-glass flex flex-col"
      collapse-mode="width"
      :collapsed-width="LAYOUT.siderCollapsedWidth"
      :width="LAYOUT.siderWidth"
    >
      <div class="h-14 px-4 flex items-center gap-3 border-b border-[color:var(--app-border)]">
        <AppLogo />
        <n-tag v-if="versionTag" size="small" type="info" :bordered="false">{{ versionTag }}</n-tag>
      </div>

      <div class="px-4 py-3 border-b border-[color:var(--app-border)]">
        <div class="text-xs app-text-muted mb-1">{{ nodePickerLabel }}</div>
        <n-select v-model:value="selectedNodeId" :options="nodeOptions" filterable />
        <div class="text-xs app-text-muted mt-2">{{ nodePickerHint }}</div>
      </div>

      <div class="flex-1 min-h-0 overflow-y-auto">
        <n-menu
          :value="activeKey"
          :options="menuOptions"
          :expanded-keys="expandedKeys"
          @update:value="navigateMenu"
          @update:expanded-keys="onUpdateExpandedKeys"
        />
      </div>
    </n-layout-sider>

    <n-layout class="bg-transparent" :class="isDesktop ? 'h-full min-h-0' : ''">
      <n-layout-header
        bordered
        class="app-glass app-topbar px-4"
        :class="isDesktop ? 'shrink-0' : 'sticky top-0 z-50'"
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
              <n-tag v-if="versionTag" size="small" type="info" :bordered="false">{{ versionTag }}</n-tag>
            </template>
          </div>

          <div class="flex items-center gap-2">
            <template v-if="isDesktop">
              <n-button quaternary @click="openDocs">{{ t('common.help') }}</n-button>
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

      <div
        data-testid="app-shell-content"
        class="bg-transparent"
        :class="
          isDesktop
            ? isJobsWorkbench
              ? 'flex-1 min-h-0 overflow-hidden'
              : 'flex-1 min-h-0 overflow-y-auto'
            : ''
        "
      >
        <div class="p-4" :class="isDesktop && isJobsWorkbench ? 'h-full min-h-0 flex flex-col' : ''">
          <div
            class="max-w-7xl mx-auto w-full"
            :class="isDesktop && isJobsWorkbench ? 'flex-1 min-h-0 flex flex-col' : ''"
          >
            <InsecureHttpBanner v-if="system.insecureHttp" class="mb-4" />
            <div :class="isDesktop && isJobsWorkbench ? 'flex-1 min-h-0' : ''">
              <router-view />
            </div>
          </div>
        </div>
      </div>
    </n-layout>
  </n-layout>

  <n-drawer v-if="!isDesktop" v-model:show="mobileMenuOpen" placement="left" :width="LAYOUT.mobileDrawerWidth">
    <n-drawer-content>
      <n-card class="mb-3" :bordered="false">
        <div class="flex items-center justify-between">
          <AppLogo size="sm" />
          <n-tag v-if="versionTag" size="small" type="info" :bordered="false">{{ versionTag }}</n-tag>
        </div>
      </n-card>

      <n-card class="mb-3" :bordered="false">
        <div class="text-xs app-text-muted mb-2">{{ nodePickerLabel }}</div>
        <n-select
          v-model:value="selectedNodeId"
          :options="nodeOptions"
          filterable
          @update:value="mobileMenuOpen = false"
        />
        <div class="text-xs app-text-muted mt-2">{{ nodePickerHint }}</div>
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
