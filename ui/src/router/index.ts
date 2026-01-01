import { createRouter, createWebHistory } from 'vue-router'

const AppShell = () => import('@/layouts/AppShell.vue')
const AgentsView = () => import('@/views/AgentsView.vue')
const DashboardView = () => import('@/views/DashboardView.vue')
const JobsView = () => import('@/views/JobsView.vue')
const LoginView = () => import('@/views/LoginView.vue')
const SettingsView = () => import('@/views/SettingsView.vue')
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
        { path: 'settings', component: SettingsView },
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
