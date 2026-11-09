// Shared session-token state: a single module-level ref so every component
// using useAuth() sees the same token and reacts to login/logout instantly.
import { ref } from 'vue'

const token = ref(localStorage.getItem('token'))

export function useAuth() {
  // Stores a new session token, in memory and in localStorage.
  function setToken(value) {
    token.value = value
    localStorage.setItem('token', value)
  }

  // Clears the session token (logout, or an expired/invalid token).
  function clearToken() {
    token.value = null
    localStorage.removeItem('token')
  }

  return { token, setToken, clearToken }
}
