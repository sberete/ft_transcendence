<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useNotifications } from '../services/notifications'

// Pas de WebSocket global sur ce projet (la connexion n'existe que dans un
// salon) : les notifications sont recuperees par polling, pas pousses en
// temps reel. Voir idea.txt / docs pour la decision.
const POLL_MS = 20000

const LABELS = {
  friend_request: (n) => `${n.payload.from_username} vous a envoyé une demande d'ami`,
  friend_accepted: (n) => `${n.payload.from_username} a accepté votre demande d'ami`,
  friend_removed: (n) => `${n.payload.from_username} vous a retiré de ses amis`,
  room_started: (n) => `${n.payload.from_username} a lancé une session : « ${n.payload.room_name} »`,
  room_ownership_transferred: (n) => `Vous êtes maintenant propriétaire du salon « ${n.payload.room_name} »`,
}

const router = useRouter()
const { items, unreadCount, load, markRead, markAllRead } = useNotifications()

const open = ref(false)
const panel = ref(null)
let pollTimer = null

function describe(n) {
  return LABELS[n.kind]?.(n) ?? n.kind
}

function timeAgo(iso) {
  const diffSec = Math.max(0, Math.floor((Date.now() - new Date(iso).getTime()) / 1000))
  if (diffSec < 60) return "à l'instant"
  const diffMin = Math.floor(diffSec / 60)
  if (diffMin < 60) return `il y a ${diffMin} min`
  const diffH = Math.floor(diffMin / 60)
  if (diffH < 24) return `il y a ${diffH} h`
  return `il y a ${Math.floor(diffH / 24)} j`
}

async function handleClick(n) {
  await markRead(n.id)
  open.value = false
  if (n.kind === 'room_started' || n.kind === 'room_ownership_transferred') {
    router.push(`/room/${n.payload.room_code}`)
  } else if (n.kind.startsWith('friend_')) {
    router.push('/friends')
  }
}

function onClickOutside(event) {
  if (panel.value && !panel.value.contains(event.target)) {
    open.value = false
  }
}

onMounted(() => {
  load()
  pollTimer = setInterval(load, POLL_MS)
  document.addEventListener('click', onClickOutside)
})

onBeforeUnmount(() => {
  clearInterval(pollTimer)
  document.removeEventListener('click', onClickOutside)
})
</script>

<template>
  <div ref="panel" class="notif">
    <button class="bell-btn" :aria-expanded="open" aria-label="Notifications" @click="open = !open">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M12 2a6 6 0 0 0-6 6v3.2c0 .9-.32 1.78-.9 2.47L4 15.5h16l-1.1-1.83a4 4 0 0 1-.9-2.47V8a6 6 0 0 0-6-6z"
        />
        <path d="M9.5 18.5a2.5 2.5 0 0 0 5 0" fill="none" stroke="currentColor" stroke-width="1.5" />
      </svg>
      <span v-if="unreadCount > 0" class="badge">{{ unreadCount > 9 ? '9+' : unreadCount }}</span>
    </button>

    <div v-if="open" class="dropdown">
      <div class="dropdown-head">
        <span>Notifications</span>
        <button v-if="unreadCount > 0" class="mark-all" @click="markAllRead">Tout marquer lu</button>
      </div>
      <p v-if="items.length === 0" class="empty">Rien de nouveau.</p>
      <ul v-else class="list">
        <li v-for="n in items" :key="n.id" :class="{ unread: !n.read_at }" @click="handleClick(n)">
          <p>{{ describe(n) }}</p>
          <span class="time">{{ timeAgo(n.created_at) }}</span>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.notif {
  position: relative;
}

.bell-btn {
  position: relative;
  padding: 0.4rem;
  border: none;
  border-radius: 50%;
  background: none;
  color: var(--text-h);
  cursor: pointer;
  transition: background 0.15s;
}

.bell-btn:hover,
.bell-btn[aria-expanded='true'] {
  background: var(--bg-hover);
}

.bell-btn svg {
  width: 22px;
  height: 22px;
  fill: currentColor;
}

.badge {
  position: absolute;
  top: 0;
  right: 0;
  min-width: 16px;
  height: 16px;
  padding: 0 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.65rem;
  font-weight: 700;
  color: #fff;
  background: var(--danger);
  border-radius: 999px;
}

.dropdown {
  position: absolute;
  right: 0;
  top: calc(100% + 0.5rem);
  width: 320px;
  max-height: 420px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.dropdown-head {
  position: sticky;
  top: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 0.75rem;
  font-weight: 700;
  color: var(--text-h);
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border);
}

.mark-all {
  border: none;
  background: none;
  color: var(--accent);
  font: inherit;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
}

.empty {
  padding: 1.25rem 0.75rem;
  text-align: center;
  color: var(--text);
  font-size: 0.9rem;
}

.list {
  display: flex;
  flex-direction: column;
}

.list li {
  padding: 0.65rem 0.75rem;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
}

.list li:last-child {
  border-bottom: none;
}

.list li:hover {
  background: var(--bg-hover);
}

.list li.unread {
  background: var(--accent-soft);
}

.list p {
  margin: 0;
  font-size: 0.85rem;
  color: var(--text-h);
  line-height: 1.35;
}

.list .time {
  font-size: 0.75rem;
  color: var(--text);
}
</style>
