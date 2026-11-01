<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { createRoom } from '../services/api'
import { useAuth } from '../services/auth'

const { t } = useI18n()
const router = useRouter()
const { token } = useAuth()

const name = ref('')
const joinId = ref('')
const error = ref('')
const creating = ref(false)

async function handleCreate() {
  error.value = ''
  creating.value = true
  try {
    const room = await createRoom(token.value, name.value)
    router.push(`/room/${room.code}`)
  } catch (err) {
    error.value = err.message
  } finally {
    creating.value = false
  }
}

function handleJoin() {
  // Accepte aussi bien "123456" qu'un lien colle en entier
  // (https://.../room/123456) : on ne garde que le code a la fin.
  const input = joinId.value.trim()
  const code = input.match(/(\d{6})\/?$/)?.[1]
  if (code) router.push(`/room/${code}`)
}
</script>

<template>
  <main class="page">
    <div class="container">
      <h1>{{ t('rooms.title') }}</h1>
      <p class="muted intro">{{ t('rooms.intro') }}</p>

      <section class="panel">
        <h2>{{ t('rooms.create.title') }}</h2>
        <form class="form" @submit.prevent="handleCreate">
          <label class="field">
            {{ t('rooms.create.nameLabel') }}
            <input v-model="name" class="input" type="text" maxlength="64" :placeholder="t('rooms.create.namePlaceholder')" required />
          </label>
          <button type="submit" class="btn btn-primary" :disabled="creating">
            {{ creating ? t('rooms.create.submitting') : t('rooms.create.submit') }}
          </button>
        </form>
      </section>

      <section class="panel">
        <h2>{{ t('rooms.join.title') }}</h2>
        <p class="muted">{{ t('rooms.join.hint') }}</p>
        <form class="form" @submit.prevent="handleJoin">
          <label class="field">
            {{ t('rooms.join.codeLabel') }}
            <input
              v-model="joinId"
              class="input"
              type="text"
              inputmode="numeric"
              :placeholder="t('rooms.join.codePlaceholder')"
              maxlength="64"
              required
            />
          </label>
          <button type="submit" class="btn btn-ghost">{{ t('rooms.join.submit') }}</button>
        </form>
      </section>

      <p v-if="error" class="msg-error">{{ error }}</p>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 560px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.intro {
  margin-top: -0.5rem;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-top: 1rem;
}

.panel h2 {
  margin-bottom: 0.25rem;
}
</style>
