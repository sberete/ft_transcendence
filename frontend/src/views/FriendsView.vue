<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { BASE_URL, listFriends, addFriend, removeFriend } from '../services/api'
import { useAuth } from '../services/auth'

const router = useRouter()
const { token, clearToken } = useAuth()

const friends = ref([])
const newUsername = ref('')
const error = ref('')
const loading = ref(true)

onMounted(async () => {
  if (!token.value) {
    router.push('/login')
    return
  }
  await loadFriends()
})

async function loadFriends() {
  try {
    friends.value = await listFriends(token.value)
    loading.value = false
  } catch {
    clearToken()
    router.push('/login')
  }
}

async function handleAddFriend() {
  error.value = ''
  try {
    await addFriend(token.value, newUsername.value)
    newUsername.value = ''
    await loadFriends()
  } catch (err) {
    error.value = err.message
  }
}

async function handleRemoveFriend(friendId) {
  try {
    await removeFriend(token.value, friendId)
    await loadFriends()
  } catch (err) {
    error.value = err.message
  }
}
</script>

<template>
  <main>
    <h1>Mes amis</h1>

    <form @submit.prevent="handleAddFriend">
      <input v-model="newUsername" type="text" placeholder="Nom d'utilisateur" required />
      <button type="submit">Ajouter</button>
    </form>
    <p v-if="error" class="error">{{ error }}</p>

    <p v-if="loading">Chargement...</p>
    <p v-else-if="friends.length === 0" class="empty">Aucun ami pour l'instant.</p>
    <ul v-else class="friend-list">
      <li v-for="friend in friends" :key="friend.id">
        <img
          :src="friend.avatar_url ? `${BASE_URL}${friend.avatar_url}` : '/default-avatar.svg'"
          alt="Avatar"
          class="avatar"
        />
        <span class="name">{{ friend.username }}</span>
        <span class="status" :class="{ online: friend.online }">
          {{ friend.online ? 'En ligne' : 'Hors ligne' }}
        </span>
        <button class="remove" @click="handleRemoveFriend(friend.id)">Retirer</button>
      </li>
    </ul>

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
  font-size: 1rem;
  cursor: pointer;
}
.error {
  color: #c00;
}
.empty {
  color: #888;
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
.remove {
  font-size: 0.8rem;
  padding: 0.25rem 0.5rem;
}
</style>
