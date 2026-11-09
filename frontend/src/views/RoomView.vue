<script setup>
// Watch-party room page: chat, member list, playback sync, and the
// owner-only video picker, all wired to the room WebSocket.
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import VideoPlayer from '../components/VideoPlayer.vue'
import { BASE_URL, getMediaDetails, getRoom, listRoomMessages, searchMedia } from '../services/api'
import { useAuth } from '../services/auth'
import { useCurrentUser } from '../services/currentUser'
import { PROVIDER_LABELS, avatarSrc } from '../services/format'
import { createPlaybackSync } from '../services/playbackSync'
import { createRoomSocket } from '../services/roomSocket'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const { token } = useAuth()
const { user, load: loadUser } = useCurrentUser()

// 6-digit code (string, never converted to Number: a leading zero like
// "012345" would get truncated by Number()).
const roomCode = computed(() => String(route.params.code))
const socket = createRoomSocket(token.value, user)
const { connected, room, members, media, messages, error } = socket

const history = ref([])
const newMessage = ref('')
const chatEl = ref(null)
const copied = ref(false)
const checking = ref(true)
const notFound = ref(false)

const isOwner = computed(() => room.value && user.value && room.value.owner_id === user.value.id)

// --- Video: loading details + sync ---
const mediaDetails = ref(null)
const mediaError = ref('')
const currentStream = computed(() => mediaDetails.value?.streams?.[0] ?? null)
const videoEl = ref(null)
const sync = createPlaybackSync(socket, videoEl)

// Receives the real <video> element from VideoPlayer and attaches sync to it.
function onVideoReady(el) {
  videoEl.value = el
  sync.attach(el)
}

// Reloads full media details whenever the room's current video changes.
watch(
  () => media.value,
  async (m) => {
    mediaDetails.value = null
    mediaError.value = ''
    if (!m) return
    try {
      mediaDetails.value = await getMediaDetails(token.value, m.provider, m.id)
    } catch (err) {
      mediaError.value = err.message
    }
  },
  { immediate: true },
)

// Starts clock calibration (ping/pong) only once the connection is truly
// ready, not before: pings sent too early would be lost (the WebSocket
// only accepts sends once open).
const stopWatchConnected = watch(connected, (isConnected) => {
  if (isConnected) {
    sync.start()
    stopWatchConnected()
  }
})

// --- Video picker (owner only) ---
const pickerOpen = ref(false)
const pickerQuery = ref('')
const pickerResults = ref([])
const pickerSearching = ref(false)

// Runs a media search inside the picker panel.
async function handlePickerSearch() {
  const q = pickerQuery.value.trim()
  if (!q) return
  pickerSearching.value = true
  try {
    pickerResults.value = await searchMedia(token.value, q)
  } catch {
    pickerResults.value = []
  } finally {
    pickerSearching.value = false
  }
}

// Sets the room's current video to the chosen item.
function selectMedia(item) {
  socket.send('playback.set_media', { provider: item.provider, id: item.id })
  pickerOpen.value = false
  pickerResults.value = []
  pickerQuery.value = ''
}

const inviteLink = computed(() => `${window.location.origin}/room/${roomCode.value}`)

