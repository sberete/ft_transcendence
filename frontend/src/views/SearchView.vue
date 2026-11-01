<script setup>
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import MediaCard from '../components/MediaCard.vue'
import { searchMedia } from '../services/api'
import { useAuth } from '../services/auth'
import { PROVIDER_LABELS } from '../services/format'

const { t } = useI18n()
const route = useRoute()
const { token } = useAuth()

const results = ref([])
const loading = ref(false)
const error = ref('')

// Filtre par source : refait l'appel (le backend ne cherche que la source
// demandee, pas un filtre applique apres coup sur un resultat deja limite).
const provider = ref('')
// Tri et pagination, eux, s'appliquent sur ce qu'on a deja recu : pas besoin
// de re-interroger les sources pour ca.
const sortBy = ref('relevance')
const page = ref(1)
const PAGE_SIZE = 12

async function runSearch(q) {
  results.value = []
  error.value = ''
  page.value = 1
  if (!q) return

  loading.value = true
  try {
    results.value = await searchMedia(token.value, q, provider.value || null)
  } catch (err) {
    error.value = err.message
  } finally {
    loading.value = false
  }
}

// La recherche vit dans l'URL (?q=...) : partageable, et le bouton retour fonctionne.
watch(() => route.query.q, (q) => runSearch(q), { immediate: true })

watch(provider, () => runSearch(route.query.q))

const sortedResults = computed(() => {
  const list = [...results.value]
  switch (sortBy.value) {
    case 'title':
      return list.sort((a, b) => a.title.localeCompare(b.title))
    case 'year-desc':
      // Sans annee connue : toujours apres les resultats datés.
      return list.sort((a, b) => (b.year ?? -Infinity) - (a.year ?? -Infinity))
    case 'duration-desc':
      return list.sort((a, b) => (b.duration_seconds ?? 0) - (a.duration_seconds ?? 0))
    default:
      return list
  }
})

const totalPages = computed(() => Math.max(1, Math.ceil(sortedResults.value.length / PAGE_SIZE)))

// Le tri peut changer sans que la page courante reste valide (ex: moins de
// pages qu'avant si on vient de filtrer) : on la recale au lieu de montrer
// une page vide.
watch([sortBy, totalPages], () => {
  if (page.value > totalPages.value) page.value = totalPages.value
})

const pagedResults = computed(() => {
  const start = (page.value - 1) * PAGE_SIZE
  return sortedResults.value.slice(start, start + PAGE_SIZE)
})
</script>

<template>
  <main class="page">
    <h1 v-if="route.query.q">{{ t('search.resultsFor', { query: route.query.q }) }}</h1>
    <h1 v-else>{{ t('search.title') }}</h1>

    <p v-if="error" class="msg-error">{{ error }}</p>
    <p v-else-if="!route.query.q" class="muted hint">{{ t('search.hint') }}</p>

    <div v-if="route.query.q && !error" class="controls">
      <label class="control">
        {{ t('search.sourceLabel') }}
        <select v-model="provider" class="input">
          <option value="">{{ t('search.sourceAll') }}</option>
          <option v-for="(label, id) in PROVIDER_LABELS" :key="id" :value="id">{{ label }}</option>
        </select>
      </label>
      <label class="control">
        {{ t('search.sortLabel') }}
        <select v-model="sortBy" class="input">
          <option value="relevance">{{ t('search.sort.relevance') }}</option>
          <option value="title">{{ t('search.sort.title') }}</option>
          <option value="year-desc">{{ t('search.sort.yearDesc') }}</option>
          <option value="duration-desc">{{ t('search.sort.durationDesc') }}</option>
        </select>
      </label>
      <p v-if="!loading" class="muted count">
        {{ t('search.resultCount', { count: sortedResults.length }, sortedResults.length) }}
      </p>
    </div>

    <div class="grid">
      <template v-if="loading">
        <div v-for="n in 12" :key="n">
          <div class="skeleton" style="aspect-ratio: 16 / 9"></div>
          <div class="skeleton" style="height: 0.9rem; width: 70%; margin-top: 0.6rem"></div>
        </div>
      </template>
      <MediaCard v-for="item in pagedResults" v-else :key="`${item.provider}:${item.id}`" :item="item" />
    </div>

    <div v-if="!loading && totalPages > 1" class="pagination">
      <button class="btn btn-ghost btn-sm" :disabled="page === 1" @click="page--">{{ t('search.prev') }}</button>
      <span class="muted">{{ t('search.pageOf', { page, total: totalPages }) }}</span>
      <button class="btn btn-ghost btn-sm" :disabled="page === totalPages" @click="page++">{{ t('search.next') }}</button>
    </div>

    <div v-if="!loading && route.query.q && results.length === 0 && !error" class="empty">
      <p>{{ t('search.empty') }}</p>
    </div>
  </main>
</template>

<style scoped>
.hint {
  margin: 0.5rem 0 1.5rem;
}

.controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 1.25rem;
  margin: 0.5rem 0 1.5rem;
}

.control {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  font-size: 0.8rem;
  color: var(--text);
}

.control .input {
  min-width: 10rem;
}

.count {
  margin-left: auto;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 1.5rem 1rem;
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  margin-top: 2rem;
}

.empty {
  padding: 3rem 0;
  text-align: center;
}
</style>
