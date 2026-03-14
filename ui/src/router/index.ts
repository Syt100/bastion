import { createRouter, createWebHistory } from 'vue-router'

const AppShell = () => import('@/layouts/AppShell.vue')
const AgentsView = () => import('@/views/AgentsView.vue')
const CommandCenterView = () => import('@/views/CommandCenterView.vue')
const JobsLandingView = () => import('@/views/JobsLandingView.vue')
const JobsWorkspaceShellView = () => import('@/views/jobs/JobsWorkspaceShellView.vue')
const JobWorkspaceView = () => import('@/views/jobs/JobWorkspaceView.vue')
const JobOverviewSectionView = () => import('@/views/jobs/JobOverviewSectionView.vue')
const JobHistorySectionView = () => import('@/views/jobs/JobHistorySectionView.vue')
const JobDataSectionView = () => import('@/views/jobs/JobDataSectionView.vue')
const LoginView = () => import('@/views/LoginView.vue')
const SettingsShellView = () => import('@/views/settings/SettingsShellView.vue')
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
const RunDetailRouteView = () => import('@/views/RunDetailRouteView.vue')
const RunsView = () => import('@/views/RunsView.vue')
const SetupView = () => import('@/views/SetupView.vue')

import { pinia } from '@/pinia'
import { useAuthStore } from '@/stores/auth'
import { scopeFromNodeId } from '@/lib/scope'

const EmptyView = { render: () => null }

type ShellScopeMode = 'collection' | 'detail' | 'none' | 'legacy-node'

function routeMeta(
  titleKey: string,
  options: {
    primaryNav?: string
    secondaryNav?: string
    scopeMode?: ShellScopeMode
    shellTitleKey?: string
    shellSubtitleKey?: string
    mobileTopBar?: { titleKey: string; backTo: string | null }
  } = {},
) {
  return {
    titleKey,
    ...options,
  }
}

function mobileMeta(
  titleKey: string,
  backTo: string | null,
  options: {
    primaryNav?: string
    secondaryNav?: string
    scopeMode?: ShellScopeMode
    shellTitleKey?: string
    shellSubtitleKey?: string
  } = {},
) {
  return routeMeta(titleKey, {
    ...options,
    mobileTopBar: { titleKey, backTo },
  })
}

const integrationsShellMeta = {
  primaryNav: 'integrations',
  shellTitleKey: 'integrations.title',
  shellSubtitleKey: 'integrations.subtitle',
} as const

const systemShellMeta = {
  primaryNav: 'system',
  shellTitleKey: 'system.title',
  shellSubtitleKey: 'system.subtitle',
} as const

