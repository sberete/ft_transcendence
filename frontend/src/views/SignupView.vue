<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AuthCard from '../components/AuthCard.vue'
import OAuthButtons from '../components/OAuthButtons.vue'
import { signup } from '../services/api'
import { useAuth } from '../services/auth'

const username = ref('')
const email = ref('')
const password = ref('')
const error = ref('')
const submitting = ref(false)
const router = useRouter()
const { setToken } = useAuth()

async function handleSubmit() {
  error.value = ''
  submitting.value = true
  try {
    const data = await signup(username.value, email.value, password.value)
    setToken(data.token)
    router.push('/')
  } catch (err) {
    error.value = err.message
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <AuthCard title="Créer un compte" subtitle="Gratuit, et ça prend 10 secondes.">
    <form @submit.prevent="handleSubmit">
      <label class="field">
        Nom d'utilisateur
        <input v-model="username" class="input" type="text" autocomplete="username" maxlength="32" required />
      </label>
      <label class="field">
        Email
        <input v-model="email" class="input" type="email" autocomplete="email" required />
      </label>
      <label class="field">
        Mot de passe
        <input
          v-model="password"
          class="input"
          type="password"
          autocomplete="new-password"
          minlength="8"
          placeholder="8 caractères minimum"
          required
        />
      </label>
      <p v-if="error" class="msg-error">{{ error }}</p>
      <button type="submit" class="btn btn-primary btn-lg" :disabled="submitting">
        {{ submitting ? 'Création...' : "S'inscrire" }}
      </button>
    </form>
    <OAuthButtons />
    <p class="switch muted">Déjà un compte ? <router-link to="/login">Se connecter</router-link></p>
  </AuthCard>
</template>
