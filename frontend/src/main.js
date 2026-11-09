// App entry point: mounts the root component with the router (navigation)
// and i18n (FR/EN) plugins already registered.
import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import router from './router'
import { i18n } from './i18n'

createApp(App).use(router).use(i18n).mount('#app')
