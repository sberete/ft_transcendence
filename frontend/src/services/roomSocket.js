import { ref } from 'vue'
import { BASE_URL } from './api'

const RECONNECT_BASE_MS = 1000
const RECONNECT_MAX_MS = 30000

// Une instance par salle visitee (pas un singleton partage comme useAuth) :
// cree a l'entree dans la salle, fermee a la sortie.
// `self` (ref vers {id, username, avatar_url}) : room.state du serveur ne liste
// que les AUTRES membres deja presents (pas soi-meme), donc on se rajoute cote
// client pour que la liste des participants soit complete. C'est un ref (pas
// une valeur figee) car le profil peut encore charger au moment de connect().
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

  function send(type, payload = {}) {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type, ...payload }))
    }
  }

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
        break
      case 'room.closed':
        error.value = 'Cette salle a ete fermee.'
        break
      case 'chat.message':
        messages.value = [...messages.value, msg]
        break
      case 'playback.state':
        media.value = msg.media
        playback.value = { playing: msg.playing, position: msg.position, server_time: msg.server_time }
        break
      case 'error':
        error.value = msg.message
        break
      default:
        break
    }

    for (const listener of listeners) listener(msg)
  }

  function connect() {
    manuallyClosed = false
    socket = new WebSocket(`${BASE_URL.replace(/^http/, 'ws')}/api/ws`)

    socket.onopen = () => send('auth', { token })
    socket.onmessage = handleMessage

    socket.onclose = (event) => {
      connected.value = false
      // 4401 = token invalide : inutile de reessayer, il ne redeviendra pas valide.
      if (manuallyClosed || event.code === 4401) return

      reconnectTimer = setTimeout(connect, reconnectDelay)
      reconnectDelay = Math.min(reconnectDelay * 2, RECONNECT_MAX_MS)
    }
  }

  function joinRoom(code) {
    currentCode = code
    if (connected.value) send('room.join', { code })
  }

  function leaveRoom() {
    send('room.leave')
    currentCode = null
    room.value = null
    members.value = []
    media.value = null
    playback.value = null
    messages.value = []
  }

  function sendChat(content) {
    send('chat.send', { content })
  }

  // Pour frontend-sync : ecouter playback.state sans dupliquer ce fichier.
  function onMessage(listener) {
    listeners.push(listener)
  }

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
