<script setup>
import { useRouter } from 'vue-router'
import { logout as apiLogout } from '../services/api'
import { useAuth } from '../services/auth'

const router = useRouter()
const { token, clearToken } = useAuth()

async function handleLogout() {
  try {
    await apiLogout(token.value)
  } catch {
    // token deja invalide/expire cote serveur, on nettoie quand meme localement
  }
  clearToken()
  router.push('/login')
}
</script>

<template>
  <main>
    <h1>SyncWatch</h1>
    <div v-if="token">
      <p>Connecté.</p>
      <button @click="handleLogout">Se déconnecter</button>
    </div>
    <div v-else>
      <p><router-link to="/login">Se connecter</router-link></p>
      <p><router-link to="/signup">S'inscrire</router-link></p>
    </div>
  </main>
</template>

<style scoped>
main {
  font-family: sans-serif;
  text-align: center;
  margin-top: 4rem;
}
</style>
