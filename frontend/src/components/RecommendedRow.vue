<script setup>
// "Recommended for you" row on the home page, backed by the hybrid
// recommendation endpoint rather than a search query.
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import MediaCard from './MediaCard.vue'
import { getRecommendations } from '../services/api'
import { useAuth } from '../services/auth'

// Same layout as MediaRow (horizontal scroll), but the data source and
// response shape differ (score/reason on top, no originating search
// query): a separate component rather than forcing MediaRow to handle two
// data shapes.
const { t } = useI18n()
const { token } = useAuth()
const items = ref([])
const loading = ref(true)
const track = ref(null)

// Loads the current user's recommendations once on mount.
onMounted(async () => {
  try {
    items.value = await getRecommendations(token.value)
  } catch {
    items.value = []
  } finally {
    loading.value = false
  }
})

// Builds the short "why this is recommended" caption for one item.
function reason(item) {
  if (item.co_watched_by > 0) return t('recommendations.coWatched', { count: item.co_watched_by }, item.co_watched_by)
  if (item.based_on_title) return t('recommendations.basedOn', { title: item.based_on_title })
  return ''
}

// Scrolls the track one "page" left (-1) or right (1).
function scroll(direction) {
  const el = track.value
  el.scrollBy({ left: direction * el.clientWidth * 0.85, behavior: 'smooth' })
}
</script>

<template>
  <!-- No watch history yet -> nothing to show, the row disappears instead
       of showing an empty state on the home page. -->
  <section v-if="loading || items.length > 0" class="row">
    <div class="row-head">
      <h2>{{ t('recommendations.title') }}</h2>
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
          <p v-if="reason(item)" class="reason">{{ reason(item) }}</p>
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

.reason {
  margin-top: 0.3rem;
  font-size: 0.78rem;
  color: var(--accent-hover);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

@media (hover: none) {
  .arrow {
    display: none;
  }
}
</style>
