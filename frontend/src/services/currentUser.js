import { ref } from 'vue'
import { getMe } from './api'
import { useAuth } from './auth'

// Profil de l'utilisateur connecte, partage entre le header et la page profil
// pour que changer d'avatar se voie partout sans recharger.
const user = ref(null)

export function useCurrentUser() {
  const { token, clearToken } = useAuth()

  async function load() {
    if (!token.value) {
      user.value = null
      return
    }
    try {
      user.value = await getMe(token.value)
    } catch {
      // Token expire ou revoque cote serveur.
      clearToken()
      user.value = null
    }
  }

  function setUser(value) {
    user.value = value
  }

  return { user, load, setUser }
}
