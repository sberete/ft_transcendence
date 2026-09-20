<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { signup } from '../services/api'
import { useAuth } from '../services/auth'

const username = ref('')
const email = ref('')
const password = ref('')
const error = ref('')
const router = useRouter()
const { setToken } = useAuth()

async function handleSubmit() {
  error.value = ''
  try {
    const data = await signup(username.value, email.value, password.value)
    setToken(data.token)
    router.push('/')
  } catch (err) {
    error.value = err.message
  }
}
</script>

<template>
  <main>
    <h1>Inscription</h1>
    <form @submit.prevent="handleSubmit">
      <input v-model="username" type="text" placeholder="Nom d'utilisateur" required />
      <input v-model="email" type="email" placeholder="Email" required />
      <input v-model="password" type="password" placeholder="Mot de passe (8 caractères min.)" required />
      <button type="submit">S'inscrire</button>
    </form>
    <p v-if="error" class="error">{{ error }}</p>
    <p><router-link to="/login">Déjà un compte ? Se connecter</router-link></p>
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
