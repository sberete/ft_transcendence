// Communication entre la fenetre popup de connexion et la page qui l'a ouverte.
//
// On n'utilise pas window.opener : les pages de Google/GitHub envoient un
// en-tete (Cross-Origin-Opener-Policy) qui coupe ce lien. BroadcastChannel,
// lui, relie tous les onglets de NOTRE origine, et seulement ceux-la.
export const OAUTH_CHANNEL = 'syncwatch-oauth'

// Le texte affiche ne vient jamais de l'URL : on passe par cette table.
export const OAUTH_ERRORS = {
  oauth: 'La connexion avec ce service a échoué. Réessayez.',
  oauth_unavailable: "Ce mode de connexion n'est pas encore disponible. Utilisez votre mot de passe.",
}

// Un token de session fait exactement 64 caracteres hexadecimaux.
// On refuse tout le reste plutot que de stocker une valeur arbitraire.
export function isSessionToken(value) {
  return typeof value === 'string' && /^[0-9a-f]{64}$/.test(value)
}

// Ramene n'importe quelle valeur a un code d'erreur connu.
export function oauthErrorCode(value) {
  return Object.hasOwn(OAUTH_ERRORS, value) ? value : 'oauth'
}
