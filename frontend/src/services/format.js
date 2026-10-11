export const PROVIDER_LABELS = {
  archive: 'Internet Archive',
  peertube: 'PeerTube',
}

export function formatDuration(seconds) {
  if (!seconds) return ''
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return `${minutes} min`
  return `${Math.floor(minutes / 60)} h ${String(minutes % 60).padStart(2, '0')}`
}

// Les descriptions venant des sources peuvent contenir du HTML arbitraire.
// Ne JAMAIS les injecter avec v-html (XSS) : on en extrait le texte brut.
// DOMParser n'execute ni scripts ni handlers (onerror...) du document parse.
export function htmlToText(html) {
  if (!html) return ''
  const withBreaks = html.replace(/<br\s*\/?>/gi, '\n').replace(/<\/p>/gi, '\n\n')
  const doc = new DOMParser().parseFromString(withBreaks, 'text/html')
  return (doc.body.textContent || '').replace(/\n{3,}/g, '\n\n').trim()
}

export function avatarSrc(baseUrl, url) {
  return url ? `${baseUrl}${url}` : '/default-avatar.svg'
}
