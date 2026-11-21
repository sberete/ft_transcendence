<script setup>
import { onMounted, ref } from 'vue'
import { BASE_URL } from '../services/api'

const status = ref('checking')

async function checkStatus() {
  status.value = 'checking'

  try {
    const response = await fetch(`${BASE_URL}/api/health`, { cache: 'no-store' })
    const body = await response.json()
    status.value = response.ok && body.status === 'ok' ? 'online' : 'offline'
  } catch {
    status.value = 'offline'
  }
}

onMounted(checkStatus)
</script>

<template>
  <main class="page">
    <section class="panel status-panel" aria-live="polite">
      <h1>État du service</h1>
      <p v-if="status === 'checking'">Vérification en cours…</p>
      <p v-else-if="status === 'online'">🟢 Le service fonctionne.</p>
      <p v-else>🔴 Le service est indisponible.</p>
      <button type="button" :disabled="status === 'checking'" @click="checkStatus">
        Réessayer
      </button>
    </section>
  </main>
</template>

<style scoped>
.status-panel {
  max-width: 32rem;
  margin: 0 auto;
}
</style>
