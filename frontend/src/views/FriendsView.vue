<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  BASE_URL,
  listFriends,
  listFriendRequests,
  sendFriendRequest,
  acceptFriendRequest,
  removeFriend,
} from '../services/api'
import { useAuth } from '../services/auth'

const router = useRouter()
const { token, clearToken } = useAuth()

const friends = ref([])
const requests = ref([])
const newUsername = ref('')
const error = ref('')
const success = ref('')
const loading = ref(true)

onMounted(async () => {
  if (!token.value) {
    router.push('/login')
    return
  }
  await loadAll()
})

async function loadAll() {
  try {
    friends.value = await listFriends(token.value)
    requests.value = await listFriendRequests(token.value)
    loading.value = false
  } catch {
    clearToken()
    router.push('/login')
  }
}

async function handleSendRequest() {
  error.value = ''
  success.value = ''
  try {
    await sendFriendRequest(token.value, newUsername.value)
    success.value = 'Demande envoyée.'
    newUsername.value = ''
  } catch (err) {
    error.value = err.message
  }
}

async function handleAccept(fromUserId) {
  error.value = ''
  try {
    await acceptFriendRequest(token.value, fromUserId)
    await loadAll()
  } catch (err) {
    error.value = err.message
  }
}

async function handleDecline(fromUserId) {
  try {
    await removeFriend(token.value, fromUserId)
    await loadAll()
  } catch (err) {
    error.value = err.message
  }
}

async function handleRemoveFriend(friendId) {
  try {
    await removeFriend(token.value, friendId)
    await loadAll()
  } catch (err) {
    error.value = err.message
  }
}

function avatarSrc(url) {
  return url ? `${BASE_URL}${url}` : '/default-avatar.svg'
}
</script>

<template>
  <main>
    <h1>Mes amis</h1>

    <form @submit.prevent="handleSendRequest">
      <input v-model="newUsername" type="text" placeholder="Nom d'utilisateur" required />
      <button type="submit">Envoyer une demande</button>
    </form>
    <p v-if="error" class="error">{{ error }}</p>
    <p v-if="success" class="success">{{ success }}</p>

    <p v-if="loading">Chargement...</p>

    <template v-else>
      <section v-if="requests.length > 0">
        <h2>Demandes reçues</h2>
        <ul class="friend-list">
          <li v-for="req in requests" :key="req.id">
            <img :src="avatarSrc(req.avatar_url)" alt="Avatar" class="avatar" />
            <span class="name">{{ req.username }}</span>
            <button @click="handleAccept(req.id)">Accepter</button>
            <button class="remove" @click="handleDecline(req.id)">Refuser</button>
          </li>
        </ul>
      </section>

      <section>
        <h2>Mes amis</h2>
        <p v-if="friends.length === 0" class="empty">Aucun ami pour l'instant.</p>
        <ul v-else class="friend-list">
          <li v-for="friend in friends" :key="friend.id">
            <img :src="avatarSrc(friend.avatar_url)" alt="Avatar" class="avatar" />
            <span class="name">{{ friend.username }}</span>
            <span class="status" :class="{ online: friend.online }">
              {{ friend.online ? 'En ligne' : 'Hors ligne' }}
            </span>
            <button class="remove" @click="handleRemoveFriend(friend.id)">Retirer</button>
          </li>
        </ul>
      </section>
    </template>

    <p><router-link to="/">Retour à l'accueil</router-link></p>
  </main>
</template>

<style scoped>
main {
  font-family: sans-serif;
  max-width: 360px;
  margin: 4rem auto;
  text-align: center;
}
h2 {
  font-size: 1rem;
  margin: 1.5rem 0 0.5rem;
  text-align: left;
}
form {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}
input {
  flex: 1;
  padding: 0.5rem;
  font-size: 1rem;
}
button {
  padding: 0.5rem 0.75rem;
  font-size: 0.9rem;
  cursor: pointer;
}
.error {
  color: #c00;
}
.success {
  color: #080;
}
.empty {
  color: #888;
  text-align: left;
}
.friend-list {
  list-style: none;
  padding: 0;
  margin: 0;
  text-align: left;
}
.friend-list li {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0;
  border-bottom: 1px solid #eee;
}
.avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  object-fit: cover;
  background: #eee;
}
.name {
  flex: 1;
}
.status {
  font-size: 0.8rem;
  color: #999;
}
.status.online {
  color: #080;
}
</style>
