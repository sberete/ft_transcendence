export const BASE_URL = 'http://localhost:3000'

async function request(path, options = {}) {
  const isFormData = options.body instanceof FormData

  const res = await fetch(`${BASE_URL}${path}`, {
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
