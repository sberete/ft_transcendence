<script setup>
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import MediaCard from '../components/MediaCard.vue'
import { searchMedia } from '../services/api'
import { useAuth } from '../services/auth'

const route = useRoute()
const { token } = useAuth()

const results = ref([])
const loading = ref(false)
const error = ref('')

// La recherche vit dans l'URL (?q=...) : partageable, et le bouton retour fonctionne.
watch(
  () => route.query.q,
  async (q) => {
    results.value = []
    error.value = ''
    if (!q) return

    loading.value = true
    try {
      results.value = await searchMedia(token.value, q)
    } catch (err) {
      error.value = err.message
    } finally {
      loading.value = false
    }
  },
  { immediate: true },
)
</script>

<template>
  <main class="page">
    <h1 v-if="route.query.q">Résultats pour « {{ route.query.q }} »</h1>
    <h1 v-else>Recherche</h1>

    <p v-if="error" class="msg-error">{{ error }}</p>
    <p v-else-if="!route.query.q" class="muted hint">Utilisez la barre de recherche en haut de la page.</p>
    <p v-else-if="!loading" class="muted hint">{{ results.length }} résultat{{ results.length > 1 ? 's' : '' }}</p>

    <div class="grid">
      <template v-if="loading">
        <div v-for="n in 12" :key="n">
          <div class="skeleton" style="aspect-ratio: 16 / 9"></div>
          <div class="skeleton" style="height: 0.9rem; width: 70%; margin-top: 0.6rem"></div>
        </div>
      </template>
      <MediaCard v-for="item in results" v-else :key="`${item.provider}:${item.id}`" :item="item" />
    </div>

    <div v-if="!loading && route.query.q && results.length === 0 && !error" class="empty">
      <p>Aucun résultat. Essayez un autre titre, comme « Nosferatu » ou « Blender ».</p>
    </div>
  </main>
</template>

<style scoped>
.hint {
  margin: 0.5rem 0 1.5rem;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 1.5rem 1rem;
}

.empty {
  padding: 3rem 0;
  text-align: center;
}
</style>
