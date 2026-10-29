// Origine publique du site : nginx sert le front ET fait reverse-proxy vers
// le backend (prefixe /api, voir docker/frontend/nginx.conf). Le navigateur
// ne parle jamais au backend en direct - HTTPS de bout en bout, meme origine.
export const BASE_URL = 'https://localhost:8080'

async function request(path, options = {}) {
  const isFormData = options.body instanceof FormData

  const res = await fetch(`${BASE_URL}/api${path}`, {
    ...options,
    headers: {
      ...(isFormData ? {} : { 'Content-Type': 'application/json' }),
      ...options.headers,
    },
  })

  const isJson = res.headers.get('content-type')?.includes('application/json')
  const body = isJson ? await res.json() : null

  if (!res.ok) {
    throw new Error(body?.error || `Request failed with status ${res.status}`)
  }

  return body
}

export function signup(username, email, password) {
  return request('/auth/signup', {
    method: 'POST',
    body: JSON.stringify({ username, email, password }),
  })
}

export function login(identifier, password) {
  return request('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ identifier, password }),
  })
}

export function logout(token) {
  return request('/auth/logout', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function getMe(token) {
  return request('/users/me', {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function updateMe(token, username) {
  return request('/users/me', {
    method: 'PATCH',
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ username }),
  })
}

export function uploadAvatar(token, file) {
  const formData = new FormData()
  formData.append('file', file)

  return request('/users/me/avatar', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
    body: formData,
  })
}

export function listFriends(token) {
  return request('/friends', {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function sendFriendRequest(token, username) {
  return request('/friends', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ username }),
  })
}

export function listFriendRequests(token) {
  return request('/friends/requests', {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function acceptFriendRequest(token, fromUserId) {
  return request(`/friends/requests/${fromUserId}/accept`, {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function removeFriend(token, friendId) {
  return request(`/friends/${friendId}`, {
    method: 'DELETE',
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function searchMedia(token, query, provider = null) {
  const params = new URLSearchParams({ q: query })
  if (provider) params.set('provider', provider)

  return request(`/media/search?${params}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function getMediaDetails(token, provider, id) {
  return request(`/media/${provider}/${encodeURIComponent(id)}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function createRoom(token, name) {
  return request('/rooms', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ name }),
  })
}

export function getRoom(token, code) {
  return request(`/rooms/${code}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function deleteRoom(token, code) {
  return request(`/rooms/${code}`, {
    method: 'DELETE',
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Historique uniquement : les nouveaux messages arrivent par WebSocket
// (evenement chat.message), pas par cette route. Plus recent en premier.
export function listRoomMessages(token, code, before = null, limit = 50) {
  const params = new URLSearchParams({ limit: String(limit) })
  if (before) params.set('before', String(before))

  return request(`/rooms/${code}/messages?${params}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Pas un appel fetch : le navigateur doit reellement naviguer vers cette URL,
// puisque le backend repond par une redirection vers Google/GitHub.
export function oauthStartUrl(provider) {
  return `${BASE_URL}/api/auth/oauth/${provider}/start`
}
