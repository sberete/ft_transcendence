// Keeps a <video> element in sync with a room's shared playback state.
// Follows docs/websocket.md section 7: the server never streams the
// position continuously, only a reference point (position + server_time)
// that each client extrapolates from. See that document for the full algorithm.

import { ref } from 'vue'

const CORRECTION_INTERVAL_MS = 2000
const PING_INTERVAL_MS = 30000
const INITIAL_PING_COUNT = 5
const SMALL_DRIFT_S = 0.25
const LARGE_DRIFT_S = 2
const RATE_BEHIND = 1.05
const RATE_AHEAD = 0.95
// Beyond this much lag when attaching to a video, we consider that we're
// joining a playback already well underway: see `caughtUp` below.
const JOIN_THRESHOLD_S = 2
// After a genuine local action (clicking play/pause, or seeking), the
// server takes a moment to respond with the updated state. If the
// automatic correction lands right in that window, it still reads the OLD
// state (e.g. "not playing") and undoes what the user just did - typically
// pausing their own video right after starting it. So we ignore any
// correction for a short delay after a local action, long enough for the
// round trip to complete.
const LOCAL_ACTION_GRACE_MS = 1500

// `socket`: the object returned by createRoomSocket (roomSocket.js),
// already connected to the room. `videoRef`: an object with a .value
// property pointing at the current <video> element (or null if none loaded).
export function createPlaybackSync(socket, videoRef) {
  let bestRtt = Infinity
  let clockOffsetMs = 0
  let pingsSent = 0
  // Until when to ignore DOM events triggered by US (see withSuppression):
  // a timestamp rather than a boolean, so that two overlapping suppressions
  // (e.g. play() followed by a seek right after, the catch-up case) extend
  // each other instead of the first one cutting off the second if its own
  // window was shorter.
  let suppressUntil = 0
  let correctionTimer = null
  let pingTimer = null
  let attachedVideo = null
  let lastLocalActionAt = 0
  // True once THIS video has been aligned to the expected position at
  // least once. While false, correctDrift() touches nothing: joining a
  // room where playback is already well underway requires an explicit
  // gesture (the "Join ongoing playback" button), never an automatic
  // attempt - that's what used to cause visible resets for the whole room
  // (play() starting from position 0 before the catch-up seek had time to
  // apply, then that 0 position leaking back to the server if the
  // suppression window expired too early while the stream was still
  // buffering the target point).
  let caughtUp = false
  // Browsers block autoplay with sound until there has been a real click
  // on THIS specific video: a click elsewhere on the page (chat, header...)
  // doesn't count. This flag covers both that case and the explicit
  // catch-up above: in both cases the UI must show a button so the click
  // counts as a user gesture.
  const needsGesture = ref(false)

  // Server clock estimate, derived from the ping/pong round trips below.
  function serverNow() {
    return Date.now() + clockOffsetMs
  }

  // Sends one latency-measurement ping.
  function sendPing() {
    socket.send('ping', { t: Date.now() })
    pingsSent++
  }

  // Handles a pong reply: refines the clock offset estimate.
  function onMessage(msg) {
    if (msg.type !== 'pong') return
    const rtt = Date.now() - msg.t
    // Keep the least-disturbed measurement (smallest round trip), except at
    // the very start where the first 5 are used to find that minimum.
    if (rtt <= bestRtt || pingsSent <= INITIAL_PING_COUNT) {
      if (rtt <= bestRtt) {
        bestRtt = rtt
        clockOffsetMs = msg.server_time + rtt / 2 - Date.now()
      }
    }
  }

  // Computes where playback should be right now, per the server's reference point.
  function expectedPosition() {
    const pb = socket.playback.value
    if (!pb) return null
    if (pb.playing) {
      return pb.position + (serverNow() - pb.server_time) / 1000
    }
    return pb.position
  }

  // Runs `action` while marking the resulting DOM events as self-triggered.
  // play()/pause()/setting currentTime fire their events ('play', 'pause',
  // 'seeked') ASYNCHRONOUSLY: raising the flag right after the call
  // protects nothing, since the event arrives after the suppression has
  // already lapsed. The correction then ends up bouncing back to the
  // server as if it were a user action, with a slightly different
  // position that overwrites the correct state - this is what made
  // playback look "never in sync". So we keep the suppression up for a
  // whole window, not just the duration of the call - and extend it
  // rather than overwrite it if a second call overlaps the first.
  function withSuppression(action, windowMs) {
    suppressUntil = Math.max(suppressUntil, Date.now() + windowMs)
    action()
  }

  // Whether we're still inside a self-triggered suppression window.
  function isSuppressed() {
    return Date.now() < suppressUntil
  }

  // Periodic tick: aligns play/pause state and nudges/snaps the position to match the server.
  function correctDrift() {
    const video = videoRef.value
    const pb = socket.playback.value
    if (!video || !pb || video.readyState === 0) return
    if (!caughtUp) return
    if (Date.now() - lastLocalActionAt < LOCAL_ACTION_GRACE_MS) return

    // Play/pause state must match the server (e.g. another member paused)
    // before even correcting the position.
    if (pb.playing && video.paused) {
      withSuppression(() => {
        video.play().then(() => { needsGesture.value = false }).catch(() => { needsGesture.value = true })
      }, 300)
    } else if (!pb.playing && !video.paused) {
      withSuppression(() => video.pause(), 300)
    }

    const expected = expectedPosition()
    if (expected == null) return

    const drift = video.currentTime - expected
    if (Math.abs(drift) < SMALL_DRIFT_S) {
      video.playbackRate = 1
    } else if (Math.abs(drift) < LARGE_DRIFT_S) {
      // Behind (drift < 0): speed up slightly. Ahead: slow down slightly.
      video.playbackRate = drift < 0 ? RATE_BEHIND : RATE_AHEAD
    } else {
      // 'seeked' can take longer than a plain play/pause (depends on
      // network buffering): wider window.
      withSuppression(() => {
        video.currentTime = expected
        video.playbackRate = 1
      }, 800)
    }
  }

  // The three events a local change (clicking the native <video> controls)
  // fires. They're sent back to the server, unless it's US who just
  // changed the video to follow the server (isSuppressed): otherwise every
  // client would keep re-triggering all the others in a loop (section 7,
  // "Avoiding loops").
  function onPlay() {
    if (isSuppressed()) return
    lastLocalActionAt = Date.now()
    socket.send('playback.play', { position: videoRef.value.currentTime })
  }
  function onPause() {
    if (isSuppressed()) return
    lastLocalActionAt = Date.now()
    socket.send('playback.pause', { position: videoRef.value.currentTime })
  }
  function onSeeked() {
    if (isSuppressed()) return
    lastLocalActionAt = Date.now()
    socket.send('playback.seek', { position: videoRef.value.currentTime })
  }

  // Attaches sync to a new <video> element, replacing any previous one.
  function attach(video) {
    if (attachedVideo === video) return
    detach()
    needsGesture.value = false
    caughtUp = false
    if (!video) return
    video.addEventListener('play', onPlay)
    video.addEventListener('pause', onPause)
    video.addEventListener('seeked', onSeeked)
    attachedVideo = video

    // If playback is already well underway when we attach (joining a room
    // mid-session), don't try to catch up automatically: require the
    // explicit click instead of letting correctDrift() handle it on its
    // own every 2s.
    const expected = expectedPosition()
    if (expected != null && expected > JOIN_THRESHOLD_S) {
      needsGesture.value = true
    } else {
      caughtUp = true
    }
  }

  // Called from a real click handler (the "Join ongoing playback" button):
  // that counts as a direct user interaction on THIS element, which the
  // automatic call inside correctDrift() is not, so the browser allows it.
  // Aligns to the expected position first, THEN starts playback, without
  // reporting this back to the server as a new action: we're catching up,
  // not deciding anything. Once this catch-up succeeds, the usual
  // automatic corrections (nudge/seek) can resume for this client.
  function resume() {
    const video = videoRef.value
    if (!video) return
    withSuppression(() => {
      const expected = expectedPosition()
      if (expected != null) video.currentTime = expected
      video.play().then(() => { needsGesture.value = false; caughtUp = true }).catch(() => { needsGesture.value = true })
    }, 1000)
  }

  // Detaches sync from the current video, removing its event listeners.
  function detach() {
    if (!attachedVideo) return
    attachedVideo.removeEventListener('play', onPlay)
    attachedVideo.removeEventListener('pause', onPause)
    attachedVideo.removeEventListener('seeked', onSeeked)
    attachedVideo = null
  }

  // Starts the ping and drift-correction timers.
  function start() {
    socket.onMessage(onMessage)
    for (let i = 0; i < INITIAL_PING_COUNT; i++) sendPing()
    pingTimer = setInterval(sendPing, PING_INTERVAL_MS)
    correctionTimer = setInterval(correctDrift, CORRECTION_INTERVAL_MS)
  }

  // Stops the timers and detaches from the current video.
  function stop() {
    clearInterval(pingTimer)
    clearInterval(correctionTimer)
    detach()
  }

  return { attach, start, stop, resume, needsGesture }
}
