import './assets/app.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { registerSW } from 'virtual:pwa-register'

import App from './App.vue'
import router from './router'
import i18n from './i18n'

// Look for a new service worker every time the app comes back to the foreground.
// An installed PWA on Android resumes from the recents list without a page load,
// so the check that runs at registration time would never run again and the phone
// keeps the old build for days. `registerType: 'autoUpdate'` reloads the page by
// itself once the new worker activates.
registerSW({
  immediate: true,
  onRegisteredSW(_swUrl, registration) {
    if (!registration) return
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible') registration.update()
    })
  },
})

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)

app.mount('#app')
