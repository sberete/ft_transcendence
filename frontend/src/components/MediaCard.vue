<script setup>
import { ref } from 'vue'
import { PROVIDER_LABELS, formatDuration } from '../services/format'

defineProps({
  item: { type: Object, required: true },
})

const imageFailed = ref(false)
</script>

<template>
  <router-link :to="`/watch/${item.provider}/${encodeURIComponent(item.id)}`" class="card">
    <div class="thumb">
      <img
        v-if="item.thumbnail && !imageFailed"
        :src="item.thumbnail"
        alt=""
        loading="lazy"
        @error="imageFailed = true"
      />
      <div v-else class="thumb-fallback">{{ item.title.charAt(0) }}</div>
      <span v-if="item.duration_seconds" class="duration">{{ formatDuration(item.duration_seconds) }}</span>
      <div class="play" aria-hidden="true">
        <svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z" /></svg>
      </div>
    </div>
    <p class="title" :title="item.title">{{ item.title }}</p>
    <p class="meta">
      {{ PROVIDER_LABELS[item.provider] }}<template v-if="item.year"> · {{ item.year }}</template>
    </p>
  </router-link>
</template>

<style scoped>
.card {
  display: block;
  color: inherit;
}

.card:hover {
  text-decoration: none;
}

.thumb {
  position: relative;
  aspect-ratio: 16 / 9;
  overflow: hidden;
  border-radius: var(--radius);
  background: var(--bg-elevated);
}

/* En absolu, l'image ne participe pas au calcul de taille : le cadre reste en 16/9
   quelles que soient les dimensions de la vignette d'origine. */
.thumb img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s;
}

.thumb-fallback {
  display: grid;
  place-items: center;
  height: 100%;
  font-size: 2.5rem;
  font-weight: 800;
  color: var(--accent-hover);
  background: linear-gradient(135deg, var(--bg-hover), var(--accent-soft));
}

.duration {
  position: absolute;
  right: 0.4rem;
  bottom: 0.4rem;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  color: #fff;
  background: rgba(0, 0, 0, 0.75);
}

.play {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.35);
  opacity: 0;
  transition: opacity 0.2s;
}

.play svg {
  width: 48px;
  height: 48px;
  padding: 10px;
  fill: #fff;
  background: var(--accent);
  border-radius: 50%;
  box-shadow: var(--shadow);
}

.card:hover .thumb img,
.card:focus-visible .thumb img {
  transform: scale(1.05);
}

.card:hover .play,
.card:focus-visible .play {
  opacity: 1;
}

.title {
  margin-top: 0.5rem;
  font-weight: 600;
  font-size: 0.92rem;
  color: var(--text-h);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.meta {
  font-size: 0.8rem;
  color: var(--text);
}
</style>
