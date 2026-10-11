<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { BASE_URL, logout as apiLogout } from '../services/api'
import { useAuth } from '../services/auth'
import { useCurrentUser } from '../services/currentUser'
import { avatarSrc } from '../services/format'

const route = useRoute()
const router = useRouter()
const { token, clearToken } = useAuth()
const { user, load: loadUser } = useCurrentUser()

const query = ref(route.query.q || '')
const scrolled = ref(false)
const menuOpen = ref(false)
const menu = ref(null)

// Garde le champ synchronise si on arrive sur /search via un lien ou l'historique.
watch(() => route.query.q, (q) => {
  query.value = q || ''
})

watch(token, loadUser)

function onScroll() {
  scrolled.value = window.scrollY > 10
}

function onClickOutside(event) {
  if (menu.value && !menu.value.contains(event.target)) {
    menuOpen.value = false
  }
}

onMounted(() => {
  loadUser()
  window.addEventListener('scroll', onScroll, { passive: true })
  document.addEventListener('click', onClickOutside)
})

onBeforeUnmount(() => {
  window.removeEventListener('scroll', onScroll)
  document.removeEventListener('click', onClickOutside)
})

function handleSearch() {
  const q = query.value.trim()
  if (q) router.push({ path: '/search', query: { q } })
}

async function handleLogout() {
  menuOpen.value = false
  try {
    await apiLogout(token.value)
  } catch {
    // token deja invalide/expire cote serveur, on nettoie quand meme localement
  }
  clearToken()
  router.push('/login')
}
</script>

<template>
  <header class="header" :class="{ solid: scrolled || route.path !== '/' }">
    <router-link to="/" class="logo" aria-label="SyncWatch, accueil">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5v14l11-7z" /></svg>
      <span>SyncWatch</span>
    </router-link>

    <nav v-if="token" class="nav">
      <router-link to="/" exact-active-class="active">Accueil</router-link>
      <router-link to="/friends" active-class="active">Amis</router-link>
    </nav>

    <div class="spacer"></div>

    <form v-if="token" class="search" role="search" @submit.prevent="handleSearch">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="11" cy="11" r="7" fill="none" stroke="currentColor" stroke-width="2" />
        <path d="m20 20-3.5-3.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
      </svg>
      <input v-model="query" type="search" placeholder="Rechercher un film, une vidéo..." maxlength="100" aria-label="Rechercher" />
    </form>

    <div v-if="token" ref="menu" class="account">
      <button class="avatar-btn" :aria-expanded="menuOpen" aria-label="Menu du compte" @click="menuOpen = !menuOpen">
        <img :src="avatarSrc(BASE_URL, user?.avatar_url)" alt="" />
      </button>
      <div v-if="menuOpen" class="dropdown">
        <p v-if="user" class="dropdown-user">{{ user.username }}</p>
        <router-link to="/profile" @click="menuOpen = false">Mon profil</router-link>
        <router-link to="/friends" @click="menuOpen = false">Mes amis</router-link>
        <button @click="handleLogout">Se déconnecter</button>
      </div>
    </div>

    <template v-else>
      <router-link to="/login" class="btn btn-ghost btn-sm">Connexion</router-link>
      <router-link to="/signup" class="btn btn-primary btn-sm">S'inscrire</router-link>
    </template>
  </header>
</template>

<style scoped>
.header {
  position: fixed;
  inset: 0 0 auto;
  z-index: 50;
  height: var(--header-h);
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding: 0 var(--gutter);
  background: linear-gradient(to bottom, rgba(11, 11, 16, 0.85), transparent);
  transition: background 0.25s;
}

.header.solid {
  background: rgba(11, 11, 16, 0.92);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--border);
}

.logo {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-weight: 800;
  font-size: 1.2rem;
  color: var(--text-h);
  letter-spacing: -0.02em;
}

.logo:hover {
  text-decoration: none;
}

.logo svg {
  width: 26px;
  height: 26px;
  padding: 4px;
  fill: #fff;
  background: var(--accent);
  border-radius: 7px;
}

.nav {
  display: flex;
  gap: 1.25rem;
}

.nav a {
  color: var(--text);
  font-weight: 500;
  font-size: 0.95rem;
}

.nav a:hover,
.nav a.active {
  color: var(--text-h);
  text-decoration: none;
}

.spacer {
  flex: 1;
}

.search {
  position: relative;
  width: min(320px, 100%);
}

.search svg {
  position: absolute;
  left: 0.75rem;
  top: 50%;
  width: 18px;
  height: 18px;
  transform: translateY(-50%);
  color: var(--text);
  pointer-events: none;
}

.search input {
  width: 100%;
  padding: 0.5rem 0.9rem 0.5rem 2.4rem;
  border: 1px solid var(--border);
  border-radius: 999px;
  font: inherit;
  font-size: 0.9rem;
  color: var(--text-h);
  background: rgba(21, 21, 29, 0.8);
}

.search input:focus {
  outline: none;
  border-color: var(--accent);
}

.account {
  position: relative;
}

.avatar-btn {
  padding: 0;
  border: 2px solid transparent;
  border-radius: 50%;
  background: none;
  cursor: pointer;
  transition: border-color 0.15s;
}

.avatar-btn:hover,
.avatar-btn[aria-expanded='true'] {
  border-color: var(--accent);
}

.avatar-btn img {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  object-fit: cover;
  background: var(--bg-hover);
}

.dropdown {
  position: absolute;
  right: 0;
  top: calc(100% + 0.5rem);
  min-width: 180px;
  display: flex;
  flex-direction: column;
  padding: 0.4rem;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.dropdown-user {
  padding: 0.5rem 0.75rem;
  font-weight: 700;
  color: var(--text-h);
  border-bottom: 1px solid var(--border);
  margin-bottom: 0.25rem;
}

.dropdown a,
.dropdown button {
  padding: 0.5rem 0.75rem;
  border: none;
  border-radius: 6px;
  font: inherit;
  font-size: 0.9rem;
  text-align: left;
  color: var(--text-h);
  background: none;
  cursor: pointer;
}

.dropdown a:hover,
.dropdown button:hover {
  background: var(--bg-hover);
  text-decoration: none;
}

.dropdown button {
  color: var(--danger);
}

@media (max-width: 720px) {
  .header {
    gap: 0.75rem;
  }

  .logo span,
  .nav {
    display: none;
  }

  .search {
    flex: 1;
  }

  .spacer {
    display: none;
  }
}
</style>
