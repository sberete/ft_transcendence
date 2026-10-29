// Synchronise un element <video> sur l'etat de lecture partage d'un salon.
// Suit docs/websocket.md section 7 : le serveur ne diffuse jamais la position
// en continu, seulement un point de reference (position + server_time) que
// chaque client extrapole. Voir ce document pour le detail de l'algorithme.

const CORRECTION_INTERVAL_MS = 2000
const PING_INTERVAL_MS = 30000
const INITIAL_PING_COUNT = 5
const SMALL_DRIFT_S = 0.25
const LARGE_DRIFT_S = 2
const RATE_BEHIND = 1.05
const RATE_AHEAD = 0.95

// `socket` : l'objet renvoye par createRoomSocket (roomSocket.js), deja
// connecte a la salle. `videoRef` : un objet avec une propriete .value qui
// pointe vers l'element <video> courant (ou null si aucune video chargee).
export function createPlaybackSync(socket, videoRef) {
  let bestRtt = Infinity
  let clockOffsetMs = 0
  let pingsSent = 0
  let suppress = false
  let correctionTimer = null
  let pingTimer = null
  let attachedVideo = null

  function serverNow() {
    return Date.now() + clockOffsetMs
  }

  function sendPing() {
    socket.send('ping', { t: Date.now() })
    pingsSent++
  }

  function onMessage(msg) {
    if (msg.type !== 'pong') return
    const rtt = Date.now() - msg.t
    // Garde la mesure la moins perturbee (plus petit aller-retour), sauf au
    // tout debut ou les 5 premieres servent a trouver ce minimum.
    if (rtt <= bestRtt || pingsSent <= INITIAL_PING_COUNT) {
      if (rtt <= bestRtt) {
        bestRtt = rtt
        clockOffsetMs = msg.server_time + rtt / 2 - Date.now()
      }
    }
  }

  function expectedPosition() {
    const pb = socket.playback.value
    if (!pb) return null
    if (pb.playing) {
      return pb.position + (serverNow() - pb.server_time) / 1000
    }
    return pb.position
  }

  // `play()`/`pause()`/l'affectation de `currentTime` declenchent leurs
  // evenements ('play', 'pause', 'seeked') de facon ASYNCHRONE : relever le
  // drapeau tout de suite apres l'appel ne protege rien, l'evenement arrive
  // apres coup avec `suppress` deja retombe a false. Le correctif applique
  // alors fini par se renvoyer lui-meme au serveur comme une action
  // utilisateur, avec une position legerement differente qui ecrase le bon
  // etat : c'est ce qui rendait la lecture visiblement "jamais synchro".
  // On garde donc `suppress` leve pendant une fenetre, pas juste le temps
  // de l'appel.
  function withSuppression(action, windowMs) {
    suppress = true
    action()
    setTimeout(() => { suppress = false }, windowMs)
  }

  function correctDrift() {
    const video = videoRef.value
    const pb = socket.playback.value
    if (!video || !pb || video.readyState === 0) return

    // L'etat lecture/pause doit matcher le serveur (ex: un autre membre a mis
    // en pause) avant meme de corriger la position.
    if (pb.playing && video.paused) {
      withSuppression(() => video.play().catch(() => {}), 300)
    } else if (!pb.playing && !video.paused) {
      withSuppression(() => video.pause(), 300)
    }

    const expected = expectedPosition()
    if (expected == null) return

    const drift = video.currentTime - expected
    if (Math.abs(drift) < SMALL_DRIFT_S) {
      video.playbackRate = 1
    } else if (Math.abs(drift) < LARGE_DRIFT_S) {
      // En retard (drift < 0) : accelere legerement. En avance : ralentit.
      video.playbackRate = drift < 0 ? RATE_BEHIND : RATE_AHEAD
    } else {
      // 'seeked' peut prendre plus longtemps qu'un simple play/pause (depend
      // du buffering reseau) : fenetre plus large.
      withSuppression(() => {
        video.currentTime = expected
        video.playbackRate = 1
      }, 800)
    }
  }

  // Les trois evenements qu'un changement local (clic sur les controles natifs
  // du <video>) declenche. On les renvoie au serveur, sauf si c'est NOUS qui
  // venons de modifier la video pour suivre le serveur (suppress) : sinon
  // chaque client relancerait tous les autres en boucle (section 7, "Eviter
  // les boucles").
  function onPlay() {
    if (suppress) return
    socket.send('playback.play', { position: videoRef.value.currentTime })
  }
  function onPause() {
    if (suppress) return
    socket.send('playback.pause', { position: videoRef.value.currentTime })
  }
  function onSeeked() {
    if (suppress) return
    socket.send('playback.seek', { position: videoRef.value.currentTime })
  }

  function attach(video) {
    if (attachedVideo === video) return
    detach()
    if (!video) return
    video.addEventListener('play', onPlay)
    video.addEventListener('pause', onPause)
    video.addEventListener('seeked', onSeeked)
    attachedVideo = video
  }

  function detach() {
    if (!attachedVideo) return
    attachedVideo.removeEventListener('play', onPlay)
    attachedVideo.removeEventListener('pause', onPause)
    attachedVideo.removeEventListener('seeked', onSeeked)
    attachedVideo = null
  }

  function start() {
    socket.onMessage(onMessage)
    for (let i = 0; i < INITIAL_PING_COUNT; i++) sendPing()
    pingTimer = setInterval(sendPing, PING_INTERVAL_MS)
    correctionTimer = setInterval(correctDrift, CORRECTION_INTERVAL_MS)
  }

  function stop() {
    clearInterval(pingTimer)
    clearInterval(correctionTimer)
    detach()
  }

  return { attach, start, stop }
}