function legacyNodeScopeRedirect(nodeId: string) {
  return {
    path: '/jobs',
    query: { scope: scopeFromNodeId(nodeId) },
  }
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/setup', component: SetupView, meta: routeMeta('auth.initTitle') },
    { path: '/login', component: LoginView, meta: routeMeta('auth.signIn') },
    {
      path: '/',
      component: AppShell,
      meta: { requiresAuth: true },
      children: [
        {
          path: '',
          component: CommandCenterView,
          meta: routeMeta('commandCenter.title', {
            primaryNav: 'command-center',
            scopeMode: 'collection',
          }),
        },
        {
          path: 'jobs',
          component: JobsLandingView,
          meta: routeMeta('jobs.title', {
            primaryNav: 'jobs',
            scopeMode: 'collection',
          }),
        },
        {
          path: 'runs',
          component: RunsView,
          meta: routeMeta('runs.title', {
            primaryNav: 'runs',
            scopeMode: 'collection',
          }),
        },
        {
          path: 'runs/:runId',
          component: RunDetailRouteView,
          meta: routeMeta('runs.detail.pageTitle', {
            primaryNav: 'runs',
            scopeMode: 'detail',
          }),
        },
        {
          path: 'fleet',
          component: AgentsView,
          meta: routeMeta('fleet.title', {
            primaryNav: 'fleet',
            scopeMode: 'none',
          }),
        },
        {
          path: 'integrations',
          component: SettingsShellView,
          meta: routeMeta('integrations.title', integrationsShellMeta),
          children: [
            { path: '', redirect: '/integrations/storage' },
            {
              path: 'storage',
              component: SettingsStorageView,
              meta: mobileMeta('settings.menu.storage', null, {
                ...integrationsShellMeta,
                secondaryNav: 'storage',
                scopeMode: 'collection',
              }),
            },
            {
              path: 'notifications',
              component: NotificationsShellView,
              meta: mobileMeta('settings.menu.notifications', null, {
                ...integrationsShellMeta,
                secondaryNav: 'notifications',
                scopeMode: 'none',
              }),
              children: [
                {
                  path: '',
                  component: NotificationsIndexView,
                  meta: mobileMeta('settings.menu.notifications', null, {
                    ...integrationsShellMeta,
                    secondaryNav: 'notifications',
                    scopeMode: 'none',
                  }),
                },
                {
                  path: 'channels',
                  component: NotificationsChannelsView,
                  meta: mobileMeta('settings.notifications.tabs.channels', '/integrations/notifications', {
                    ...integrationsShellMeta,
                    secondaryNav: 'notifications',
                    scopeMode: 'none',
                  }),
                },
                {
                  path: 'destinations',
                  component: NotificationsDestinationsView,
                  meta: mobileMeta('settings.notifications.tabs.destinations', '/integrations/notifications', {
                    ...integrationsShellMeta,
                    secondaryNav: 'notifications',
                    scopeMode: 'none',
                  }),
                },
                {
                  path: 'templates',
                  component: NotificationsTemplatesView,
                  meta: mobileMeta('settings.notifications.tabs.templates', '/integrations/notifications', {
                    ...integrationsShellMeta,
                    secondaryNav: 'notifications',
                    scopeMode: 'none',
                  }),
                },
                {
                  path: 'queue',
                  component: NotificationsQueueView,
                  meta: mobileMeta('settings.notifications.tabs.queue', '/integrations/notifications', {
                    ...integrationsShellMeta,
                    secondaryNav: 'notifications',
                    scopeMode: 'none',
                  }),
                },
              ],
            },
          ],
        },
        {
          path: 'system',
          component: SettingsShellView,
          meta: routeMeta('system.title', systemShellMeta),
          children: [
            { path: '', redirect: '/system/runtime' },
            {
              path: 'runtime',
              component: HubRuntimeConfigView,
              meta: mobileMeta('settings.menu.runtimeConfig', null, {
                ...systemShellMeta,
                secondaryNav: 'runtime',
                scopeMode: 'none',
              }),
            },
            {
              path: 'bulk-operations',
              component: BulkOperationsView,
              meta: mobileMeta('settings.menu.bulkOperations', null, {
                ...systemShellMeta,
                secondaryNav: 'bulk-operations',
                scopeMode: 'none',
              }),
            },
            {
              path: 'appearance',
              component: AppearanceView,
              meta: mobileMeta('settings.menu.appearance', null, {
                ...systemShellMeta,
                secondaryNav: 'appearance',
                scopeMode: 'none',
              }),
            },
            {
              path: 'about',
              component: AboutView,
              meta: mobileMeta('settings.menu.about', null, {
                ...systemShellMeta,
                secondaryNav: 'about',
                scopeMode: 'none',
              }),
            },
            {
              path: 'maintenance',
              redirect: '/system/maintenance/cleanup',
            },
            {
              path: 'maintenance/cleanup',
              component: MaintenanceCleanupView,
              meta: mobileMeta('settings.maintenance.cleanup.title', null, {
                ...systemShellMeta,
                secondaryNav: 'maintenance',
                scopeMode: 'none',
              }),
            },
          ],
        },

        // Temporary canonical aliases while remaining views migrate.
        {
          path: 'agents',
          redirect: (to) => ({ path: '/fleet', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings',
          redirect: (to) => ({ path: '/integrations/storage', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings/storage',
          redirect: (to) => ({ path: '/integrations/storage', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings/notifications',
          redirect: (to) => ({ path: '/integrations/notifications', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings/notifications/:tab',
          redirect: (to) => ({
            path: `/integrations/notifications/${String(to.params.tab)}`,
            query: to.query,
            hash: to.hash,
          }),
        },
        {
          path: 'settings/hub-runtime-config',
          redirect: (to) => ({ path: '/system/runtime', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings/bulk-operations',
          redirect: (to) => ({ path: '/system/bulk-operations', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings/appearance',
          redirect: (to) => ({ path: '/system/appearance', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings/about',
          redirect: (to) => ({ path: '/system/about', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings/maintenance',
          redirect: (to) => ({ path: '/system/maintenance/cleanup', query: to.query, hash: to.hash }),
        },
        {
          path: 'settings/maintenance/cleanup',
          redirect: (to) => ({ path: '/system/maintenance/cleanup', query: to.query, hash: to.hash }),
        },

        {
          path: 'n/:nodeId',
          children: [
            {
              path: '',
              redirect: (to) => legacyNodeScopeRedirect(String(to.params.nodeId)),
            },
            {
              path: 'jobs',
              component: JobsWorkspaceShellView,
              meta: routeMeta('jobs.title', {
                primaryNav: 'jobs',
                scopeMode: 'legacy-node',
              }),
              children: [
                {
                  path: ':jobId',
                  component: JobWorkspaceView,
                  meta: routeMeta('jobs.detail.title', {
                    primaryNav: 'jobs',
                    scopeMode: 'legacy-node',
                  }),
                  children: [
                    { path: '', redirect: 'overview' },
                    {
                      path: 'overview',
                      component: JobOverviewSectionView,
                      meta: routeMeta('jobs.workspace.sections.overview', {
                        primaryNav: 'jobs',
                        scopeMode: 'legacy-node',
                      }),
                      children: [{ path: 'runs/:runId', component: EmptyView, meta: routeMeta('runs.title', { primaryNav: 'jobs', scopeMode: 'legacy-node' }) }],
                    },
                    {
                      path: 'history',
                      component: JobHistorySectionView,
                      meta: routeMeta('jobs.workspace.sections.history', {
                        primaryNav: 'jobs',
                        scopeMode: 'legacy-node',
                      }),
                      children: [{ path: 'runs/:runId', component: EmptyView, meta: routeMeta('runs.title', { primaryNav: 'jobs', scopeMode: 'legacy-node' }) }],
                    },
                    {
                      path: 'data',
                      component: JobDataSectionView,
                      meta: routeMeta('jobs.workspace.sections.data', {
                        primaryNav: 'jobs',
                        scopeMode: 'legacy-node',
                      }),
                      children: [{ path: 'runs/:runId', component: EmptyView, meta: routeMeta('runs.title', { primaryNav: 'jobs', scopeMode: 'legacy-node' }) }],
                    },
                  ],
                },
              ],
            },
            {
              path: 'settings',
              component: SettingsShellView,
              meta: routeMeta('integrations.title', {
                ...integrationsShellMeta,
              }),
              children: [
                {
                  path: 'storage',
                  component: SettingsStorageView,
                  meta: mobileMeta('settings.menu.storage', null, {
                    ...integrationsShellMeta,
                    secondaryNav: 'storage',
                    scopeMode: 'legacy-node',
                  }),
                },
              ],
            },
          ],
        },
      ],
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
