<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { login } from '../services/api'
import { useAuth } from '../services/auth'

const identifier = ref('')
const password = ref('')
const error = ref('')
const router = useRouter()
const { setToken } = useAuth()

async function handleSubmit() {
  error.value = ''
  try {
    const data = await login(identifier.value, password.value)
    setToken(data.token)
    router.push('/')
  } catch (err) {
    error.value = err.message
  }
}
</script>

<template>
  <main>
    <h1>Connexion</h1>
    <form @submit.prevent="handleSubmit">
      <input v-model="identifier" type="text" placeholder="Email ou nom d'utilisateur" required />
      <input v-model="password" type="password" placeholder="Mot de passe" required />
      <button type="submit">Se connecter</button>
    </form>
    <p v-if="error" class="error">{{ error }}</p>
    <p><router-link to="/signup">Pas de compte ? S'inscrire</router-link></p>
  </main>
</template>

<style scoped>
main {
  font-family: sans-serif;
  max-width: 320px;
  margin: 4rem auto;
  text-align: center;
}
form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
input,
button {
  padding: 0.5rem;
  font-size: 1rem;
}
.error {
  color: #c00;
}
</style>
