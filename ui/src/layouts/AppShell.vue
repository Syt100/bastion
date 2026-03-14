<script setup lang="ts">
import type { Component } from 'vue'
import { computed, h, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NButton,
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
import { MenuOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

import AppLogo from '@/components/AppLogo.vue'
import InsecureHttpBanner from '@/components/InsecureHttpBanner.vue'
import { useAuthStore } from '@/stores/auth'
import { useAgentsStore } from '@/stores/agents'
import { useSystemStore } from '@/stores/system'
import { useUiStore } from '@/stores/ui'
import type { SupportedLocale } from '@/i18n'
import { getLocaleDropdownOptions } from '@/i18n/language'
import { MQ } from '@/lib/breakpoints'
import { formatToastError } from '@/lib/errors'
import { LAYOUT } from '@/lib/layout'
import { useMediaQuery } from '@/lib/media'
import { PRIMARY_NAV_ITEMS, getSecondaryNavItems, type ShellPrimaryNavKey } from '@/navigation/shell'
import { isNodeScopedPath, nodeScopedPath, stripNodeScope } from '@/lib/nodeRoute'
import { parseScopeQueryValue, scopeFromNodeId, scopeToNodeId, type ScopeValue } from '@/lib/scope'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const { t } = useI18n()

const ui = useUiStore()
const auth = useAuthStore()
const agents = useAgentsStore()
const system = useSystemStore()

const versionTag = computed(() => (system.version ? `v${system.version}` : null))
const mobileMenuOpen = ref(false)
const isDesktop = useMediaQuery(MQ.mdUp)

watch(isDesktop, (value) => {
  if (value) mobileMenuOpen.value = false
})

function icon(iconComponent: Component) {
  return () => h(NIcon, null, { default: () => h(iconComponent) })
}

const currentPrimaryNav = computed<ShellPrimaryNavKey>(() => {
  for (const record of [...route.matched].reverse()) {
    const value = record.meta?.primaryNav
    if (
      value === 'command-center' ||
      value === 'jobs' ||
      value === 'runs' ||
      value === 'fleet' ||
      value === 'integrations' ||
      value === 'system'
    ) {
      return value
    }
  }
  return 'command-center'
})

const currentScopeMode = computed<'collection' | 'detail' | 'none' | 'legacy-node'>(() => {
  for (const record of [...route.matched].reverse()) {
    const value = record.meta?.scopeMode
    if (value === 'collection' || value === 'detail' || value === 'none' || value === 'legacy-node') {
      return value
    }
  }
  return 'none'
})

const explicitScope = computed(() => parseScopeQueryValue(route.query.scope))
const legacyNodeId = computed(() => (typeof route.params.nodeId === 'string' ? route.params.nodeId : null))
const isLegacyNodeRoute = computed(() => currentScopeMode.value === 'legacy-node' || !!legacyNodeId.value)
const nodeSuffix = computed(() => {
  if (!isNodeScopedPath(route.path)) return null
  const suffix = stripNodeScope(route.path)
  if (suffix.startsWith('/jobs/')) return '/jobs'
  return suffix === '/' ? '/jobs' : suffix
})

const selectedScope = computed<ScopeValue>({
  get: () => {
    if (legacyNodeId.value) return scopeFromNodeId(legacyNodeId.value)
    if (currentScopeMode.value === 'collection') return explicitScope.value ?? ui.preferredScope
    return ui.preferredScope
  },
  set: (value) => {
    void onSelectScope(value)
  },
})

const scopePickerLabel = computed(() =>
  currentScopeMode.value === 'collection' || isLegacyNodeRoute.value
    ? t('nav.scopePicker.currentLabel')
    : t('nav.scopePicker.preferredLabel'),
)

const scopePickerHint = computed(() => {
  if (isLegacyNodeRoute.value) return t('nav.scopePicker.hintLegacy')
  if (currentScopeMode.value === 'collection') return t('nav.scopePicker.hintCurrent')
  return t('nav.scopePicker.hintPreferred')
})

function formatScopeLabel(scope: ScopeValue): string {
  if (scope === 'all') return t('nav.scopePicker.all')
  if (scope === 'hub') return t('nav.scopePicker.hub')
  const agentId = scopeToNodeId(scope)
  const agent = agents.items.find((item) => item.id === agentId)
  if (!agent) return agentId || scope
  const status = agent.revoked
    ? t('agents.status.revoked')
    : agent.online
      ? t('agents.status.online')
      : t('agents.status.offline')
  return agent.name?.trim() ? `${agent.name} — ${status}` : `${agent.id} — ${status}`
}

const scopeOptions = computed(() => {
  const options: Array<{ label: string; value: ScopeValue; disabled?: boolean }> = []
  if (!isLegacyNodeRoute.value) {
    options.push({ label: formatScopeLabel('all'), value: 'all' })
  }
  options.push({ label: formatScopeLabel('hub'), value: 'hub' })
  for (const agent of agents.items) {
    const scope = `agent:${agent.id}` as ScopeValue
    options.push({
      label: formatScopeLabel(scope),
      value: scope,
      disabled: agent.revoked,
    })
  }
  return options
})

const primaryMenuOptions = computed<MenuOption[]>(() =>
  PRIMARY_NAV_ITEMS.map((item) => ({
    label: t(item.titleKey),
    key: item.to,
    icon: icon(item.icon),
  })),
)

const secondaryNavItems = computed(() => getSecondaryNavItems(currentPrimaryNav.value))
const secondaryMenuOptions = computed<MenuOption[]>(() =>
  secondaryNavItems.value.map((item) => ({
    label: t(item.titleKey),
    key: item.to,
  })),
)

const activePrimaryKey = computed(() => {
  const current = PRIMARY_NAV_ITEMS.find((item) => item.key === currentPrimaryNav.value)
  return current?.to ?? '/'
})

const activeSecondaryKey = computed(() => {
  for (const item of secondaryNavItems.value) {
    if (route.path === item.to || route.path.startsWith(`${item.to}/`)) return item.to
  }
  return null
})

const currentPrimaryLabel = computed(() => {
  const current = PRIMARY_NAV_ITEMS.find((item) => item.key === currentPrimaryNav.value)
  return current ? t(current.titleKey) : t('app.name')
})

const isJobsWorkbench = computed(
  () => isDesktop.value && currentPrimaryNav.value === 'jobs' && isNodeScopedPath(route.path),
)

function navigatePrimary(key: unknown): void {
  if (typeof key !== 'string') return
  if (key === route.path) return
  void router.push(key)
}

function navigateSecondary(key: unknown): void {
  if (typeof key !== 'string') return
  void router.push(key)
}

async function onSelectScope(scope: ScopeValue): Promise<void> {
  ui.setPreferredScope(scope)

  if (isLegacyNodeRoute.value) {
    const nodeId = scopeToNodeId(scope)
    if (!nodeId) return
    const suffix = nodeSuffix.value ?? '/jobs'
    const target = nodeScopedPath(nodeId, suffix)
    if (route.path === target) return
    await router.push(target)
    return
  }

  if (currentScopeMode.value === 'collection') {
    await router.push({
      path: route.path,
      query: {
        ...route.query,
        scope,
      },
      hash: route.hash,
    })
  }
}

function onSelectLanguage(key: string | number): void {
  ui.setLocale(key as SupportedLocale)
}

function openDocs(): void {
  const docsRoot = ui.locale === 'zh-CN' ? '/docs/zh/' : '/docs/'
  window.open(docsRoot, '_blank', 'noopener')
}

async function onLogout(): Promise<void> {
  try {
    await auth.logout()
    await router.push('/login')
  } catch (error) {
    message.error(formatToastError(t('errors.logoutFailed'), error, t))
  }
}

const languageOptions = computed(() => getLocaleDropdownOptions())

onMounted(async () => {
  try {
    await agents.refresh()
  } catch (error) {
    message.error(formatToastError(t('errors.fetchAgentsFailed'), error, t))
  }
})

watch(legacyNodeId, (value) => {
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
      :width="LAYOUT.siderWidth"
      :collapsed-width="LAYOUT.siderCollapsedWidth"
      collapse-mode="width"
    >
      <div class="h-14 px-4 flex items-center gap-3 border-b border-[color:var(--app-border)]">
        <AppLogo />
        <n-tag v-if="versionTag" size="small" type="info" :bordered="false">{{ versionTag }}</n-tag>
      </div>

      <div class="px-4 py-3 border-b border-[color:var(--app-border)] space-y-2">
        <div class="text-xs app-text-muted">{{ scopePickerLabel }}</div>
        <n-select v-model:value="selectedScope" :options="scopeOptions" filterable />
        <div class="text-xs app-text-muted">{{ scopePickerHint }}</div>
      </div>

      <div class="flex-1 min-h-0 overflow-y-auto px-2 py-3">
        <n-menu :value="activePrimaryKey" :options="primaryMenuOptions" @update:value="navigatePrimary" />

        <div v-if="secondaryMenuOptions.length > 0" class="mt-5">
          <div class="px-3 pb-2 text-xs uppercase tracking-[0.16em] app-text-muted">
            {{ t('nav.context') }}
          </div>
          <n-menu :value="activeSecondaryKey" :options="secondaryMenuOptions" @update:value="navigateSecondary" />
        </div>
      </div>
    </n-layout-sider>

    <n-layout class="bg-transparent" :class="isDesktop ? 'h-full min-h-0' : ''">
      <n-layout-header
        bordered
        class="app-glass app-topbar px-4"
        :class="isDesktop ? 'shrink-0' : 'sticky top-0 z-50'"
      >
        <div class="h-14 flex items-center justify-between max-w-[88rem] mx-auto gap-4">
          <div class="flex items-center gap-3 min-w-0">
            <n-button v-if="!isDesktop" quaternary :aria-label="t('common.openMenu')" @click="mobileMenuOpen = true">
              <template #icon>
                <n-icon><MenuOutline /></n-icon>
              </template>
            </n-button>

            <template v-if="!isDesktop">
              <AppLogo />
              <n-tag v-if="versionTag" size="small" type="info" :bordered="false">{{ versionTag }}</n-tag>
            </template>

            <div v-else class="min-w-0">
              <div class="text-xs uppercase tracking-[0.16em] app-text-muted">{{ t('nav.workspace') }}</div>
              <div class="font-medium truncate">{{ currentPrimaryLabel }}</div>
            </div>
          </div>

          <div v-if="isDesktop" class="app-topbar-action-group flex items-center gap-1 rounded-full px-1 py-1">
            <n-button quaternary size="small" @click="openDocs">{{ t('common.help') }}</n-button>
            <n-dropdown :options="languageOptions" trigger="click" @select="onSelectLanguage">
              <n-button quaternary size="small">{{ t('common.language') }}</n-button>
            </n-dropdown>
            <n-button quaternary size="small" @click="ui.toggleDarkMode()">
              {{ ui.darkMode ? t('common.light') : t('common.dark') }}
            </n-button>
            <n-button quaternary size="small" type="error" @click="onLogout">{{ t('common.logout') }}</n-button>
          </div>
        </div>
      </n-layout-header>

      <main
        data-testid="app-shell-main"
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
          <div class="max-w-[88rem] mx-auto w-full" :class="isDesktop && isJobsWorkbench ? 'flex-1 min-h-0 flex flex-col' : ''">
            <InsecureHttpBanner v-if="system.insecureHttp" class="mb-4" />
            <div :class="isDesktop && isJobsWorkbench ? 'flex-1 min-h-0' : ''">
              <router-view />
            </div>
          </div>
        </div>
      </main>
    </n-layout>
  </n-layout>

  <n-drawer v-if="!isDesktop" v-model:show="mobileMenuOpen" placement="left" :width="LAYOUT.mobileDrawerWidth">
    <n-drawer-content>
      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <AppLogo size="sm" />
          <n-tag v-if="versionTag" size="small" type="info" :bordered="false">{{ versionTag }}</n-tag>
        </div>

        <div class="space-y-2 rounded-2xl border border-[color:var(--app-border)] bg-[color:var(--app-surface)] p-4">
          <div class="text-xs app-text-muted">{{ scopePickerLabel }}</div>
          <n-select
            v-model:value="selectedScope"
            :options="scopeOptions"
            filterable
            @update:value="mobileMenuOpen = false"
          />
          <div class="text-xs app-text-muted">{{ scopePickerHint }}</div>
        </div>

        <n-menu
          :value="activePrimaryKey"
          :options="primaryMenuOptions"
          @update:value="
            (key) => {
              mobileMenuOpen = false
              navigatePrimary(key)
            }
          "
        />

        <div v-if="secondaryMenuOptions.length > 0">
          <div class="mb-2 text-xs uppercase tracking-[0.16em] app-text-muted">
            {{ t('nav.context') }}
          </div>
          <n-menu
            :value="activeSecondaryKey"
            :options="secondaryMenuOptions"
            @update:value="
              (key) => {
                mobileMenuOpen = false
                navigateSecondary(key)
              }
            "
          />
        </div>

        <div class="grid grid-cols-2 gap-2 border-t border-[color:var(--app-border)] pt-4">
          <n-button size="small" @click="mobileMenuOpen = false; openDocs()">{{ t('common.help') }}</n-button>
          <n-dropdown :options="languageOptions" trigger="click" @select="onSelectLanguage">
            <n-button size="small">{{ t('common.language') }}</n-button>
          </n-dropdown>
          <n-button size="small" @click="ui.toggleDarkMode()">{{ ui.darkMode ? t('common.light') : t('common.dark') }}</n-button>
          <n-button size="small" type="error" @click="mobileMenuOpen = false; onLogout()">{{ t('common.logout') }}</n-button>
        </div>
      </div>
    </n-drawer-content>
  </n-drawer>
</template>
