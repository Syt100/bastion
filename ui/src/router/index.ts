import { createRouter, createWebHistory } from 'vue-router'

import AppShell from '@/layouts/AppShell.vue'
import AgentsView from '@/views/AgentsView.vue'
import DashboardView from '@/views/DashboardView.vue'
import LoginView from '@/views/LoginView.vue'
import PlaceholderView from '@/views/PlaceholderView.vue'
import SetupView from '@/views/SetupView.vue'

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
        { path: 'jobs', component: PlaceholderView, props: { title: 'Jobs' } },
        { path: 'agents', component: AgentsView },
        { path: 'settings', component: PlaceholderView, props: { title: 'Settings' } },
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
