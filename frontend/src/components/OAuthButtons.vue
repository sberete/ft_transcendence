<script setup>
// Google/GitHub login buttons: opens a popup, listens for its result over
// BroadcastChannel, and falls back to a full-page redirect when a popup
// can't be used.
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { oauthStartUrl } from '../services/api'
import { useAuth } from '../services/auth'
import { OAUTH_CHANNEL, isSessionToken, oauthErrorCode } from '../services/oauth'

// Both buttons are always shown. If a provider isn't configured
// server-side, the backend returns an error code instead.
// Brand names: never translated, even in English.
const PROVIDERS = ['google', 'github']
const LABELS = { google: 'Google', github: 'GitHub' }
const POPUP_WIDTH = 520
const POPUP_HEIGHT = 680

const { t } = useI18n()
const router = useRouter()
const { setToken } = useAuth()
const error = ref('')
let channel = null

onMounted(() => {
  // Once the popup lands back on our site, it announces the result here.
  // BroadcastChannel is supported by every common browser (Safari since
  // 15.4), but an older or unusual browser could throw synchronously here
  // - without this try/catch, that broke the whole component's mount
  // instead of degrading gracefully.
  try {
    channel = new BroadcastChannel(OAUTH_CHANNEL)
  } catch {
    channel = null
  }
  if (!channel) return

  channel.onmessage = (event) => {
    if (event.data?.type !== 'result') return
    // Acknowledge receipt: the popup knows it can close itself now.
    channel.postMessage({ type: 'ack' })

    if (isSessionToken(event.data.token)) {
      setToken(event.data.token)
      router.push('/')
    } else {
      error.value = t(`oauth.errors.${oauthErrorCode(event.data.error)}`)
    }
  }
})

onBeforeUnmount(() => channel?.close())

// Opens the provider's login popup, falling back to full-page navigation
// if no channel is available or the popup gets blocked.
function openPopup(provider) {
  error.value = ''

  // With no channel to receive the result, opening a popup would be
  // pointless (it would have no way to tell us): go straight to full-page,
  // as if the popup had been blocked.
  if (!channel) {
    window.location.href = oauthStartUrl(provider)
    return
  }

  const left = window.screenX + Math.max(0, (window.outerWidth - POPUP_WIDTH) / 2)
  const top = window.screenY + Math.max(0, (window.outerHeight - POPUP_HEIGHT) / 2)
  const popup = window.open(
    oauthStartUrl(provider),
    'syncwatch-oauth',
    `popup,width=${POPUP_WIDTH},height=${POPUP_HEIGHT},left=${left},top=${top}`,
  )

  // Popup blocked by the browser: fall back to full-page navigation.
  if (!popup) {
    window.location.href = oauthStartUrl(provider)
  }
}
</script>

<template>
  <div class="oauth">
    <div class="divider"><span>{{ t('oauth.divider') }}</span></div>
    <!-- A real link (href): right-click and "open in new tab" still work. -->
    <a
      v-for="provider in PROVIDERS"
      :key="provider"
      :href="oauthStartUrl(provider)"
      class="btn btn-ghost btn-lg oauth-btn"
      @click.prevent="openPopup(provider)"
    >
      <!-- Four-color "G" logo: traced from Google's official sign-in button. -->
      <svg v-if="provider === 'google'" class="logo" viewBox="0 0 48 48" aria-hidden="true">
        <path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z" />
        <path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z" />
        <path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z" />
        <path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z" />
      </svg>
      <!-- GitHub logo: official mark (simple-icons). currentColor = button text color. -->
      <svg v-else-if="provider === 'github'" class="logo" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
      </svg>
      <span>{{ t('oauth.continueWith', { provider: LABELS[provider] }) }}</span>
    </a>
    <p v-if="error" class="msg-error" role="alert">{{ error }}</p>
  </div>
</template>

<style scoped>
.oauth {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-top: 1.25rem;
}

.oauth-btn {
  gap: 0.75rem;
}

.logo {
  width: 20px;
  height: 20px;
  flex: none;
}

.divider {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 0.8rem;
  color: var(--text);
}

.divider::before,
.divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--border);
}
</style>
