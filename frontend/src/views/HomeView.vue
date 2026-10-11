<script setup>
import HeroBanner from '../components/HeroBanner.vue'
import MediaRow from '../components/MediaRow.vue'
import { useAuth } from '../services/auth'

const { token } = useAuth()

// Video mise en avant : Big Buck Bunny (Blender Foundation, licence CC-BY) sur Framatube.
const FEATURED = { provider: 'peertube', id: '1f5bfc59-998b-41b3-9be3-e8084ad1a2a1' }

// Chaque rangee est une recherche ciblee sur une source.
const ROWS = [
  { title: 'Animation libre', query: 'blender', provider: 'peertube' },
  { title: 'Charlie Chaplin', query: 'chaplin', provider: 'archive' },
  { title: "Frissons d'époque", query: 'dracula', provider: 'archive' },
  { title: 'Buster Keaton', query: 'keaton', provider: 'archive' },
  { title: 'Documentaires', query: 'documentaire', provider: 'peertube' },
  { title: 'Sherlock Holmes', query: 'sherlock holmes', provider: 'archive' },
  { title: "Dessins animés d'antan", query: 'popeye', provider: 'archive' },
  { title: 'Hitchcock', query: 'hitchcock', provider: 'archive' },
]
</script>

<template>
  <main v-if="token">
    <HeroBanner :provider="FEATURED.provider" :id="FEATURED.id" />
    <MediaRow v-for="row in ROWS" :key="row.query" :title="row.title" :query="row.query" :provider="row.provider" />
  </main>

  <main v-else class="landing">
    <div class="landing-bg" aria-hidden="true"></div>
    <div class="landing-content">
      <h1>Regardez ensemble,<br />où que vous soyez.</h1>
      <p class="lead">
        Des centaines de films du domaine public et de vidéos libres, à regarder seul ou à plusieurs, synchronisés en
        temps réel avec vos amis.
      </p>
      <div class="cta">
        <router-link to="/signup" class="btn btn-primary btn-lg">Créer un compte</router-link>
        <router-link to="/login" class="btn btn-ghost btn-lg">J'ai déjà un compte</router-link>
      </div>

      <ul class="features">
        <li>
          <strong>100 % légal</strong>
          <span>Internet Archive et PeerTube, contenus libres de droits.</span>
        </li>
        <li>
          <strong>Entre amis</strong>
          <span>Ajoutez vos amis et voyez qui est en ligne.</span>
        </li>
        <li>
          <strong>Sans hébergement</strong>
          <span>Les vidéos sont lues directement depuis leur source.</span>
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
