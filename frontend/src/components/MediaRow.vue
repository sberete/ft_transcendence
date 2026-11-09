<script setup>
// Horizontally scrollable row of MediaCards, backed by one search query.
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import MediaCard from './MediaCard.vue'
import { searchMedia } from '../services/api'
import { useAuth } from '../services/auth'

// One row = one themed search ("chaplin" on Archive, "animation" on PeerTube...).
const props = defineProps({
  title: { type: String, required: true },
  query: { type: String, required: true },
  provider: { type: String, default: null },
})

const { t } = useI18n()
const { token } = useAuth()
const items = ref([])
const loading = ref(true)
const track = ref(null)

// Loads this row's results once on mount.
onMounted(async () => {
  try {
    items.value = await searchMedia(token.value, props.query, props.provider)
  } catch {
    items.value = []
  } finally {
    loading.value = false
  }
})

// Scrolls the track one "page" left (-1) or right (1).
function scroll(direction) {
  const el = track.value
  el.scrollBy({ left: direction * el.clientWidth * 0.85, behavior: 'smooth' })
}
</script>

<template>
  <!-- An empty row (source down, no results) disappears instead of showing a gap. -->
  <section v-if="loading || items.length > 0" class="row">
    <div class="row-head">
      <h2>{{ title }}</h2>
      <router-link v-if="!loading" :to="{ path: '/search', query: { q: query } }" class="see-all">{{ t('media.seeAll') }}</router-link>
    </div>

    <div class="row-body">
      <button class="arrow left" :aria-label="t('media.scrollLeft')" @click="scroll(-1)">‹</button>
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
      <button class="arrow right" :aria-label="t('media.scrollRight')" @click="scroll(1)">›</button>
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
  /* Without this, min-width: auto lets a large image stretch the card. */
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

/* Touch screens scroll by swiping: no arrows needed. */
@media (hover: none) {
  .arrow {
    display: none;
  }
}
</style>
