import type { Component } from 'vue'
import {
  ArchiveOutline,
  ConstructOutline,
  HomeOutline,
  NotificationsOutline,
  PeopleOutline,
  SettingsOutline,
} from '@vicons/ionicons5'

import type { SettingsSidebarItem } from '@/navigation/settings'
import { getIntegrationsSidebarItems, getSystemSidebarItems } from '@/navigation/settings'

export type ShellPrimaryNavKey =
  | 'command-center'
  | 'jobs'
  | 'runs'
  | 'fleet'
  | 'integrations'
  | 'system'

export type ShellNavItem = {
  key: ShellPrimaryNavKey
  to: string
  titleKey: string
  icon: Component
}

export const PRIMARY_NAV_ITEMS: ShellNavItem[] = [
  { key: 'command-center', to: '/', titleKey: 'nav.commandCenter', icon: HomeOutline },
  { key: 'jobs', to: '/jobs', titleKey: 'nav.jobs', icon: ArchiveOutline },
  { key: 'runs', to: '/runs', titleKey: 'nav.runs', icon: ConstructOutline },
  { key: 'fleet', to: '/fleet', titleKey: 'nav.fleet', icon: PeopleOutline },
  { key: 'integrations', to: '/integrations', titleKey: 'nav.integrations', icon: NotificationsOutline },
  { key: 'system', to: '/system', titleKey: 'nav.system', icon: SettingsOutline },
]

export function getSecondaryNavItems(primaryNav: ShellPrimaryNavKey | null): SettingsSidebarItem[] {
  if (primaryNav === 'integrations') return getIntegrationsSidebarItems()
  if (primaryNav === 'system') return getSystemSidebarItems()
  return []
}
