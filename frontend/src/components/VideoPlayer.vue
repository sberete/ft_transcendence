<script setup>
// <video> wrapper that picks the right playback strategy per stream kind
// and browser, and exposes the underlying element to its parent.
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import Hls from 'hls.js'

// kind: 'mp4' (played natively by <video>) or 'hls' (.m3u8 playlist split
// into segments, played via hls.js except on Safari which handles it natively).
const props = defineProps({
  src: { type: String, required: true },
  kind: { type: String, required: true },
  // Image shown before playback starts (otherwise a black rectangle).
  poster: { type: String, default: null },
})

// Emitted in addition to defineExpose({ video }): a parent that grabs the
// <video> element through a component ref would have to unwrap TWO levels
// of ref (componentRef.value.video.value), which proved unreliable in
// practice (see RoomView.vue). An event is unambiguous: the received value
// is directly the element, never a ref to unwrap.
const emit = defineEmits(['video-ready'])

const video = ref(null)
watch(video, (el) => emit('video-ready', el))
let hls = null

// (Re)loads the current src/kind into the <video> element, via hls.js when needed.
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

// Exposes the <video> element: room playback sync will need it.
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
