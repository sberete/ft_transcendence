<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { BASE_URL, listRoomMessages } from '../services/api'
import { useAuth } from '../services/auth'
import { useCurrentUser } from '../services/currentUser'
import { avatarSrc } from '../services/format'
import { createRoomSocket } from '../services/roomSocket'

const route = useRoute()
const router = useRouter()
const { token } = useAuth()
const { user, load: loadUser } = useCurrentUser()

const roomId = computed(() => Number(route.params.id))
const socket = createRoomSocket(token.value, user)
const { connected, room, members, messages, error } = socket

const history = ref([])
const newMessage = ref('')
const chatEl = ref(null)

// Historique (REST, pagine, plus recent en premier) + messages vus depuis la
// connexion WS (deja chronologiques) : on affiche l'un a la suite de l'autre.
const allMessages = computed(() => [...history.value, ...messages.value])

function scrollToBottom() {
  requestAnimationFrame(() => {
    if (chatEl.value) chatEl.value.scrollTop = chatEl.value.scrollHeight
  })
}

watch(() => allMessages.value.length, scrollToBottom)

onMounted(async () => {
  if (!user.value) await loadUser()

  try {
    const recent = await listRoomMessages(token.value, roomId.value)
    history.value = recent.reverse()
  } catch {
    // L'historique est secondaire : si ca echoue, le chat live fonctionne quand meme.
  }

  socket.connect()
  socket.joinRoom(roomId.value)
})

onBeforeUnmount(() => {
  socket.leaveRoom()
  socket.close()
})

function handleSend() {
  const content = newMessage.value.trim()
  if (!content) return
  socket.sendChat(content)
  newMessage.value = ''
}

function leaveToLobby() {
  router.push('/rooms')
}
</script>

<template>
  <main class="page room">
    <div class="container">
      <div class="room-head">
        <div>
          <h1>{{ room?.name || 'Salle' }}</h1>
          <p class="muted">
            <span v-if="!connected">Connexion...</span>
            <span v-else>{{ members.length }} participant{{ members.length > 1 ? 's' : '' }}</span>
          </p>
        </div>
        <button class="btn btn-ghost btn-sm" @click="leaveToLobby">Quitter</button>
      </div>

      <p v-if="error" class="msg-error">{{ error }}</p>

      <div class="room-body">
        <div class="video-slot panel">
          <p class="muted">Lecteur vidéo synchronisé — bientôt disponible.</p>
        </div>

        <aside class="chat panel">
          <div class="members">
            <span v-for="m in members" :key="m.id" class="member">
              <img :src="avatarSrc(BASE_URL, m.avatar_url)" alt="" />
              {{ m.username }}
            </span>
          </div>

          <div ref="chatEl" class="messages">
            <p v-if="allMessages.length === 0" class="muted empty">Aucun message pour l'instant.</p>
            <p v-for="msg in allMessages" :key="msg.id" class="message">
              <strong>{{ msg.sender.username }}</strong> {{ msg.content }}
            </p>
          </div>

          <form class="composer" @submit.prevent="handleSend">
            <input
              v-model="newMessage"
              class="input"
              type="text"
              maxlength="500"
              placeholder="Écrire un message..."
              :disabled="!connected"
            />
            <button type="submit" class="btn btn-primary btn-sm" :disabled="!connected">Envoyer</button>
          </form>
        </aside>
      </div>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 1100px;
  margin: 0 auto;
}

.room-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.room-body {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 1.5rem;
  align-items: stretch;
}

.video-slot {
  aspect-ratio: 16 / 9;
  display: flex;
  align-items: center;
  justify-content: center;
}

.chat {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  max-height: 70vh;
}

.members {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border);
}

.member {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.8rem;
  color: var(--text);
}

.member img {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  object-fit: cover;
  background: var(--bg-hover);
}

.messages {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-height: 0;
}

.message {
  font-size: 0.9rem;
  color: var(--text);
  word-break: break-word;
}

.message strong {
  color: var(--text-h);
  margin-right: 0.35rem;
}

.empty {
  margin: auto 0;
  text-align: center;
}

.composer {
  display: flex;
  gap: 0.5rem;
}

@media (max-width: 900px) {
  .room-body {
    grid-template-columns: 1fr;
  }

  .chat {
    max-height: 50vh;
  }
}
</style>
