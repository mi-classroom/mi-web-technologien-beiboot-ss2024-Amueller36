import './assets/main.css'

import { createApp, type App as VueApp } from 'vue'
import App from './App.vue'
import router from './router'
const app : VueApp = createApp(App)

app.use(router)

app.mount('#app')
