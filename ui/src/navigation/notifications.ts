import type { Component } from 'vue'
import { DocumentTextOutline, ListOutline, OptionsOutline, PinOutline } from '@vicons/ionicons5'

export type NotificationsTabKey = 'channels' | 'destinations' | 'templates' | 'queue'

export function isNotificationsTabKey(value: string): value is NotificationsTabKey {
  return value === 'channels' || value === 'destinations' || value === 'templates' || value === 'queue'
}

export const DEFAULT_NOTIFICATIONS_TAB_KEY: NotificationsTabKey = 'destinations'

export type NotificationsNavItem = {
  key: NotificationsTabKey
  to: string
  titleKey: string
  descriptionKey: string
  icon: Component
  order: number
}

export const NOTIFICATIONS_NAV_ITEMS: NotificationsNavItem[] = [
  {
    key: 'channels',
    titleKey: 'settings.notifications.tabs.channels',
    descriptionKey: 'settings.notifications.overview.channelsDesc',
    to: '/settings/notifications/channels',
    icon: OptionsOutline,
    order: 10,
  },
  {
    key: 'destinations',
    titleKey: 'settings.notifications.tabs.destinations',
    descriptionKey: 'settings.notifications.overview.destinationsDesc',
    to: '/settings/notifications/destinations',
    icon: PinOutline,
    order: 20,
  },
  {
    key: 'templates',
    titleKey: 'settings.notifications.tabs.templates',
    descriptionKey: 'settings.notifications.overview.templatesDesc',
    to: '/settings/notifications/templates',
    icon: DocumentTextOutline,
    order: 30,
  },
  {
    key: 'queue',
    titleKey: 'settings.notifications.tabs.queue',
    descriptionKey: 'settings.notifications.overview.queueDesc',
    to: '/settings/notifications/queue',
    icon: ListOutline,
    order: 40,
  },
]

export function getNotificationsNavItems(): NotificationsNavItem[] {
  return NOTIFICATIONS_NAV_ITEMS.slice().sort((a, b) => a.order - b.order)
}

