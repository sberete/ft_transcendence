import { createI18n } from 'vue-i18n'
import fr from './locales/fr.json'
import en from './locales/en.json'

const STORAGE_KEY = 'locale'
const SUPPORTED = ['fr', 'en']

function initialLocale() {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (SUPPORTED.includes(stored)) return stored
  } catch {
    // localStorage indisponible (navigation privee stricte...) : on repart
    // simplement sur le francais, langue par defaut du projet.
  }
  return 'fr'
}

export const i18n = createI18n({
  legacy: false,
  locale: initialLocale(),
  fallbackLocale: 'fr',
  messages: { fr, en },
})

// Garde l'attribut lang en phase avec la langue active : pas seulement le
// texte visible, pertinent pour l'accessibilite (lecteurs d'ecran, etc.) -
// une fois au chargement, puis a chaque changement via setLocale().
document.documentElement.lang = i18n.global.locale.value

export function setLocale(locale) {
  if (!SUPPORTED.includes(locale)) return
  i18n.global.locale.value = locale
  document.documentElement.lang = locale
  try {
    localStorage.setItem(STORAGE_KEY, locale)
  } catch {
    // idem : pas grave si la preference ne survit pas a la session.
  }
}
