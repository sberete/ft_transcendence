const BASE_URL = 'http://localhost:3000'

async function request(path, options = {}) {
  const res = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
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

export function login(email, password) {
  return request('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  })
}

export function logout(token) {
  return request('/auth/logout', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  })
}
