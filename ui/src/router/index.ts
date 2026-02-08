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

function mobileMeta(titleKey: string, backTo: string | null) {
  return { titleKey, mobileTopBar: { titleKey, backTo } }
}

const settingsRootMeta = mobileMeta('settings.title', null)
const settingsSectionMeta = (titleKey: string) => mobileMeta(titleKey, '/settings')
const settingsNotificationsMeta = (titleKey: string) => mobileMeta(titleKey, '/settings/notifications')

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
              meta: settingsRootMeta,
              children: [
                {
                  path: 'storage',
                  component: SettingsStorageView,
                  meta: mobileMeta('settings.menu.storage', null),
                },
              ],
            },
          ],
        },
        { path: 'agents', component: AgentsView, meta: { titleKey: 'agents.title' } },
        {
          path: 'settings',
          component: SettingsShellView,
          meta: settingsRootMeta,
          children: [
            { path: '', component: SettingsIndexView, meta: settingsRootMeta },
            {
              path: 'appearance',
              component: AppearanceView,
              meta: settingsSectionMeta('settings.menu.appearance'),
            },
            {
              path: 'about',
              component: AboutView,
              meta: settingsSectionMeta('settings.menu.about'),
            },
            {
              path: 'hub-runtime-config',
              component: HubRuntimeConfigView,
              meta: settingsSectionMeta('settings.menu.runtimeConfig'),
            },
            {
              path: 'bulk-operations',
              component: BulkOperationsView,
              meta: settingsSectionMeta('settings.menu.bulkOperations'),
            },
            {
              path: 'storage',
              redirect: '/n/hub/settings/storage',
            },
            {
              path: 'notifications',
              component: NotificationsShellView,
              meta: settingsSectionMeta('settings.menu.notifications'),
              children: [
                {
                  path: '',
                  component: NotificationsIndexView,
                  meta: settingsSectionMeta('settings.menu.notifications'),
                },
                {
                  path: 'channels',
                  component: NotificationsChannelsView,
                  meta: settingsNotificationsMeta('settings.notifications.tabs.channels'),
                },
                {
                  path: 'destinations',
                  component: NotificationsDestinationsView,
                  meta: settingsNotificationsMeta('settings.notifications.tabs.destinations'),
                },
                {
                  path: 'templates',
                  component: NotificationsTemplatesView,
                  meta: settingsNotificationsMeta('settings.notifications.tabs.templates'),
                },
                {
                  path: 'queue',
                  component: NotificationsQueueView,
                  meta: settingsNotificationsMeta('settings.notifications.tabs.queue'),
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
              meta: settingsSectionMeta('settings.maintenance.cleanup.title'),
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
