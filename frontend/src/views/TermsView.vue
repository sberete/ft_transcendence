<script setup>
import { useI18n } from 'vue-i18n'

const { t, tm } = useI18n()
</script>

<template>
  <main class="page">
    <div class="container">
      <h1>{{ t('legal.terms.title') }}</h1>
      <p class="muted intro">{{ t('legal.terms.updated') }}</p>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.purpose.title') }}</h2>
        <p>{{ t('legal.terms.sections.purpose.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.account.title') }}</h2>
        <p>{{ t('legal.terms.sections.account.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.allowedUse.title') }}</h2>
        <ul>
          <li v-for="(item, index) in tm('legal.terms.sections.allowedUse.items')" :key="index">{{ item }}</li>
        </ul>
      </section>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.yourContent.title') }}</h2>
        <p>{{ t('legal.terms.sections.yourContent.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.availability.title') }}</h2>
        <p>{{ t('legal.terms.sections.availability.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.termination.title') }}</h2>
        <p>{{ t('legal.terms.sections.termination.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.liability.title') }}</h2>
        <p>{{ t('legal.terms.sections.liability.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.changes.title') }}</h2>
        <p>{{ t('legal.terms.sections.changes.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.terms.sections.contact.title') }}</h2>
        <p class="muted">{{ t('legal.terms.sections.contact.body') }}</p>
      </section>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 720px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.intro {
  margin-top: -0.5rem;
}

.panel {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.panel p,
.panel li {
  color: var(--text);
  line-height: 1.6;
}

.panel ul {
  padding-left: 1.2rem;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
</style>
