<script setup>
import { ref, onMounted } from 'vue'

const backendStatus = ref('checking...')
const databaseOk = ref(null)

onMounted(async () => {
  try {
    const res = await fetch('http://localhost:3000/health')
    const data = await res.json()
    backendStatus.value = data.status
    databaseOk.value = data.database
  } catch (err) {
    backendStatus.value = 'unreachable'
  }
})
</script>

<template>
  <main>
    <h1>SyncWatch</h1>
    <p>Backend: <strong>{{ backendStatus }}</strong></p>
    <p v-if="databaseOk !== null">Database: <strong>{{ databaseOk ? 'connected' : 'down' }}</strong></p>
  </main>
</template>

<style scoped>
main {
  font-family: sans-serif;
  text-align: center;
  margin-top: 4rem;
}
</style>
