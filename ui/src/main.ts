import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import './styles/main.css'
import { pinia } from './pinia'
import { i18n } from './i18n'

const app = createApp(App)

app.use(pinia)
app.use(router)
app.use(i18n)

app.mount('#app')
