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
  </footer>
</template>

<style scoped>
.footer {
  padding: 2rem var(--gutter);
  border-top: 1px solid var(--border);
  font-size: 0.8rem;
  color: var(--text);
  text-align: center;
}
</style>
