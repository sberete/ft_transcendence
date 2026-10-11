<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { BASE_URL, getMe, updateMe, uploadAvatar } from '../services/api'
import { useAuth } from '../services/auth'
import { useCurrentUser } from '../services/currentUser'
import { avatarSrc } from '../services/format'

const router = useRouter()
const { token, clearToken } = useAuth()
const { setUser } = useCurrentUser()

const profile = ref(null)
const username = ref('')
const error = ref('')
const success = ref('')
const uploading = ref(false)

onMounted(loadProfile)

async function loadProfile() {
  try {
    profile.value = await getMe(token.value)
    username.value = profile.value.username
  } catch {
    clearToken()
    router.push('/login')
  }
}

// Met a jour la page ET le header (avatar, pseudo dans le menu).
function applyProfile(value) {
  profile.value = value
  setUser(value)
}

async function handleUpdateUsername() {
  error.value = ''
  success.value = ''
  try {
    applyProfile(await updateMe(token.value, username.value))
    success.value = 'Profil mis à jour.'
  } catch (err) {
    error.value = err.message
  }
}

async function handleAvatarChange(event) {
  const file = event.target.files[0]
  if (!file) return

  error.value = ''
  success.value = ''
  uploading.value = true
  try {
    applyProfile(await uploadAvatar(token.value, file))
    success.value = 'Avatar mis à jour.'
  } catch (err) {
    error.value = err.message
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <main class="page">
    <div class="container">
      <h1>Mon profil</h1>

      <div v-if="profile" class="panel profile">
        <div class="avatar-col">
          <img :src="avatarSrc(BASE_URL, profile.avatar_url)" alt="Avatar" class="avatar" />
          <label class="btn btn-ghost btn-sm">
            {{ uploading ? 'Envoi...' : "Changer d'avatar" }}
            <input type="file" accept="image/png,image/jpeg,image/webp" hidden @change="handleAvatarChange" />
          </label>
        </div>

        <form class="fields" @submit.prevent="handleUpdateUsername">
          <label class="field">
            Nom d'utilisateur
            <input v-model="username" class="input" type="text" maxlength="32" required />
          </label>
          <label class="field">
            Email
            <input :value="profile.email" class="input" type="email" disabled />
          </label>
          <p v-if="error" class="msg-error">{{ error }}</p>
          <p v-if="success" class="msg-success">{{ success }}</p>
          <button type="submit" class="btn btn-primary">Enregistrer</button>
        </form>
      </div>

      <div v-else class="panel skeleton" style="height: 240px"></div>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 720px;
  margin: 0 auto;
}

.profile {
  display: flex;
  flex-wrap: wrap;
  gap: 2rem;
  margin-top: 1.5rem;
}

.avatar-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.avatar {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  object-fit: cover;
  background: var(--bg-hover);
  border: 3px solid var(--border);
}

.fields {
  flex: 1;
  min-width: 240px;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.fields .btn {
  align-self: flex-start;
}
</style>
