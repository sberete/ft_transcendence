<script setup>
// Friends page: incoming friend requests, the friends list with online
// status, and the form to send a new request by username.
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
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

const { t } = useI18n()
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

// Reloads both the friends list and pending requests from the backend.
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

// Sends a new friend request by username.
async function handleSendRequest() {
  error.value = ''
  success.value = ''
  try {
    await sendFriendRequest(token.value, newUsername.value)
    success.value = t('friends.successSent')
    newUsername.value = ''
  } catch (err) {
    error.value = err.message
  }
}

// Accepts a pending incoming request.
async function handleAccept(fromUserId) {
  error.value = ''
  try {
    await acceptFriendRequest(token.value, fromUserId)
    await loadAll()
  } catch (err) {
    error.value = err.message
  }
}

// Declines a pending incoming request (same endpoint as removing a friend).
async function handleDecline(fromUserId) {
  try {
    await removeFriend(token.value, fromUserId)
    await loadAll()
  } catch (err) {
    error.value = err.message
  }
}

// Removes an existing friend.
async function handleRemoveFriend(friendId) {
  try {
    await removeFriend(token.value, friendId)
    await loadAll()
  } catch (err) {
    error.value = err.message
  }
}

// Resolves an avatar URL, falling back to the default avatar when unset.
function avatarSrc(url) {
  return url ? `${BASE_URL}${url}` : '/default-avatar.svg'
}
</script>

<template>
  <main class="page">
    <div class="container">
      <h1>{{ t('friends.title') }}</h1>

      <form class="panel add" @submit.prevent="handleSendRequest">
        <label class="field">
          {{ t('friends.addLabel') }}
          <div class="add-row">
            <input v-model="newUsername" class="input" type="text" :placeholder="t('friends.usernamePlaceholder')" required />
            <button type="submit" class="btn btn-primary">{{ t('friends.send') }}</button>
          </div>
        </label>
        <p v-if="error" class="msg-error">{{ error }}</p>
        <p v-if="success" class="msg-success">{{ success }}</p>
      </form>

      <div v-if="loading" class="skeleton" style="height: 160px; margin-top: 2rem"></div>

      <template v-else>
        <section v-if="requests.length > 0">
          <h2>{{ t('friends.requestsReceived') }} <span class="count">{{ requests.length }}</span></h2>
          <ul class="list">
            <li v-for="req in requests" :key="req.id" class="panel person">
              <img :src="avatarSrc(req.avatar_url)" alt="" class="avatar" />
              <span class="name">{{ req.username }}</span>
              <button class="btn btn-primary btn-sm" @click="handleAccept(req.id)">{{ t('friends.accept') }}</button>
              <button class="btn btn-ghost btn-sm" @click="handleDecline(req.id)">{{ t('friends.decline') }}</button>
            </li>
          </ul>
        </section>

        <section>
          <h2>{{ t('friends.friendsHeading') }} <span class="count">{{ friends.length }}</span></h2>
          <p v-if="friends.length === 0" class="muted">{{ t('friends.noFriends') }}</p>
          <ul v-else class="list">
            <li v-for="friend in friends" :key="friend.id" class="panel person">
              <div class="avatar-wrap">
                <img :src="avatarSrc(friend.avatar_url)" alt="" class="avatar" />
                <span class="dot" :class="{ online: friend.online }"></span>
              </div>
              <div class="name">
                {{ friend.username }}
                <span class="status">{{ friend.online ? t('friends.online') : t('friends.offline') }}</span>
              </div>
              <button class="btn btn-danger btn-sm" @click="handleRemoveFriend(friend.id)">{{ t('friends.remove') }}</button>
            </li>
          </ul>
        </section>
      </template>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 720px;
  margin: 0 auto;
}

.add {
  margin-top: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.add-row {
  display: flex;
  gap: 0.5rem;
}

h2 {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 2rem 0 0.75rem;
}

.count {
  padding: 0 0.5rem;
  border-radius: 999px;
  font-size: 0.8rem;
  color: var(--accent-hover);
  background: var(--accent-soft);
}

.list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  list-style: none;
  padding: 0;
  margin: 0;
}

.person {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
}

.avatar-wrap {
  position: relative;
}

.avatar {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  object-fit: cover;
  background: var(--bg-hover);
}

.dot {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 12px;
  height: 12px;
  border: 2px solid var(--bg-elevated);
  border-radius: 50%;
  background: #555;
}

.dot.online {
  background: var(--success);
}

.name {
  flex: 1;
  display: flex;
  flex-direction: column;
  font-weight: 600;
  color: var(--text-h);
}

.status {
  font-size: 0.8rem;
  font-weight: 400;
  color: var(--text);
}
</style>
