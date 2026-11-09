// WebSocket client for a single room: connects, authenticates, joins, and
// exposes the live room/members/media/playback/chat state as refs. See
// docs/websocket.md for the wire protocol this implements.
import { ref } from 'vue'
import { BASE_URL } from './api'

const RECONNECT_BASE_MS = 1000
const RECONNECT_MAX_MS = 30000

// One instance per room visited (not a shared singleton like useAuth):
// created on entering the room, closed on leaving.
// `self` (ref to {id, username, avatar_url}): the server's room.state only
// lists the OTHER members already present (not oneself), so we add
// ourselves back on the client so the participant list is complete. It's a
// ref (not a frozen value) because the profile may still be loading at connect() time.
export function createRoomSocket(token, self) {
  const connected = ref(false)
  const room = ref(null)
  const members = ref([])
  const media = ref(null)
  const playback = ref(null)
  const messages = ref([])
  const error = ref('')

  let socket = null
  let currentCode = null
  let reconnectDelay = RECONNECT_BASE_MS
  let reconnectTimer = null
  let manuallyClosed = false
  const listeners = []

  // Sends one JSON frame if the socket is currently open.
  function send(type, payload = {}) {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type, ...payload }))
    }
  }

  // Parses an incoming frame and updates local state for its type.
  function handleMessage(event) {
    let msg
    try {
      msg = JSON.parse(event.data)
    } catch {
      return
    }

    switch (msg.type) {
      case 'auth.ok':
        connected.value = true
        error.value = ''
        reconnectDelay = RECONNECT_BASE_MS
        if (currentCode) send('room.join', { code: currentCode })
        break
      case 'room.state':
        room.value = msg.room
        members.value = [self.value, ...msg.members]
        media.value = msg.media
        playback.value = msg.playback
        break
      case 'room.member_joined':
        if (msg.user.id !== self.value?.id) {
          members.value = [...members.value, msg.user]
        }
        break
      case 'room.member_left':
        members.value = members.value.filter((m) => m.id !== msg.user_id)
        // If the one leaving was the owner, ownership was transferred to
        // whoever has been present the longest (see registry::leave on the
        // backend): reflect that immediately, without waiting for a room.state.
        if (msg.new_owner_id != null && room.value) {
          room.value = { ...room.value, owner_id: msg.new_owner_id }
        }
        break
      case 'chat.message':
        messages.value = [...messages.value, msg]
        break
      case 'playback.state': {
        // playback.state is broadcast for EVERY action (play/pause/seek),
        // not just a media change, but `media` is always present in the
        // message. Reassigning it unconditionally would produce a new
        // object every time (freshly parsed JSON) even for the exact same
        // video - that alone was enough to make the player remount
        // (RoomView.vue watches media.value) on every play/pause click,
        // resetting playback to zero repeatedly. So we only reassign when
        // the content actually changed.
        const same = media.value?.provider === msg.media?.provider && media.value?.id === msg.media?.id
        if (!same) media.value = msg.media
        playback.value = { playing: msg.playing, position: msg.position, server_time: msg.server_time }
        break
      }
      case 'error':
        error.value = msg.message
        break
      default:
        break
    }

    for (const listener of listeners) listener(msg)
  }

  // Opens the WebSocket connection and wires up its handlers.
  function connect() {
    manuallyClosed = false
    socket = new WebSocket(`${BASE_URL.replace(/^http/, 'ws')}/api/ws`)

    socket.onopen = () => send('auth', { token })
    socket.onmessage = handleMessage

    socket.onclose = (event) => {
      connected.value = false
      // 4401 = invalid token: no point retrying, it won't become valid again.
      if (manuallyClosed || event.code === 4401) return

      reconnectTimer = setTimeout(connect, reconnectDelay)
      reconnectDelay = Math.min(reconnectDelay * 2, RECONNECT_MAX_MS)
    }
  }

  // Joins (or queues joining, if not yet connected) the given room code.
  function joinRoom(code) {
    currentCode = code
    if (connected.value) send('room.join', { code })
  }

  // Leaves the current room and resets all local room state.
  function leaveRoom() {
    send('room.leave')
    currentCode = null
    room.value = null
    members.value = []
    media.value = null
    playback.value = null
    messages.value = []
  }

  // Sends a chat message to the current room.
  function sendChat(content) {
    send('chat.send', { content })
  }

  // Registers an extra listener for every incoming frame.
  // Used by playbackSync.js to listen for playback.state without duplicating this file.
  function onMessage(listener) {
    listeners.push(listener)
  }

  // Closes the connection for good (no auto-reconnect afterwards).
  function close() {
    manuallyClosed = true
    clearTimeout(reconnectTimer)
    socket?.close(1000)
  }

  return {
    connected,
    room,
    members,
    media,
    playback,
    messages,
    error,
    connect,
    joinRoom,
    leaveRoom,
    sendChat,
    send,
    onMessage,
    close,
  }
}
