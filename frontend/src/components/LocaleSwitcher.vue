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
  <div
    class="flex shrink-0 gap-1 rounded-full bg-[var(--bg-hover)] p-1"
    role="group"
    :aria-label="t('nav.languageSwitcher')"
  >
    <button
      v-for="l in LOCALES"
      :key="l.code"
      type="button"
      class="cursor-pointer rounded-full border-none bg-transparent px-2.5 py-1 text-xs font-bold text-[var(--text)] transition-colors"
      :class="{ 'bg-[var(--bg-elevated)] text-[var(--text-h)]': locale === l.code }"
      :aria-pressed="locale === l.code"
      @click="setLocale(l.code)"
    >
      {{ l.label }}
    </button>
  </div>
</template>
