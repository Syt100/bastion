import { createRouter, createWebHistory } from 'vue-router'

const AppShell = () => import('@/layouts/AppShell.vue')
const AgentsView = () => import('@/views/AgentsView.vue')
const DashboardView = () => import('@/views/DashboardView.vue')
const JobsWorkspaceShellView = () => import('@/views/jobs/JobsWorkspaceShellView.vue')
const JobWorkspaceView = () => import('@/views/jobs/JobWorkspaceView.vue')
const JobOverviewSectionView = () => import('@/views/jobs/JobOverviewSectionView.vue')
const JobHistorySectionView = () => import('@/views/jobs/JobHistorySectionView.vue')
const JobDataSectionView = () => import('@/views/jobs/JobDataSectionView.vue')
const LoginView = () => import('@/views/LoginView.vue')
const SettingsShellView = () => import('@/views/settings/SettingsShellView.vue')
const SettingsIndexView = () => import('@/views/settings/SettingsIndexView.vue')
const AppearanceView = () => import('@/views/settings/AppearanceView.vue')
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

const EmptyView = { render: () => null }

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
            {
              path: 'jobs',
              component: JobsWorkspaceShellView,
              meta: { titleKey: 'jobs.title' },
              children: [
                {
                  path: ':jobId',
                  component: JobWorkspaceView,
                  meta: { titleKey: 'jobs.detail.title' },
                  children: [
                    { path: '', redirect: 'overview' },
                    {
                      path: 'overview',
                      component: JobOverviewSectionView,
                      meta: { titleKey: 'jobs.workspace.sections.overview' },
                      children: [{ path: 'runs/:runId', component: EmptyView, meta: { titleKey: 'runs.title' } }],
                    },
                    {
                      path: 'history',
                      component: JobHistorySectionView,
                      meta: { titleKey: 'jobs.workspace.sections.history' },
                      children: [{ path: 'runs/:runId', component: EmptyView, meta: { titleKey: 'runs.title' } }],
                    },
                    {
                      path: 'data',
                      component: JobDataSectionView,
                      meta: { titleKey: 'jobs.workspace.sections.data' },
                      children: [{ path: 'runs/:runId', component: EmptyView, meta: { titleKey: 'runs.title' } }],
                    },
                  ],
                },
              ],
            },
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
              path: 'appearance',
              component: AppearanceView,
              meta: {
                titleKey: 'settings.menu.appearance',
                mobileTopBar: { titleKey: 'settings.menu.appearance', backTo: '/settings' },
              },
            },
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
