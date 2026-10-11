<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AuthCard from '../components/AuthCard.vue'
import { useAuth } from '../services/auth'
import { OAUTH_CHANNEL, isSessionToken, oauthErrorCode } from '../services/oauth'

// Le backend renvoie ici apres une connexion Google/GitHub :
//   succes : /oauth/callback#token=...     echec : /oauth/callback?error=...
//
// Cette page s'affiche soit dans la popup, soit en pleine page (popup bloquee).
// Pour le savoir, elle annonce le resultat sur le canal : si une page de
// connexion repond, on est dans la popup et on se ferme. Sinon on gere seul.
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

  // Le token quitte la barre d'adresse et l'historique tout de suite.
  window.history.replaceState(null, '', '/oauth/callback')

  channel = new BroadcastChannel(OAUTH_CHANNEL)
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

// Personne n'a repondu : on est en pleine page.
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
  <!-- Si le navigateur refuse window.close(), on le dit a l'utilisateur. -->
  <AuthCard v-if="done" title="C'est fait" subtitle="Vous pouvez fermer cette fenêtre." />
  <AuthCard v-else title="Connexion en cours…" subtitle="Un instant." />
</template>
