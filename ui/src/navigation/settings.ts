import type { Component } from 'vue'
import { CloudOutline, ConstructOutline, InformationCircleOutline, NotificationsOutline } from '@vicons/ionicons5'

export type SettingsNavItem = {
  key: string
  to: string
  titleKey: string
  descriptionKey?: string
  icon?: Component
  showInOverview: boolean
  showInSidebar: boolean
  order: number
}

export const SETTINGS_NAV_ITEMS: SettingsNavItem[] = [
  {
    key: 'overview',
    to: '/settings',
    titleKey: 'settings.menu.overview',
    showInOverview: false,
    showInSidebar: true,
    order: 0,
  },
  {
    key: 'storage',
    to: '/settings/storage',
    titleKey: 'settings.menu.storage',
    descriptionKey: 'settings.overview.storageDesc',
    icon: CloudOutline,
    showInOverview: true,
    showInSidebar: true,
    order: 10,
  },
  {
    key: 'notifications',
    to: '/settings/notifications',
    titleKey: 'settings.menu.notifications',
    descriptionKey: 'settings.overview.notificationsDesc',
    icon: NotificationsOutline,
    showInOverview: true,
    showInSidebar: true,
    order: 20,
  },
  {
    key: 'maintenance',
    to: '/settings/maintenance',
    titleKey: 'settings.menu.maintenance',
    descriptionKey: 'settings.overview.maintenanceDesc',
    icon: ConstructOutline,
    showInOverview: true,
    showInSidebar: true,
    order: 30,
  },
  {
    key: 'about',
    to: '/settings/about',
    titleKey: 'settings.menu.about',
    descriptionKey: 'settings.overview.aboutDesc',
    icon: InformationCircleOutline,
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

