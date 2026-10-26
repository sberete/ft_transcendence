<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { createRoom } from '../services/api'
import { useAuth } from '../services/auth'

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
    router.push(`/room/${room.id}`)
  } catch (err) {
    error.value = err.message
  } finally {
    creating.value = false
  }
}

function handleJoin() {
  const id = joinId.value.trim()
  if (id) router.push(`/room/${id}`)
}
</script>

<template>
  <main class="page">
    <div class="container">
      <h1>Salles</h1>
      <p class="muted intro">Regardez une vidéo en même temps que vos amis, avec le chat et la lecture synchronisés.</p>

      <section class="panel">
        <h2>Créer une salle</h2>
        <form class="form" @submit.prevent="handleCreate">
          <label class="field">
            Nom de la salle
            <input v-model="name" class="input" type="text" maxlength="64" placeholder="Soirée film" required />
          </label>
          <button type="submit" class="btn btn-primary" :disabled="creating">
            {{ creating ? 'Création...' : 'Créer' }}
          </button>
        </form>
      </section>

      <section class="panel">
        <h2>Rejoindre une salle</h2>
        <p class="muted">Demandez l'identifiant de la salle à la personne qui l'a créée.</p>
        <form class="form" @submit.prevent="handleJoin">
          <label class="field">
            Identifiant de la salle
            <input v-model="joinId" class="input" type="text" inputmode="numeric" required />
          </label>
          <button type="submit" class="btn btn-ghost">Rejoindre</button>
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
