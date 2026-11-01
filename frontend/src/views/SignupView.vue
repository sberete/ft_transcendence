<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import AuthCard from '../components/AuthCard.vue'
import OAuthButtons from '../components/OAuthButtons.vue'
import { signup } from '../services/api'
import { useAuth } from '../services/auth'

const { t } = useI18n()
const username = ref('')
const email = ref('')
const password = ref('')
const error = ref('')
const submitting = ref(false)
const router = useRouter()
const { setToken } = useAuth()

async function handleSubmit() {
  error.value = ''
  submitting.value = true
  try {
    const data = await signup(username.value, email.value, password.value)
    setToken(data.token)
    router.push('/')
  } catch (err) {
    error.value = err.message
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <AuthCard :title="t('auth.signup.title')" :subtitle="t('auth.signup.subtitle')">
    <form @submit.prevent="handleSubmit">
      <label class="field">
        {{ t('auth.signup.username') }}
        <input v-model="username" class="input" type="text" autocomplete="username" maxlength="32" required />
      </label>
      <label class="field">
        {{ t('auth.signup.email') }}
        <input v-model="email" class="input" type="email" autocomplete="email" required />
      </label>
      <label class="field">
        {{ t('auth.signup.password') }}
        <input
          v-model="password"
          class="input"
          type="password"
          autocomplete="new-password"
          minlength="8"
          :placeholder="t('auth.signup.passwordHint')"
          required
        />
      </label>
      <p v-if="error" class="msg-error">{{ error }}</p>
      <button type="submit" class="btn btn-primary btn-lg" :disabled="submitting">
        {{ submitting ? t('auth.signup.submitting') : t('auth.signup.submit') }}
      </button>
    </form>
    <OAuthButtons />
    <p class="switch muted">{{ t('auth.signup.haveAccount') }} <router-link to="/login">{{ t('auth.signup.loginLink') }}</router-link></p>
  </AuthCard>
</template>
