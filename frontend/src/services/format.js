// Small display-formatting helpers shared across media-related views.
export const PROVIDER_LABELS = {
  archive: 'Internet Archive',
  peertube: 'PeerTube',
}

// Turns a duration in seconds into a short human-readable label.
export function formatDuration(seconds) {
  if (!seconds) return ''
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return `${minutes} min`
  return `${Math.floor(minutes / 60)} h ${String(minutes % 60).padStart(2, '0')}`
}

// Strips HTML coming from external sources down to plain text.
// Descriptions from providers can contain arbitrary HTML. NEVER inject
// them with v-html (XSS): extract plain text from them instead. DOMParser
// does not execute scripts or handlers (onerror...) from the parsed document.
export function htmlToText(html) {
  if (!html) return ''
  const withBreaks = html.replace(/<br\s*\/?>/gi, '\n').replace(/<\/p>/gi, '\n\n')
  const doc = new DOMParser().parseFromString(withBreaks, 'text/html')
  return (doc.body.textContent || '').replace(/\n{3,}/g, '\n\n').trim()
}

// Resolves an avatar URL, falling back to the default avatar when unset.
export function avatarSrc(baseUrl, url) {
  return url ? `${baseUrl}${url}` : '/default-avatar.svg'
}
