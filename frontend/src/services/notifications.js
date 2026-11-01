import { ref, computed } from 'vue'
import { listNotifications, markNotificationRead, markAllNotificationsRead } from './api'
import { useAuth } from './auth'

// Partage entre la cloche du header et un eventuel autre affichage, comme
// currentUser.js pour le profil.
const items = ref([])

export function useNotifications() {
  const { token } = useAuth()
  const unreadCount = computed(() => items.value.filter((n) => !n.read_at).length)

  async function load() {
    if (!token.value) {
      items.value = []
      return
    }
    try {
      items.value = await listNotifications(token.value)
    } catch {
      // Appele en polling en arriere-plan : une erreur reseau ponctuelle ne
      // doit pas vider la liste ni deconnecter l'utilisateur, juste se
      // reessayer au prochain intervalle.
    }
  }

  async function markRead(id) {
    const target = items.value.find((n) => n.id === id)
    if (!target || target.read_at) return
    target.read_at = new Date().toISOString()
    try {
      await markNotificationRead(token.value, id)
    } catch {
      // Best-effort cote affichage aussi : le prochain load() corrigera l'etat.
    }
  }

  async function markAllRead() {
    if (unreadCount.value === 0) return
    const now = new Date().toISOString()
    items.value.forEach((n) => {
      if (!n.read_at) n.read_at = now
    })
    try {
      await markAllNotificationsRead(token.value)
    } catch {
      // idem
    }
  }

  return { items, unreadCount, load, markRead, markAllRead }
}
