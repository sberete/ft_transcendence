<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import AuthCard from '../components/AuthCard.vue'
import OAuthButtons from '../components/OAuthButtons.vue'
import { login } from '../services/api'
import { useAuth } from '../services/auth'
import { oauthErrorCode } from '../services/oauth'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const identifier = ref('')
const password = ref('')
// ?error=... : connexion Google/GitHub echouee en pleine page (popup bloquee).
const error = ref(route.query.error ? t(`oauth.errors.${oauthErrorCode(route.query.error)}`) : '')
const submitting = ref(false)
const { setToken } = useAuth()

// ?redirect= vient de l'URL, donc de n'importe qui : on n'accepte qu'un chemin
// interne ("/watch/..."), jamais "//evil.com" ou "https://..." (open redirect).
function safeRedirect(value) {
  return typeof value === 'string' && value.startsWith('/') && !value.startsWith('//') ? value : '/'
}

async function handleSubmit() {
  error.value = ''
  submitting.value = true
  try {
    const data = await login(identifier.value, password.value)
    setToken(data.token)
    router.push(safeRedirect(route.query.redirect))
  } catch (err) {
    error.value = err.message
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <AuthCard :title="t('auth.login.title')" :subtitle="t('auth.login.subtitle')">
    <form @submit.prevent="handleSubmit">
      <label class="field">
        {{ t('auth.login.identifier') }}
        <input v-model="identifier" class="input" type="text" autocomplete="username" required />
      </label>
      <label class="field">
        {{ t('auth.login.password') }}
        <input v-model="password" class="input" type="password" autocomplete="current-password" required />
      </label>
      <p v-if="error" class="msg-error">{{ error }}</p>
      <button type="submit" class="btn btn-primary btn-lg" :disabled="submitting">
        {{ submitting ? t('auth.login.submitting') : t('auth.login.submit') }}
      </button>
    </form>
    <OAuthButtons />
    <p class="switch muted">{{ t('auth.login.noAccount') }} <router-link to="/signup">{{ t('auth.login.signupLink') }}</router-link></p>
  </AuthCard>
</template>
