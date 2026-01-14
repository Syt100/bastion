import { createRouter, createWebHistory } from 'vue-router'

const AppShell = () => import('@/layouts/AppShell.vue')
const AgentsView = () => import('@/views/AgentsView.vue')
const DashboardView = () => import('@/views/DashboardView.vue')
const JobsView = () => import('@/views/JobsView.vue')
const LoginView = () => import('@/views/LoginView.vue')
const SettingsShellView = () => import('@/views/settings/SettingsShellView.vue')
const SettingsIndexView = () => import('@/views/settings/SettingsIndexView.vue')
const SettingsStorageView = () => import('@/views/settings/SettingsStorageView.vue')
const AboutView = () => import('@/views/settings/AboutView.vue')
const HubRuntimeConfigView = () => import('@/views/settings/HubRuntimeConfigView.vue')
const BulkOperationsView = () => import('@/views/settings/BulkOperationsView.vue')
const NotificationsShellView = () => import('@/views/settings/notifications/NotificationsShellView.vue')
const NotificationsIndexView = () => import('@/views/settings/notifications/NotificationsIndexView.vue')
const NotificationsChannelsView = () => import('@/views/settings/notifications/NotificationsChannelsView.vue')
const NotificationsDestinationsView = () => import('@/views/settings/notifications/NotificationsDestinationsView.vue')
const NotificationsTemplatesView = () => import('@/views/settings/notifications/NotificationsTemplatesView.vue')
const NotificationsQueueView = () => import('@/views/settings/notifications/NotificationsQueueView.vue')
const MaintenanceCleanupView = () => import('@/views/settings/maintenance/MaintenanceCleanupView.vue')
const SetupView = () => import('@/views/SetupView.vue')

import { pinia } from '@/pinia'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/setup', component: SetupView, meta: { titleKey: 'auth.initTitle' } },
    { path: '/login', component: LoginView, meta: { titleKey: 'auth.signIn' } },
    {
      path: '/',
      component: AppShell,
      children: [
        { path: '', component: DashboardView, meta: { titleKey: 'dashboard.title' } },
        { path: 'jobs', redirect: '/n/hub/jobs' },
        {
          path: 'n/:nodeId',
          children: [
            { path: '', redirect: (to) => ({ path: `/n/${encodeURIComponent(String(to.params.nodeId))}/jobs` }) },
            { path: 'jobs', component: JobsView, meta: { titleKey: 'jobs.title' } },
            {
              path: 'settings',
              component: SettingsShellView,
              meta: { titleKey: 'settings.title', mobileTopBar: { titleKey: 'settings.title', backTo: null } },
              children: [
                {
                  path: 'storage',
                  component: SettingsStorageView,
                  meta: { titleKey: 'settings.menu.storage', mobileTopBar: { titleKey: 'settings.menu.storage', backTo: null } },
                },
              ],
            },
          ],
        },
        { path: 'agents', component: AgentsView, meta: { titleKey: 'agents.title' } },
        {
          path: 'settings',
          component: SettingsShellView,
          meta: { titleKey: 'settings.title', mobileTopBar: { titleKey: 'settings.title', backTo: null } },
          children: [
            { path: '', component: SettingsIndexView, meta: { titleKey: 'settings.title', mobileTopBar: { titleKey: 'settings.title', backTo: null } } },
            {
              path: 'about',
              component: AboutView,
              meta: { titleKey: 'settings.menu.about', mobileTopBar: { titleKey: 'settings.menu.about', backTo: '/settings' } },
            },
            {
              path: 'hub-runtime-config',
              component: HubRuntimeConfigView,
              meta: {
                titleKey: 'settings.menu.runtimeConfig',
                mobileTopBar: { titleKey: 'settings.menu.runtimeConfig', backTo: '/settings' },
              },
            },
            {
              path: 'bulk-operations',
              component: BulkOperationsView,
              meta: {
                titleKey: 'settings.menu.bulkOperations',
                mobileTopBar: { titleKey: 'settings.menu.bulkOperations', backTo: '/settings' },
              },
            },
            {
              path: 'storage',
              redirect: '/n/hub/settings/storage',
            },
            {
              path: 'notifications',
              component: NotificationsShellView,
              meta: { titleKey: 'settings.menu.notifications', mobileTopBar: { titleKey: 'settings.menu.notifications', backTo: '/settings' } },
              children: [
                {
                  path: '',
                  component: NotificationsIndexView,
                  meta: { titleKey: 'settings.menu.notifications', mobileTopBar: { titleKey: 'settings.menu.notifications', backTo: '/settings' } },
                },
                {
                  path: 'channels',
                  component: NotificationsChannelsView,
                  meta: { titleKey: 'settings.notifications.tabs.channels', mobileTopBar: { titleKey: 'settings.notifications.tabs.channels', backTo: '/settings/notifications' } },
                },
                {
                  path: 'destinations',
                  component: NotificationsDestinationsView,
                  meta: { titleKey: 'settings.notifications.tabs.destinations', mobileTopBar: { titleKey: 'settings.notifications.tabs.destinations', backTo: '/settings/notifications' } },
                },
                {
                  path: 'templates',
                  component: NotificationsTemplatesView,
                  meta: { titleKey: 'settings.notifications.tabs.templates', mobileTopBar: { titleKey: 'settings.notifications.tabs.templates', backTo: '/settings/notifications' } },
                },
                {
                  path: 'queue',
                  component: NotificationsQueueView,
                  meta: { titleKey: 'settings.notifications.tabs.queue', mobileTopBar: { titleKey: 'settings.notifications.tabs.queue', backTo: '/settings/notifications' } },
                },
              ],
            },
            {
              path: 'maintenance',
              redirect: '/settings/maintenance/cleanup',
            },
            {
              path: 'maintenance/cleanup',
              component: MaintenanceCleanupView,
              meta: { titleKey: 'settings.maintenance.cleanup.title', mobileTopBar: { titleKey: 'settings.maintenance.cleanup.title', backTo: '/settings' } },
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
