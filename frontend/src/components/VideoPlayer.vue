<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import Hls from 'hls.js'

// kind : 'mp4' (lu nativement par <video>) ou 'hls' (playlist .m3u8 decoupee
// en segments, lue via hls.js sauf sur Safari qui la gere nativement).
const props = defineProps({
  src: { type: String, required: true },
  kind: { type: String, required: true },
  // Image affichee avant le lancement (sinon un rectangle noir).
  poster: { type: String, default: null },
})

// Emis en plus de defineExpose({ video }) : un parent qui recupere l'element
// <video> expose a travers un ref de composant doit deballer DEUX niveaux de
// ref (componentRef.value.video.value), ce qui s'est avere peu fiable en
// pratique (voir RoomView.vue). Un evenement est sans ambiguite : la valeur
// recue est directement l'element, jamais un ref a deballer.
const emit = defineEmits(['video-ready'])

const video = ref(null)
watch(video, (el) => emit('video-ready', el))
let hls = null

function load() {
  if (hls) {
    hls.destroy()
    hls = null
  }

  if (props.kind === 'hls' && Hls.isSupported()) {
    hls = new Hls()
    hls.loadSource(props.src)
    hls.attachMedia(video.value)
  } else {
    video.value.src = props.src
  }
}

onMounted(load)
watch(() => [props.src, props.kind], load)
onBeforeUnmount(() => hls?.destroy())

// Expose l'element <video> : la future sync de room (play/pause/seek) en aura besoin.
defineExpose({ video })
</script>

<template>
  <video ref="video" controls playsinline preload="metadata" :poster="poster" class="player"></video>
</template>

<style scoped>
.player {
  width: 100%;
  max-height: 70vh;
  background: #000;
  border-radius: 4px;
}
</style>
