import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import './styles/main.css'
import { pinia } from './pinia'
import { i18n } from './i18n'
import { useAuthStore } from '@/stores/auth'

const app = createApp(App)

app.use(pinia)
app.use(router)
app.use(i18n)

if (typeof window !== 'undefined') {
  window.addEventListener('bastion:unauthorized', () => {
    const auth = useAuthStore(pinia)
    auth.status = 'anonymous'
    auth.csrfToken = null
    if (router.currentRoute.value.path !== '/login') {
      void router.push('/login')
    }
  })
}

app.mount('#app')
