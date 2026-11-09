<script setup>
// Home page: hero banner, recommendations, and themed rows when logged
// in; a marketing landing page otherwise.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import HeroBanner from '../components/HeroBanner.vue'
import MediaRow from '../components/MediaRow.vue'
import RecommendedRow from '../components/RecommendedRow.vue'
import { useAuth } from '../services/auth'

const { t } = useI18n()
const { token } = useAuth()

// Featured video: Big Buck Bunny (Blender Foundation, CC-BY license) on Framatube.
const FEATURED = { provider: 'peertube', id: '1f5bfc59-998b-41b3-9be3-e8084ad1a2a1' }

// Each row is a search targeted at one source. `query` goes to the backend
// as-is (search terms, never translated); only `titleKey` (the displayed
// label) follows the active locale.
const ROWS = [
  { titleKey: 'home.rows.animation', query: 'blender', provider: 'peertube' },
  { titleKey: 'home.rows.chaplin', query: 'chaplin', provider: 'archive' },
  { titleKey: 'home.rows.era', query: 'dracula', provider: 'archive' },
  { titleKey: 'home.rows.keaton', query: 'keaton', provider: 'archive' },
  { titleKey: 'home.rows.documentaries', query: 'documentaire', provider: 'peertube' },
  { titleKey: 'home.rows.sherlock', query: 'sherlock holmes', provider: 'archive' },
  { titleKey: 'home.rows.cartoons', query: 'popeye', provider: 'archive' },
  { titleKey: 'home.rows.hitchcock', query: 'hitchcock', provider: 'archive' },
]

const rows = computed(() => ROWS.map((row) => ({ ...row, title: t(row.titleKey) })))
</script>

<template>
  <main v-if="token">
    <HeroBanner :provider="FEATURED.provider" :id="FEATURED.id" />
    <RecommendedRow />
    <MediaRow v-for="row in rows" :key="row.query" :title="row.title" :query="row.query" :provider="row.provider" />
  </main>

  <main v-else class="landing">
    <div class="landing-bg" aria-hidden="true"></div>
    <div class="landing-content">
      <h1>{{ t('home.landing.titleLine1') }}<br />{{ t('home.landing.titleLine2') }}</h1>
      <p class="lead">{{ t('home.landing.lead') }}</p>
      <div class="cta">
        <router-link to="/signup" class="btn btn-primary btn-lg">{{ t('home.landing.createAccount') }}</router-link>
        <router-link to="/login" class="btn btn-ghost btn-lg">{{ t('home.landing.haveAccount') }}</router-link>
      </div>

      <ul class="features">
        <li>
          <strong>{{ t('home.landing.features.legal.title') }}</strong>
          <span>{{ t('home.landing.features.legal.desc') }}</span>
        </li>
        <li>
          <strong>{{ t('home.landing.features.friends.title') }}</strong>
          <span>{{ t('home.landing.features.friends.desc') }}</span>
        </li>
        <li>
          <strong>{{ t('home.landing.features.hosting.title') }}</strong>
          <span>{{ t('home.landing.features.hosting.desc') }}</span>
        </li>
      </ul>
    </div>
  </main>
</template>

<style scoped>
.landing {
  position: relative;
  min-height: 100svh;
  display: flex;
  align-items: center;
  padding: calc(var(--header-h) + 2rem) var(--gutter) 3rem;
  overflow: hidden;
}

.landing-bg {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse at 20% 20%, rgba(139, 92, 246, 0.35), transparent 55%),
    radial-gradient(ellipse at 85% 75%, rgba(56, 189, 248, 0.18), transparent 50%);
}

.landing-content {
  position: relative;
  max-width: 720px;
}

.landing h1 {
  font-size: clamp(2.25rem, 6vw, 4rem);
}

.lead {
  margin-top: 1.25rem;
  font-size: 1.15rem;
  max-width: 560px;
}

.cta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-top: 2rem;
}

.features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 1rem;
  list-style: none;
  padding: 0;
  margin: 3.5rem 0 0;
}

.features li {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 1rem;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: rgba(21, 21, 29, 0.6);
  font-size: 0.9rem;
}

.features strong {
  color: var(--text-h);
}
</style>
