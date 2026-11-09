// Global vue-i18n instance (French by default, English as fallback): the
// chosen locale is persisted in localStorage and mirrored onto <html lang>
// so it stays correct for screen readers, not just visually.
import { createI18n } from 'vue-i18n'
import fr from './locales/fr.json'
import en from './locales/en.json'
import es from './locales/es.json'

const STORAGE_KEY = 'locale'
const SUPPORTED = ['fr', 'en', 'es']

// Locale to start with: the saved one if it's valid, otherwise French.
function initialLocale() {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (SUPPORTED.includes(stored)) return stored
  } catch {
    // localStorage unavailable (strict private browsing...): just fall
    // back to French, the project's default language.
  }
  return 'fr'
}

export const i18n = createI18n({
  legacy: false,
  locale: initialLocale(),
  fallbackLocale: 'fr',
  messages: { fr, en, es },
})

// Keep the lang attribute in sync with the active locale from the start.
document.documentElement.lang = i18n.global.locale.value

// Switches the active locale, syncs <html lang>, and persists the choice.
export function setLocale(locale) {
  if (!SUPPORTED.includes(locale)) return
  i18n.global.locale.value = locale
  document.documentElement.lang = locale
  try {
    localStorage.setItem(STORAGE_KEY, locale)
  } catch {
    // same as above: fine if the preference doesn't survive the session.
  }
}
