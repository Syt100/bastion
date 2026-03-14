import type { Component } from 'vue'
import { CloudOutline, ColorPaletteOutline, ConstructOutline, InformationCircleOutline, NotificationsOutline, OptionsOutline } from '@vicons/ionicons5'

export type SettingsNavItem = {
  key: string
  to: string
  titleKey: string
  descriptionKey?: string
  icon?: Component
  domain: 'integrations' | 'system'
  showInOverview: boolean
  showInSidebar: boolean
  order: number
}

export const SETTINGS_NAV_ITEMS: SettingsNavItem[] = [
  {
    key: 'appearance',
    to: '/system/appearance',
    titleKey: 'settings.menu.appearance',
    descriptionKey: 'settings.overview.appearanceDesc',
    icon: ColorPaletteOutline,
    domain: 'system',
    showInOverview: true,
    showInSidebar: true,
    order: 5,
  },
  {
    key: 'storage',
    to: '/integrations/storage',
    titleKey: 'settings.menu.storage',
    descriptionKey: 'settings.overview.storageDesc',
    icon: CloudOutline,
    domain: 'integrations',
    showInOverview: true,
    showInSidebar: true,
    order: 10,
  },
  {
    key: 'notifications',
    to: '/integrations/notifications',
    titleKey: 'settings.menu.notifications',
    descriptionKey: 'settings.overview.notificationsDesc',
    icon: NotificationsOutline,
    domain: 'integrations',
    showInOverview: true,
    showInSidebar: true,
    order: 20,
  },
  {
    key: 'maintenance',
    to: '/system/maintenance',
    titleKey: 'settings.menu.maintenance',
    descriptionKey: 'settings.overview.maintenanceDesc',
    icon: ConstructOutline,
    domain: 'system',
    showInOverview: true,
    showInSidebar: true,
    order: 30,
  },
  {
    key: 'runtime-config',
    to: '/system/runtime',
    titleKey: 'settings.menu.runtimeConfig',
    descriptionKey: 'settings.overview.runtimeConfigDesc',
    icon: OptionsOutline,
    domain: 'system',
    showInOverview: true,
    showInSidebar: true,
    order: 35,
  },
  {
    key: 'bulk-operations',
    to: '/system/bulk-operations',
    titleKey: 'settings.menu.bulkOperations',
    descriptionKey: 'settings.overview.bulkOperationsDesc',
    icon: ConstructOutline,
    domain: 'system',
    showInOverview: true,
    showInSidebar: true,
    order: 37,
  },
  {
    key: 'about',
    to: '/system/about',
    titleKey: 'settings.menu.about',
    descriptionKey: 'settings.overview.aboutDesc',
    icon: InformationCircleOutline,
    domain: 'system',
    showInOverview: true,
    showInSidebar: true,
    order: 40,
  },
]

export type SettingsOverviewItem = {
  key: string
  titleKey: string
  descriptionKey: string
  to: string
  icon: Component
}

export function getSettingsOverviewItems(): SettingsOverviewItem[] {
  return SETTINGS_NAV_ITEMS.filter((i) => i.showInOverview)
    .slice()
    .sort((a, b) => a.order - b.order)
    .map((i) => ({
      key: i.key,
      titleKey: i.titleKey,
      descriptionKey: i.descriptionKey ?? i.titleKey,
      to: i.to,
      icon: i.icon ?? CloudOutline,
    }))
}

export type SettingsSidebarItem = {
  key: string
  titleKey: string
  to: string
}

export function getSettingsSidebarItems(): SettingsSidebarItem[] {
  return SETTINGS_NAV_ITEMS.filter((i) => i.showInSidebar)
    .slice()
    .sort((a, b) => a.order - b.order)
    .map((i) => ({
      key: i.key,
      titleKey: i.titleKey,
      to: i.to,
    }))
}

export function getSettingsMenuRouteKeys(): string[] {
  const keys = new Set<string>()
  for (const item of getSettingsSidebarItems()) {
    keys.add(item.to)
  }
  return [...keys]
}

export function getIntegrationsSidebarItems(): SettingsSidebarItem[] {
  return SETTINGS_NAV_ITEMS.filter((i) => i.showInSidebar && i.domain === 'integrations')
    .slice()
    .sort((a, b) => a.order - b.order)
    .map((i) => ({
      key: i.key,
      titleKey: i.titleKey,
      to: i.to,
    }))
}

export function getSystemSidebarItems(): SettingsSidebarItem[] {
  return SETTINGS_NAV_ITEMS.filter((i) => i.showInSidebar && i.domain === 'system')
    .slice()
    .sort((a, b) => a.order - b.order)
    .map((i) => ({
      key: i.key,
      titleKey: i.titleKey,
      to: i.to,
    }))
}