// Copies the room's invite link to the clipboard.
async function copyInviteLink() {
  try {
    await navigator.clipboard.writeText(inviteLink.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // Clipboard unavailable (insecure context, permission denied...): the
    // link stays visible and selectable by hand.
  }
}

// History (REST, paginated, most recent first) + messages seen since the
// WS connection (already chronological): displayed one after the other.
const allMessages = computed(() => [...history.value, ...messages.value])

// Scrolls the chat panel to the latest message.
function scrollToBottom() {
  requestAnimationFrame(() => {
    if (chatEl.value) chatEl.value.scrollTop = chatEl.value.scrollHeight
  })
}

watch(() => allMessages.value.length, scrollToBottom)

onMounted(async () => {
  if (!user.value) await loadUser()

  // Check that the room actually exists BEFORE showing anything about it:
  // without this, a non-existent room would still display the whole frame
  // (chat, composer...) with just an error message buried inside, as if
  // we had almost managed to enter it.
  try {
    await getRoom(token.value, roomCode.value)
  } catch {
    notFound.value = true
    checking.value = false
    return
  }
  checking.value = false

  try {
    const recent = await listRoomMessages(token.value, roomCode.value)
    history.value = recent.reverse()
  } catch {
    // History is secondary: if it fails, live chat still works.
  }

  socket.connect()
  socket.joinRoom(roomCode.value)
})

onBeforeUnmount(() => {
  sync.stop()
  socket.leaveRoom()
  socket.close()
})

// Sends the composer's current message.
function handleSend() {
  const content = newMessage.value.trim()
  if (!content) return
  socket.sendChat(content)
  newMessage.value = ''
}

// Leaves the room and returns to the rooms lobby.
function leaveToLobby() {
  router.push('/rooms')
}
</script>

<template>
  <main class="page room">
    <div class="container">
      <div v-if="checking" class="skeleton" style="height: 320px"></div>

      <div v-else-if="notFound" class="panel not-found">
        <h1>{{ t('room.notFoundTitle') }}</h1>
        <p class="muted">{{ t('room.notFoundText', { code: roomCode }) }}</p>
        <router-link to="/rooms" class="btn btn-primary">{{ t('room.backToRooms') }}</router-link>
      </div>

      <template v-else>
        <div class="room-head">
          <div>
            <h1>{{ room?.name || t('room.fallbackName') }}</h1>
            <p class="muted">
              <span v-if="!connected">{{ t('room.connecting') }}</span>
              <span v-else>{{ t('room.participantsWithCode', { count: members.length, code: roomCode }, members.length) }}</span>
            </p>
          </div>
          <div class="room-actions">
            <button class="btn btn-ghost btn-sm" @click="copyInviteLink">
              {{ copied ? t('room.linkCopied') : t('room.copyLink') }}
            </button>
            <button class="btn btn-ghost btn-sm" @click="leaveToLobby">{{ t('room.leave') }}</button>
          </div>
        </div>

        <p v-if="error" class="msg-error">{{ error }}</p>

        <div class="room-body">
        <div class="video-slot panel">
          <div v-if="pickerOpen" class="picker">
            <div class="picker-head">
              <h2>{{ t('room.picker.title') }}</h2>
              <button class="btn btn-ghost btn-sm" @click="pickerOpen = false">{{ t('room.picker.cancel') }}</button>
            </div>
            <form class="picker-form" @submit.prevent="handlePickerSearch">
              <input
                v-model="pickerQuery"
                class="input"
                type="text"
                :placeholder="t('room.picker.searchPlaceholder')"
                maxlength="100"
              />
              <button type="submit" class="btn btn-primary btn-sm" :disabled="pickerSearching">
                {{ pickerSearching ? t('room.picker.searching') : t('room.picker.search') }}
              </button>
            </form>
            <ul v-if="pickerResults.length" class="picker-results">
              <li v-for="item in pickerResults" :key="`${item.provider}:${item.id}`">
                <button class="picker-result" @click="selectMedia(item)">
                  <img v-if="item.thumbnail" :src="item.thumbnail" alt="" />
                  <span class="picker-result-info">
                    <span class="picker-result-title">{{ item.title }}</span>
                    <span class="muted">{{ PROVIDER_LABELS[item.provider] }}<template v-if="item.year"> · {{ item.year }}</template></span>
                  </span>
                </button>
              </li>
            </ul>
            <p v-else-if="!pickerSearching && pickerQuery" class="muted">{{ t('room.picker.noResults') }}</p>
          </div>

          <template v-else-if="media">
            <div v-if="mediaError" class="video-message">
              <p class="msg-error">{{ t('room.videoLoadError') }}</p>
              <p class="muted">{{ mediaError }}</p>
            </div>
            <div v-else-if="!currentStream" class="skeleton" style="width: 100%; height: 100%"></div>
            <VideoPlayer
              v-else
              :src="currentStream.url"
              :kind="currentStream.kind"
              :poster="mediaDetails.backdrop || mediaDetails.thumbnail"
              @video-ready="onVideoReady"
            />
            <div v-if="sync.needsGesture.value" class="gesture-overlay">
              <button class="btn btn-primary" @click="sync.resume()">{{ t('room.joinInProgress') }}</button>
            </div>
            <button v-if="isOwner" class="btn btn-ghost btn-sm change-video" @click="pickerOpen = true">
              {{ t('room.changeVideo') }}
            </button>
          </template>

          <div v-else class="video-message">
            <p class="muted" v-if="isOwner">{{ t('room.noVideoOwner') }}</p>
            <p class="muted" v-else>{{ t('room.noVideoGuest') }}</p>
            <button v-if="isOwner" class="btn btn-primary btn-sm" @click="pickerOpen = true">
              {{ t('room.chooseVideo') }}
            </button>
          </div>
        </div>

        <aside class="chat panel">
          <div class="members">
            <span v-for="m in members" :key="m.id" class="member">
              <img :src="avatarSrc(BASE_URL, m.avatar_url)" alt="" />
              {{ m.username }}
            </span>
          </div>

          <div ref="chatEl" class="messages">
            <p v-if="allMessages.length === 0" class="muted empty">{{ t('room.noMessages') }}</p>
            <p v-for="msg in allMessages" :key="msg.id" class="message">
              <strong>{{ msg.sender.username }}</strong> {{ msg.content }}
            </p>
          </div>

          <form class="composer" @submit.prevent="handleSend">
            <input
              v-model="newMessage"
              class="input"
              type="text"
              maxlength="500"
              :placeholder="t('room.composerPlaceholder')"
              :disabled="!connected"
            />
            <button type="submit" class="btn btn-primary btn-sm" :disabled="!connected">{{ t('room.send') }}</button>
          </form>
        </aside>
        </div>
      </template>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 1100px;
  margin: 0 auto;
}

