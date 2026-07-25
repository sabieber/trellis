import { createI18n } from 'vue-i18n'
import moment from 'moment'
import 'moment/dist/locale/de'

import en from './locales/en.json'
import de from './locales/de.json'

export type Locale = 'en' | 'de'

function initialLocale(): Locale {
  const stored = localStorage.getItem('locale')
  if (stored === 'en' || stored === 'de') return stored
  return navigator.language.startsWith('de') ? 'de' : 'en'
}

const i18n = createI18n({
  legacy: false,
  locale: initialLocale(),
  fallbackLocale: 'en',
  messages: { en, de },
})

// Keep moment's date formatting in the same language as the UI.
export function setLocale(locale: Locale) {
  i18n.global.locale.value = locale
  moment.locale(locale)
  localStorage.setItem('locale', locale)
}

moment.locale(i18n.global.locale.value)

export default i18n
