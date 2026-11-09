// Thin wrapper around fetch for every backend call: adds the JSON content
// type (unless sending FormData), the base URL, and turns non-2xx
// responses into thrown errors with the backend's own message.

// Public origin of the site: nginx serves the frontend AND reverse-proxies
// to the backend (prefix /api, see docker/frontend/nginx.conf). The browser
// never talks to the backend directly - HTTPS end to end, same origin.
export const BASE_URL = 'https://localhost:8080'

// Shared fetch helper: builds the request, parses JSON if present, throws on error.
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

// Creates a new account.
export function signup(username, email, password) {
  return request('/auth/signup', {
    method: 'POST',
    body: JSON.stringify({ username, email, password }),
  })
}

// Logs in with username/email + password, returns a session token.
export function login(identifier, password) {
  return request('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ identifier, password }),
  })
}

// Invalidates the current session server-side.
export function logout(token) {
  return request('/auth/logout', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Fetches the logged-in user's own profile.
export function getMe(token) {
  return request('/users/me', {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Updates the logged-in user's username.
export function updateMe(token, username) {
  return request('/users/me', {
    method: 'PATCH',
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ username }),
  })
}

// Uploads a new avatar with live progress reporting.
// XMLHttpRequest rather than fetch: it's the only API with a real upload
// progress event, cross-browser (fetch only exposes the request's own
// progress via a ReadableStream body, not supported everywhere).
// onProgress(percent) fires on every progress event once the total size is known.
export function uploadAvatar(token, file, onProgress) {
  const formData = new FormData()
  formData.append('file', file)

  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    xhr.open('POST', `${BASE_URL}/api/users/me/avatar`)
    xhr.setRequestHeader('Authorization', `Bearer ${token}`)

    xhr.upload.onprogress = (event) => {
      if (onProgress && event.lengthComputable) {
        onProgress(Math.round((event.loaded / event.total) * 100))
      }
    }

    xhr.onload = () => {
      let body = null
      try {
        body = xhr.responseText ? JSON.parse(xhr.responseText) : null
      } catch {
        // Non-JSON response: shouldn't happen on this route.
      }
      if (xhr.status >= 200 && xhr.status < 300) {
        resolve(body)
      } else {
        reject(new Error(body?.error || `Request failed with status ${xhr.status}`))
      }
    }
    xhr.onerror = () => reject(new Error('Network error'))

    xhr.send(formData)
  })
}

// Removes the current avatar and cleans up the file server-side.
export function deleteAvatar(token) {
  return request('/users/me/avatar', {
    method: 'DELETE',
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Lists the logged-in user's accepted friends.
export function listFriends(token) {
  return request('/friends', {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Sends a friend request by username.
export function sendFriendRequest(token, username) {
  return request('/friends', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ username }),
  })
}

// Lists pending incoming friend requests.
export function listFriendRequests(token) {
  return request('/friends/requests', {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Accepts a pending friend request.
export function acceptFriendRequest(token, fromUserId) {
  return request(`/friends/requests/${fromUserId}/accept`, {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Removes an existing friend (symmetric on both accounts).
export function removeFriend(token, friendId) {
  return request(`/friends/${friendId}`, {
    method: 'DELETE',
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Searches the aggregated media catalogue, optionally scoped to one provider.
export function searchMedia(token, query, provider = null) {
  const params = new URLSearchParams({ q: query })
  if (provider) params.set('provider', provider)

  return request(`/media/search?${params}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Fetches full details (streams, title, etc.) for one media item.
export function getMediaDetails(token, provider, id) {
  return request(`/media/${provider}/${encodeURIComponent(id)}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Creates a new watch-party room.
export function createRoom(token, name) {
  return request('/rooms', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ name }),
  })
}

// Fetches a room's metadata by its join code.
export function getRoom(token, code) {
  return request(`/rooms/${code}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Loads past chat history for a room.
// History only: new messages arrive over the WebSocket (chat.message
// event), not through this route. Most recent first.
export function listRoomMessages(token, code, before = null, limit = 50) {
  const params = new URLSearchParams({ limit: String(limit) })
  if (before) params.set('before', String(before))

  return request(`/rooms/${code}/messages?${params}`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Builds the URL that starts an OAuth login flow for a given provider.
// Not a fetch call: the browser must actually navigate to this URL, since
// the backend responds with a redirect to Google/GitHub.
export function oauthStartUrl(provider) {
  return `${BASE_URL}/api/auth/oauth/${provider}/start`
}

// Lists the logged-in user's notifications.
export function listNotifications(token) {
  return request('/notifications', {
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Marks a single notification as read.
export function markNotificationRead(token, id) {
  return request(`/notifications/${id}/read`, {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Marks every notification as read.
export function markAllNotificationsRead(token) {
  return request('/notifications/read-all', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  })
}

// Fetches the personalized "recommended for you" list.
export function getRecommendations(token) {
  return request('/recommendations', {
    headers: { Authorization: `Bearer ${token}` },
  })
}
