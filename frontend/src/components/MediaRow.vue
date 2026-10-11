<script setup>
import { ref, onMounted } from 'vue'
import MediaCard from './MediaCard.vue'
import { searchMedia } from '../services/api'
import { useAuth } from '../services/auth'

// Une rangee = une recherche thematique ("chaplin" sur Archive, "animation" sur PeerTube...).
const props = defineProps({
  title: { type: String, required: true },
  query: { type: String, required: true },
  provider: { type: String, default: null },
})

const { token } = useAuth()
const items = ref([])
const loading = ref(true)
const track = ref(null)

onMounted(async () => {
  try {
    items.value = await searchMedia(token.value, props.query, props.provider)
  } catch {
    items.value = []
  } finally {
    loading.value = false
  }
})

function scroll(direction) {
  const el = track.value
  el.scrollBy({ left: direction * el.clientWidth * 0.85, behavior: 'smooth' })
}
</script>

<template>
  <!-- Une rangee vide (source en panne, aucun resultat) disparait au lieu d'afficher un trou. -->
  <section v-if="loading || items.length > 0" class="row">
    <div class="row-head">
      <h2>{{ title }}</h2>
      <router-link v-if="!loading" :to="{ path: '/search', query: { q: query } }" class="see-all">Tout voir</router-link>
    </div>

    <div class="row-body">
      <button class="arrow left" aria-label="Défiler vers la gauche" @click="scroll(-1)">‹</button>
      <div ref="track" class="track">
        <template v-if="loading">
          <div v-for="n in 6" :key="n" class="slot">
            <div class="skeleton thumb-skeleton"></div>
            <div class="skeleton line-skeleton"></div>
          </div>
        </template>
        <div v-for="item in items" v-else :key="`${item.provider}:${item.id}`" class="slot">
          <MediaCard :item="item" />
        </div>
      </div>
      <button class="arrow right" aria-label="Défiler vers la droite" @click="scroll(1)">›</button>
    </div>
  </section>
</template>

<style scoped>
.row {
  margin-bottom: 2.5rem;
}

.row-head {
  display: flex;
  align-items: baseline;
  gap: 1rem;
  padding: 0 var(--gutter);
  margin-bottom: 0.75rem;
}

.see-all {
  font-size: 0.85rem;
  font-weight: 600;
}

.row-body {
  position: relative;
}

.track {
  display: flex;
  gap: 0.9rem;
  overflow-x: auto;
  scroll-snap-type: x mandatory;
  scroll-padding: 0 var(--gutter);
  padding: 0 var(--gutter) 0.5rem;
  scrollbar-width: none;
}

.track::-webkit-scrollbar {
  display: none;
}

.slot {
  flex: 0 0 clamp(200px, 22vw, 280px);
  /* Sans ca, min-width: auto laisse une grande image elargir la carte. */
  min-width: 0;
  scroll-snap-align: start;
}

.thumb-skeleton {
  aspect-ratio: 16 / 9;
}

.line-skeleton {
  height: 0.9rem;
  width: 70%;
  margin-top: 0.6rem;
}

.arrow {
  position: absolute;
  top: 0;
  bottom: 3.5rem;
  z-index: 2;
  width: var(--gutter);
  min-width: 36px;
  border: none;
  font-size: 2.2rem;
  color: #fff;
  background: rgba(11, 11, 16, 0.7);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s;
}

.arrow.left {
  left: 0;
}

.arrow.right {
  right: 0;
}

.row-body:hover .arrow,
.arrow:focus-visible {
  opacity: 1;
}

/* Sur ecran tactile on defile au doigt : pas de fleches. */
@media (hover: none) {
  .arrow {
    display: none;
  }
}
</style>
