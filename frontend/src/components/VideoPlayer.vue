<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';

const props = defineProps<{
  src: string,
  videoCurrentPlaybackTime: number
}>();

const emit = defineEmits<{
  (e: 'durationChange', duration: number): void
}>();

const videoPlayer = ref<HTMLVideoElement | null>(null);

watch(() => props.src, (newSrc) => {
  if (videoPlayer.value) {
    videoPlayer.value.src = newSrc;
  }
});


// Adjust the video time to match the slider's start value
watch(() => props.videoCurrentPlaybackTime, (newValue) => {
  if (videoPlayer.value) {
    videoPlayer.value.currentTime = newValue; 
  }
});

onMounted(() => {
  if (videoPlayer.value) {
    videoPlayer.value.addEventListener('loadedmetadata', () => {
      emit('durationChange', Math.floor(videoPlayer.value?.duration || 0));
    });
  }
});
</script>

<template>
  <video ref="videoPlayer" controls>
    <source :src="src" type="video/mp4">
  </video>
</template>

<style scoped>
/* Video player styles */
video {
  width: 1080px;
  max-height: 70vh; /* Limit video height to 70% of the viewport height */
  margin-bottom: 50px;
}
</style>