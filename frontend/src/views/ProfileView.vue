<script setup>
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { BASE_URL, deleteAvatar, getMe, updateMe, uploadAvatar } from '../services/api'
import { useAuth } from '../services/auth'
import { useCurrentUser } from '../services/currentUser'
import { avatarSrc } from '../services/format'

const { t } = useI18n()
const router = useRouter()
const { token, clearToken } = useAuth()
const { setUser } = useCurrentUser()

const profile = ref(null)
const username = ref('')
const error = ref('')
const success = ref('')
const uploading = ref(false)
const uploadProgress = ref(0)
const deletingAvatar = ref(false)

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
    success.value = t('profile.successProfile')
  } catch (err) {
    error.value = err.message
  }
}

async function handleAvatarChange(event) {
  const file = event.target.files[0]
  // Vide la valeur tout de suite : sinon re-choisir le MEME fichier après
  // une erreur ne redéclencherait pas @change (le navigateur ne voit
  // aucun changement de valeur).
  event.target.value = ''
  if (!file) return

  error.value = ''
  success.value = ''
  uploading.value = true
  uploadProgress.value = 0
  try {
    applyProfile(await uploadAvatar(token.value, file, (percent) => {
      uploadProgress.value = percent
    }))
    success.value = t('profile.successAvatar')
  } catch (err) {
    error.value = err.message
  } finally {
    uploading.value = false
  }
}

async function handleAvatarDelete() {
  error.value = ''
  success.value = ''
  deletingAvatar.value = true
  try {
    applyProfile(await deleteAvatar(token.value))
    success.value = t('profile.successAvatarDeleted')
  } catch (err) {
    error.value = err.message
  } finally {
    deletingAvatar.value = false
  }
}
</script>

<template>
  <main class="page">
    <div class="container">
      <h1>{{ t('profile.title') }}</h1>

      <div v-if="profile" class="panel profile">
        <div class="avatar-col">
          <img :src="avatarSrc(BASE_URL, profile.avatar_url)" alt="Avatar" class="avatar" />
          <div v-if="uploading" class="upload-progress">
            <div class="upload-progress-track">
              <div class="upload-progress-bar" :style="{ width: `${uploadProgress}%` }"></div>
            </div>
            <span class="upload-progress-label">{{ uploadProgress }}%</span>
          </div>
          <div class="avatar-actions">
            <label class="btn btn-ghost btn-sm" :class="{ 'btn-disabled': uploading }">
              {{ t('profile.changeAvatar') }}
              <input
                type="file"
                accept="image/png,image/jpeg,image/webp"
                hidden
                :disabled="uploading"
                @change="handleAvatarChange"
              />
            </label>
            <button
              v-if="profile.avatar_url"
              type="button"
              class="btn btn-ghost btn-sm"
              :disabled="deletingAvatar"
              @click="handleAvatarDelete"
            >
              {{ deletingAvatar ? t('profile.deleting') : t('profile.delete') }}
            </button>
          </div>
        </div>

        <form class="fields" @submit.prevent="handleUpdateUsername">
          <label class="field">
            {{ t('profile.username') }}
            <input v-model="username" class="input" type="text" maxlength="32" required />
          </label>
          <label class="field">
            {{ t('profile.email') }}
            <input :value="profile.email" class="input" type="email" disabled />
          </label>
          <p v-if="error" class="msg-error">{{ error }}</p>
          <p v-if="success" class="msg-success">{{ success }}</p>
          <button type="submit" class="btn btn-primary">{{ t('profile.save') }}</button>
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

.upload-progress {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 120px;
}

.upload-progress-track {
  flex: 1;
  height: 0.4rem;
  border-radius: 999px;
  background: var(--bg-hover);
  overflow: hidden;
}

.upload-progress-bar {
  height: 100%;
  background: var(--accent);
  transition: width 0.15s ease;
}

.upload-progress-label {
  min-width: 2.5em;
  text-align: right;
  font-size: 0.75rem;
  color: var(--text);
}

.avatar-actions {
  display: flex;
  gap: 0.5rem;
}

.avatar-actions .btn-disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
