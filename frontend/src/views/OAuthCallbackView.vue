<script setup>
// Landing page for the OAuth redirect from the backend, shown either
// inside the login popup or full-page if the popup was blocked.
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import AuthCard from '../components/AuthCard.vue'
import { useAuth } from '../services/auth'
import { OAUTH_CHANNEL, isSessionToken, oauthErrorCode } from '../services/oauth'

const { t } = useI18n()

// The backend redirects here after a Google/GitHub login:
//   success: /oauth/callback#token=...     failure: /oauth/callback?error=...
//
// This page may render either inside the popup or full-page (popup
// blocked). To find out, it announces the result on the channel: if a
// login page answers, we're in the popup and close it. Otherwise we
// handle things on our own.
const ACK_TIMEOUT_MS = 500

const route = useRoute()
const router = useRouter()
const { setToken } = useAuth()

const done = ref(false)
let channel = null
let timer = null

onMounted(() => {
  const token = new URLSearchParams(window.location.hash.slice(1)).get('token')
  const result = isSessionToken(token) ? { token } : { error: oauthErrorCode(route.query.error) }

  // The token leaves the address bar and history right away.
  window.history.replaceState(null, '', '/oauth/callback')

  // Same safeguard as in OAuthButtons.vue: a browser without
  // BroadcastChannel must not crash here, just behave as if nobody
  // answered (full page).
  try {
    channel = new BroadcastChannel(OAUTH_CHANNEL)
  } catch {
    channel = null
  }

  if (!channel) {
    finishAlone(result)
    return
  }

  channel.onmessage = (event) => {
    if (event.data?.type !== 'ack') return
    clearTimeout(timer)
    done.value = true
    window.close()
  }
  channel.postMessage({ type: 'result', ...result })

  timer = setTimeout(() => finishAlone(result), ACK_TIMEOUT_MS)
})

onBeforeUnmount(() => {
  clearTimeout(timer)
  channel?.close()
})

// Nobody answered: we're running full-page, handle the result ourselves.
function finishAlone(result) {
  if (result.token) {
    setToken(result.token)
    router.replace('/')
  } else {
    router.replace({ path: '/login', query: { error: result.error } })
  }
}
</script>

<template>
  <!-- If the browser refuses window.close(), tell the user. -->
  <AuthCard v-if="done" :title="t('oauthCallback.doneTitle')" :subtitle="t('oauthCallback.doneSubtitle')" />
  <AuthCard v-else :title="t('oauthCallback.pendingTitle')" :subtitle="t('oauthCallback.pendingSubtitle')" />
</template>
