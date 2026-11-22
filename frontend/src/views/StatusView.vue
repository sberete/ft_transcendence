<script setup>
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { BASE_URL } from '../services/api'

const { t } = useI18n()
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
      <h1>{{ t('status.title') }}</h1>
      <p v-if="status === 'checking'">{{ t('status.checking') }}</p>
      <p v-else-if="status === 'online'">{{ t('status.online') }}</p>
      <p v-else>{{ t('status.offline') }}</p>
      <button type="button" :disabled="status === 'checking'" @click="checkStatus">
        {{ t('status.retry') }}
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
