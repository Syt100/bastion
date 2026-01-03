import { createRouter, createWebHistory } from 'vue-router'

const AppShell = () => import('@/layouts/AppShell.vue')
const AgentsView = () => import('@/views/AgentsView.vue')
const DashboardView = () => import('@/views/DashboardView.vue')
const JobsView = () => import('@/views/JobsView.vue')
const LoginView = () => import('@/views/LoginView.vue')
const SettingsShellView = () => import('@/views/settings/SettingsShellView.vue')
const SettingsIndexView = () => import('@/views/settings/SettingsIndexView.vue')
const SettingsStorageView = () => import('@/views/settings/SettingsStorageView.vue')
const NotificationsShellView = () => import('@/views/settings/notifications/NotificationsShellView.vue')
const NotificationsIndexView = () => import('@/views/settings/notifications/NotificationsIndexView.vue')
const NotificationsChannelsView = () => import('@/views/settings/notifications/NotificationsChannelsView.vue')
const NotificationsDestinationsView = () => import('@/views/settings/notifications/NotificationsDestinationsView.vue')
const NotificationsTemplatesView = () => import('@/views/settings/notifications/NotificationsTemplatesView.vue')
const NotificationsQueueView = () => import('@/views/settings/notifications/NotificationsQueueView.vue')
const SetupView = () => import('@/views/SetupView.vue')

import { pinia } from '@/pinia'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/setup', component: SetupView },
    { path: '/login', component: LoginView },
    {
      path: '/',
      component: AppShell,
      children: [
        { path: '', component: DashboardView },
        { path: 'jobs', component: JobsView },
        { path: 'agents', component: AgentsView },
        {
          path: 'settings',
          component: SettingsShellView,
          meta: { mobileTopBar: { titleKey: 'settings.title', backTo: null } },
          children: [
            { path: '', component: SettingsIndexView, meta: { mobileTopBar: { titleKey: 'settings.title', backTo: null } } },
            {
              path: 'storage',
              component: SettingsStorageView,
              meta: { mobileTopBar: { titleKey: 'settings.menu.storage', backTo: '/settings' } },
            },
            {
              path: 'notifications',
              component: NotificationsShellView,
              meta: { mobileTopBar: { titleKey: 'settings.menu.notifications', backTo: '/settings' } },
              children: [
                {
                  path: '',
                  component: NotificationsIndexView,
                  meta: { mobileTopBar: { titleKey: 'settings.menu.notifications', backTo: '/settings' } },
                },
                {
                  path: 'channels',
                  component: NotificationsChannelsView,
                  meta: { mobileTopBar: { titleKey: 'settings.notifications.tabs.channels', backTo: '/settings/notifications' } },
                },
                {
                  path: 'destinations',
                  component: NotificationsDestinationsView,
                  meta: { mobileTopBar: { titleKey: 'settings.notifications.tabs.destinations', backTo: '/settings/notifications' } },
                },
                {
                  path: 'templates',
                  component: NotificationsTemplatesView,
                  meta: { mobileTopBar: { titleKey: 'settings.notifications.tabs.templates', backTo: '/settings/notifications' } },
                },
                {
                  path: 'queue',
                  component: NotificationsQueueView,
                  meta: { mobileTopBar: { titleKey: 'settings.notifications.tabs.queue', backTo: '/settings/notifications' } },
                },
              ],
            },
          ],
        },
      ],
      meta: { requiresAuth: true },
    },
  ],
})

router.beforeEach(async (to) => {
  if (!to.meta.requiresAuth) return true

  const auth = useAuthStore(pinia)
  if (auth.status === 'unknown') {
    await auth.refreshSession()
  }

  if (!auth.isAuthenticated) {
    return { path: '/login' }
  }

  return true
})

export default router
