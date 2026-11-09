<script setup>
// FR/EN/ES toggle shown in the navbar, visible whether logged in or not.
import { useI18n } from 'vue-i18n'
import { setLocale } from '../i18n'

const LOCALES = [
  { code: 'fr', label: 'FR' },
  { code: 'en', label: 'EN' },
  { code: 'es', label: 'ES' },
]

const { locale, t } = useI18n()
</script>

<template>
  <div class="locale-switcher" role="group" :aria-label="t('nav.languageSwitcher')">
    <button
      v-for="l in LOCALES"
      :key="l.code"
      type="button"
      class="locale-btn"
      :class="{ active: locale === l.code }"
      :aria-pressed="locale === l.code"
      @click="setLocale(l.code)"
    >
      {{ l.label }}
    </button>
  </div>
</template>

<style scoped>
.locale-switcher {
  display: flex;
  gap: 0.15rem;
  padding: 0.2rem;
  flex: none;
  border-radius: 999px;
  background: var(--bg-hover);
}

.locale-btn {
  padding: 0.2rem 0.55rem;
  border: none;
  border-radius: 999px;
  font: inherit;
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--text);
  background: none;
  cursor: pointer;
}

.locale-btn.active {
  color: var(--text-h);
  background: var(--bg-elevated);
}
</style>
