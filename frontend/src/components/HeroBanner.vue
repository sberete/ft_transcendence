<script setup>
// Large featured-video banner shown at the top of the home page.
import { computed, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getMediaDetails } from '../services/api'
import { useAuth } from '../services/auth'
import { PROVIDER_LABELS, formatDuration, htmlToText } from '../services/format'

const props = defineProps({
  provider: { type: String, required: true },
  id: { type: String, required: true },
})

const { t } = useI18n()
const { token } = useAuth()
const media = ref(null)

const description = computed(() => htmlToText(media.value?.description))
// Builds the CSS background-image for the backdrop. The URL comes from an
// external source: JSON.stringify wraps it in escaped quotes, so a
// parenthesis inside the URL can't break out of url(...).
const backdropStyle = computed(() => {
  const url = media.value?.backdrop || media.value?.thumbnail
  return url ? { backgroundImage: `url(${JSON.stringify(url)})` } : null
})
const watchLink = computed(() => `/watch/${props.provider}/${encodeURIComponent(props.id)}`)

// Loads the featured item's details once on mount.
onMounted(async () => {
  try {
    media.value = await getMediaDetails(token.value, props.provider, props.id)
  } catch {
    media.value = null
  }
})
</script>

<template>
  <section class="hero">
    <div
      class="backdrop"
      :style="backdropStyle"
    ></div>
    <div class="shade"></div>

    <div class="content">
      <p class="badge">{{ t('hero.featured') }}</p>
      <template v-if="media">
        <h1>{{ media.title }}</h1>
        <p class="meta">
          {{ PROVIDER_LABELS[media.provider] }}
          <template v-if="media.year"> · {{ media.year }}</template>
          <template v-if="media.duration_seconds"> · {{ formatDuration(media.duration_seconds) }}</template>
        </p>
        <p v-if="description" class="desc">{{ description }}</p>
        <div class="actions">
          <router-link :to="watchLink" class="btn btn-primary btn-lg">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor" aria-hidden="true"><path d="M8 5v14l11-7z" /></svg>
            {{ t('hero.watch') }}
          </router-link>
        </div>
      </template>
      <template v-else>
        <div class="skeleton" style="height: 2.5rem; width: 60%"></div>
        <div class="skeleton" style="height: 1rem; width: 40%; margin-top: 1rem"></div>
        <div class="skeleton" style="height: 3rem; width: 160px; margin-top: 1.5rem"></div>
      </template>
    </div>
  </section>
</template>

<style scoped>
.hero {
  position: relative;
  min-height: min(78vh, 680px);
  display: flex;
  align-items: flex-end;
  padding: calc(var(--header-h) + 2rem) var(--gutter) 4rem;
  margin-bottom: 1rem;
  overflow: hidden;
}

.backdrop {
  position: absolute;
  inset: 0;
  background: var(--bg-elevated) center / cover no-repeat;
}

/* Gradient so the text stays readable regardless of the image. */
.shade {
  position: absolute;
  inset: 0;
  background:
    linear-gradient(to top, var(--bg) 0%, rgba(11, 11, 16, 0.6) 35%, transparent 70%),
    linear-gradient(to right, rgba(11, 11, 16, 0.85) 0%, transparent 65%);
}

.content {
  position: relative;
  max-width: 620px;
}

.badge {
  display: inline-block;
  padding: 0.2rem 0.6rem;
  margin-bottom: 0.9rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--accent-hover);
  background: var(--accent-soft);
}

.meta {
  margin-top: 0.6rem;
  font-weight: 500;
}

.desc {
  margin-top: 1rem;
  color: #d4d4de;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  white-space: pre-line;
}

.actions {
  margin-top: 1.5rem;
  display: flex;
  gap: 0.75rem;
}
</style>
