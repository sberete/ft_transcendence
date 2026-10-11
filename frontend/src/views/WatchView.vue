<script setup>
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import VideoPlayer from '../components/VideoPlayer.vue'
import { getMediaDetails } from '../services/api'
import { useAuth } from '../services/auth'
import { PROVIDER_LABELS, formatDuration, htmlToText } from '../services/format'

const route = useRoute()
const { token } = useAuth()

const media = ref(null)
const currentStream = ref(null)
const loading = ref(true)
const error = ref('')
const expanded = ref(false)

const description = computed(() => htmlToText(media.value?.description))

watch(
  () => [route.params.provider, route.params.id],
  async ([provider, id]) => {
    loading.value = true
    error.value = ''
    media.value = null
    expanded.value = false
    try {
      media.value = await getMediaDetails(token.value, provider, id)
      currentStream.value = media.value.streams[0]
      document.title = `${media.value.title} · SyncWatch`
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
  <main class="page watch">
    <div class="player-wrap">
      <div v-if="loading" class="skeleton player-skeleton"></div>
      <div v-else-if="error" class="player-error">
        <p>Impossible de charger cette vidéo.</p>
        <p class="muted">{{ error }}</p>
        <router-link to="/" class="btn btn-ghost">Retour à l'accueil</router-link>
      </div>
      <VideoPlayer
        v-else-if="currentStream"
        :src="currentStream.url"
        :kind="currentStream.kind"
        :poster="media.backdrop || media.thumbnail"
      />
    </div>

    <section v-if="media" class="info">
      <div class="info-head">
        <div>
          <h1>{{ media.title }}</h1>
          <p class="meta">
            <span class="source">{{ PROVIDER_LABELS[media.provider] }}</span>
            <template v-if="media.year"> · {{ media.year }}</template>
            <template v-if="media.duration_seconds"> · {{ formatDuration(media.duration_seconds) }}</template>
          </p>
        </div>

        <div v-if="media.streams.length > 1" class="qualities" role="group" aria-label="Qualité">
          <button
            v-for="stream in media.streams"
            :key="stream.url"
            class="btn btn-sm"
            :class="stream === currentStream ? 'btn-primary' : 'btn-ghost'"
            @click="currentStream = stream"
          >
            {{ stream.quality || stream.kind.toUpperCase() }}
          </button>
        </div>
      </div>

      <div v-if="description" class="panel description">
        <!-- Texte brut uniquement (voir htmlToText) : jamais de v-html sur du contenu externe. -->
        <p :class="{ clamped: !expanded }">{{ description }}</p>
        <button v-if="description.length > 280" class="toggle" @click="expanded = !expanded">
          {{ expanded ? 'Voir moins' : 'Voir plus' }}
        </button>
      </div>
    </section>
  </main>
</template>

<style scoped>
.watch {
  max-width: 1280px;
  margin: 0 auto;
}

.player-wrap {
  overflow: hidden;
  border-radius: var(--radius-lg);
  background: #000;
  box-shadow: var(--shadow);
}

.player-wrap :deep(.player) {
  display: block;
  aspect-ratio: 16 / 9;
  max-height: 75vh;
  border-radius: 0;
}

.player-skeleton {
  aspect-ratio: 16 / 9;
  border-radius: 0;
}

.player-error {
  aspect-ratio: 16 / 9;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  padding: 1rem;
  text-align: center;
  color: var(--text-h);
  background: var(--bg-elevated);
}

.info {
  margin-top: 1.5rem;
}

.info-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.info h1 {
  font-size: clamp(1.4rem, 3vw, 2rem);
}

.meta {
  margin-top: 0.4rem;
}

.source {
  color: var(--accent-hover);
  font-weight: 600;
}

.qualities {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.description {
  margin-top: 1.25rem;
}

.description p {
  white-space: pre-line;
  color: #d4d4de;
}

.description p.clamped {
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.toggle {
  margin-top: 0.5rem;
  padding: 0;
  border: none;
  font: inherit;
  font-weight: 600;
  color: var(--accent-hover);
  background: none;
  cursor: pointer;
}
</style>