.room-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.not-found {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.75rem;
  max-width: 480px;
  margin: 2rem auto 0;
  text-align: left;
}

.room-actions {
  display: flex;
  gap: 0.5rem;
  flex-shrink: 0;
}

.room-body {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 1.5rem;
  align-items: stretch;
}

.video-slot {
  position: relative;
  aspect-ratio: 16 / 9;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding: 0;
}

.video-slot :deep(.player) {
  width: 100%;
  height: 100%;
  max-height: none;
  border-radius: 0;
  object-fit: contain;
}

.video-message {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem;
  text-align: center;
}

.change-video {
  position: absolute;
  top: 0.6rem;
  right: 0.6rem;
  background: rgba(11, 11, 16, 0.75);
  backdrop-filter: blur(4px);
}

.gesture-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(11, 11, 16, 0.6);
}

.picker {
  width: 100%;
  height: 100%;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  overflow-y: auto;
  text-align: left;
}

.picker-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.picker-form {
  display: flex;
  gap: 0.5rem;
}

.picker-form .input {
  flex: 1;
}

.picker-results {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.picker-result {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  width: 100%;
  padding: 0.5rem;
  border: 1px solid transparent;
  border-radius: var(--radius);
  background: var(--bg-hover);
  color: inherit;
  cursor: pointer;
  text-align: left;
}

.picker-result:hover {
  border-color: var(--accent);
}

.picker-result img {
  width: 64px;
  height: 36px;
  border-radius: 4px;
  object-fit: cover;
  background: var(--bg-elevated);
  flex-shrink: 0;
}

.picker-result-info {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  min-width: 0;
}

.picker-result-title {
  font-weight: 600;
  color: var(--text-h);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  max-height: 70vh;
}

.members {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border);
}

.member {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.8rem;
  color: var(--text);
}

.member img {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  object-fit: cover;
  background: var(--bg-hover);
}

.messages {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-height: 0;
}

.message {
  font-size: 0.9rem;
  color: var(--text);
  word-break: break-word;
}

.message strong {
  color: var(--text-h);
  margin-right: 0.35rem;
}

.empty {
  margin: auto 0;
  text-align: center;
}

.composer {
  display: flex;
  gap: 0.5rem;
}

@media (max-width: 900px) {
  .room-body {
    grid-template-columns: 1fr;
  }

  .chat {
    max-height: 50vh;
  }
}
</style>
