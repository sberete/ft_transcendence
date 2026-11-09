// Shared constants/helpers for the OAuth popup login flow (see OAuthButtons.vue
// and OAuthCallbackView.vue).

// Channel used to talk between the login popup and the page that opened it.
// We don't use window.opener: Google/GitHub's pages send a
// Cross-Origin-Opener-Policy header that severs that link. BroadcastChannel
// instead links every tab of OUR origin, and only those.
export const OAUTH_CHANNEL = 'syncwatch-oauth'

// Displayed text never comes straight from the URL: it's looked up here instead.
export const OAUTH_ERRORS = {
  oauth: 'La connexion avec ce service a échoué. Réessayez.',
  oauth_unavailable: "Ce mode de connexion n'est pas encore disponible. Utilisez votre mot de passe.",
}

// Checks that a value looks like a real session token (64 hex chars).
// We reject anything else rather than storing an arbitrary value.
export function isSessionToken(value) {
  return typeof value === 'string' && /^[0-9a-f]{64}$/.test(value)
}

// Maps any incoming value to a known error code, defaulting to "oauth".
export function oauthErrorCode(value) {
  return Object.hasOwn(OAUTH_ERRORS, value) ? value : 'oauth'
}
