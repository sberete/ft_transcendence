<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { BASE_URL, getMe, updateMe, uploadAvatar } from '../services/api'
import { useAuth } from '../services/auth'

const router = useRouter()
const { token, clearToken } = useAuth()

const profile = ref(null)
const username = ref('')
const error = ref('')
const success = ref('')
const uploading = ref(false)

onMounted(async () => {
  if (!token.value) {
    router.push('/login')
    return
  }
  await loadProfile()
})

async function loadProfile() {
  try {
    profile.value = await getMe(token.value)
    username.value = profile.value.username
  } catch {
    clearToken()
    router.push('/login')
  }
}

async function handleUpdateUsername() {
  error.value = ''
  success.value = ''
  try {
    profile.value = await updateMe(token.value, username.value)
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
    profile.value = await uploadAvatar(token.value, file)
    success.value = 'Avatar mis à jour.'
  } catch (err) {
    error.value = err.message
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <main v-if="profile">
    <h1>Mon profil</h1>

    <img
      :src="profile.avatar_url ? `${BASE_URL}${profile.avatar_url}` : '/default-avatar.svg'"
      alt="Avatar"
      class="avatar"
    />
    <label class="avatar-upload">
      {{ uploading ? 'Envoi...' : "Changer d'avatar" }}
      <input type="file" accept="image/png,image/jpeg,image/webp" @change="handleAvatarChange" hidden />
    </label>

    <form @submit.prevent="handleUpdateUsername">
      <label>
        Nom d'utilisateur
        <input v-model="username" type="text" required />
      </label>
      <label>
        Email
        <input :value="profile.email" type="email" disabled />
      </label>
      <button type="submit">Enregistrer</button>
    </form>

    <p v-if="error" class="error">{{ error }}</p>
    <p v-if="success" class="success">{{ success }}</p>

    <p><router-link to="/">Retour à l'accueil</router-link></p>
  </main>
</template>

<style scoped>
main {
  font-family: sans-serif;
  max-width: 320px;
  margin: 4rem auto;
  text-align: center;
}
.avatar {
  width: 96px;
  height: 96px;
  border-radius: 50%;
  object-fit: cover;
  background: #eee;
}
.avatar-upload {
  display: block;
  margin: 0.5rem 0 1.5rem;
  color: #06c;
  cursor: pointer;
  font-size: 0.9rem;
}
form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  text-align: left;
}
label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
}
input,
button {
  padding: 0.5rem;
  font-size: 1rem;
}
.error {
  color: #c00;
}
.success {
  color: #080;
}
</style>
