<script setup>
// Cadre commun aux pages connexion / inscription.
defineProps({
  title: { type: String, required: true },
  subtitle: { type: String, default: '' },
})
</script>

<template>
  <main class="auth">
    <div class="auth-bg" aria-hidden="true"></div>
    <router-link to="/" class="logo">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5v14l11-7z" /></svg>
      SyncWatch
    </router-link>
    <div class="card">
      <h1>{{ title }}</h1>
      <p v-if="subtitle" class="muted subtitle">{{ subtitle }}</p>
      <slot />
    </div>
  </main>
</template>

<style scoped>
.auth {
  position: relative;
  min-height: 100svh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2rem;
  padding: 2rem 16px;
  overflow: hidden;
}

.auth-bg {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse at 30% 10%, rgba(139, 92, 246, 0.3), transparent 55%),
    radial-gradient(ellipse at 80% 90%, rgba(56, 189, 248, 0.15), transparent 50%);
}

.logo {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.4rem;
  font-weight: 800;
  color: var(--text-h);
}

.logo:hover {
  text-decoration: none;
}

.logo svg {
  width: 30px;
  height: 30px;
  padding: 5px;
  fill: #fff;
  background: var(--accent);
  border-radius: 8px;
}

.card {
  position: relative;
  width: 100%;
  max-width: 400px;
  padding: 2rem;
  background: rgba(21, 21, 29, 0.85);
  backdrop-filter: blur(12px);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow);
}

.card h1 {
  font-size: 1.6rem;
}

.subtitle {
  margin-top: 0.4rem;
}

.card :deep(form) {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-top: 1.5rem;
}

.card :deep(.switch) {
  margin-top: 1.5rem;
  text-align: center;
  font-size: 0.9rem;
}
</style>
