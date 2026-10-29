<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppHeader from './components/AppHeader.vue'

const route = useRoute()
const bare = computed(() => route.meta.bare)
</script>

<template>
  <AppHeader v-if="!bare" />
  <router-view />
  <footer v-if="!bare" class="footer">
    <p>SyncWatch · projet pédagogique · contenus du domaine public et sous licence libre</p>
    <nav class="footer-links">
      <router-link to="/privacy">Politique de confidentialité</router-link>
      <router-link to="/terms">Conditions générales d'utilisation</router-link>
    </nav>
  </footer>
</template>

<style scoped>
.footer {
  padding: 2rem var(--gutter);
  border-top: 1px solid var(--border);
  font-size: 0.8rem;
  color: var(--text);
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.footer-links {
  display: flex;
  justify-content: center;
  gap: 1.5rem;
}

.footer-links a {
  color: var(--text);
}

.footer-links a:hover {
  color: var(--text-h);
}
</style>
