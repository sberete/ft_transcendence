<script setup>
// Privacy policy page: purely static legal text, rendered section by
// section from the i18n messages (legal.privacy.*) so it's fully translated.
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <main class="page">
    <div class="container">
      <h1>{{ t('legal.privacy.title') }}</h1>
      <p class="muted intro">{{ t('legal.privacy.updated') }}</p>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.whoWeAre.title') }}</h2>
        <p>{{ t('legal.privacy.sections.whoWeAre.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.dataCollected.title') }}</h2>
        <ul>
          <li><strong>{{ t('legal.privacy.dataItems.account.label') }}</strong> : {{ t('legal.privacy.dataItems.account.text') }}</li>
          <li><strong>{{ t('legal.privacy.dataItems.oauth.label') }}</strong> : {{ t('legal.privacy.dataItems.oauth.text') }}</li>
          <li><strong>{{ t('legal.privacy.dataItems.social.label') }}</strong> : {{ t('legal.privacy.dataItems.social.text') }}</li>
          <li><strong>{{ t('legal.privacy.dataItems.rooms.label') }}</strong> : {{ t('legal.privacy.dataItems.rooms.text') }}</li>
          <li><strong>{{ t('legal.privacy.dataItems.technical.label') }}</strong> : {{ t('legal.privacy.dataItems.technical.text') }}</li>
        </ul>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.why.title') }}</h2>
        <p>{{ t('legal.privacy.sections.why.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.externalSources.title') }}</h2>
        <p>{{ t('legal.privacy.sections.externalSources.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.cookies.title') }}</h2>
        <p>{{ t('legal.privacy.sections.cookies.before') }}<code>localStorage</code>{{ t('legal.privacy.sections.cookies.after') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.retention.title') }}</h2>
        <p>{{ t('legal.privacy.sections.retention.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.security.title') }}</h2>
        <p>{{ t('legal.privacy.sections.security.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.rights.title') }}</h2>
        <p>{{ t('legal.privacy.sections.rights.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.changes.title') }}</h2>
        <p>{{ t('legal.privacy.sections.changes.body') }}</p>
      </section>

      <section class="panel">
        <h2>{{ t('legal.privacy.sections.contact.title') }}</h2>
        <p class="muted">{{ t('legal.privacy.sections.contact.body') }}</p>
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

code {
  background: var(--bg-hover);
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-size: 0.9em;
}
</style>
