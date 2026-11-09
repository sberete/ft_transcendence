// Logged-in user's profile, shared between the header and the profile page
// so that changing an avatar is reflected everywhere without a reload.
import { ref } from 'vue'
import { getMe } from './api'
import { useAuth } from './auth'

const user = ref(null)

export function useCurrentUser() {
  const { token, clearToken } = useAuth()

  // Fetches (or clears) the current profile based on the session token.
  async function load() {
    if (!token.value) {
      user.value = null
      return
    }
    try {
      user.value = await getMe(token.value)
    } catch {
      // Token expired or revoked server-side.
      clearToken()
      user.value = null
    }
  }

  // Overwrites the cached profile (e.g. right after an update call).
  function setUser(value) {
    user.value = value
  }

  return { user, load, setUser }
}
