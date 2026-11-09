// Shared notification list, polled from the backend: used by the header
// bell and any other place that needs to show unread count/items, the
// same sharing pattern as currentUser.js for the profile.
import { ref, computed } from 'vue'
import { listNotifications, markNotificationRead, markAllNotificationsRead } from './api'
import { useAuth } from './auth'

const items = ref([])

export function useNotifications() {
  const { token } = useAuth()
  const unreadCount = computed(() => items.value.filter((n) => !n.read_at).length)

  // Refreshes the list from the backend (or clears it when logged out).
  async function load() {
    if (!token.value) {
      items.value = []
      return
    }
    try {
      items.value = await listNotifications(token.value)
    } catch {
      // Called on a background polling interval: a one-off network error
      // must not clear the list or log the user out, just retry next tick.
    }
  }

  // Marks a single notification as read, optimistically then server-side.
  async function markRead(id) {
    const target = items.value.find((n) => n.id === id)
    if (!target || target.read_at) return
    target.read_at = new Date().toISOString()
    try {
      await markNotificationRead(token.value, id)
    } catch {
      // Best-effort on the display side too: the next load() will fix the state.
    }
  }

  // Marks every notification as read, optimistically then server-side.
  async function markAllRead() {
    if (unreadCount.value === 0) return
    const now = new Date().toISOString()
    items.value.forEach((n) => {
      if (!n.read_at) n.read_at = now
    })
    try {
      await markAllNotificationsRead(token.value)
    } catch {
      // same as above
    }
  }

  return { items, unreadCount, load, markRead, markAllRead }
}
