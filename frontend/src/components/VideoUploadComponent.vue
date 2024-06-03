<template>
  <div>
    <form @submit.prevent="uploadVideoScaleAndCutIntoFrames">
      <input type="file" @change="displayVideoInPlayer" accept="video/*" required />
      <input type="text" v-model="scale" placeholder="Scale (default, 1600:-1)" />
      <input type="number" v-model.number="framesPerSecond" :min="1" :max="60" placeholder="Frames per Second (f.e. 24)"
        required />
      <button type="submit">Upload</button>
      <p>Uploaded Video ID: {{ uploadedVideoUUID }}</p>
    </form>
    <video ref="videoPlayer" :src="usersLocalPathToUploadedVideo" controls></video>

    <div class="time-range">
      Time frame: <span>{{ formatTime(0) }} - {{ formatTime(uploadedVideoDuration) }}</span>
    </div>
    <VueSlider v-model="sliderValue" :min="0" :max="uploadedVideoDuration" :tooltip="'always'"
      @change="adjustShownTimelineImages"></VueSlider>
    <p>Slider Value {{ sliderValue }}</p>
    <button v-if="allFrames.length != 0" @click="showTimeline = !showTimeline">
        {{ showTimeline ? 'Hide Timeline' : 'Show Timeline' }}
      </button>
    <div v-if="showTimeline" class="timeline-container">
      <div class="timeline-buttons">
        <button @click="unselectAllFrames">Unselect All Frames</button>
        <button @click="sendUnselectedFrames">Send Unselected Frames for Image generation</button>

      </div>

      <div class="timeline">
        <div v-for="frame in extractedFrames" :key="frame.frameNumber"
          :class="['timeline-item', { selected: isFrameSelected(frame) }]" @click="toggleFrameSelection(frame, $event)">
          <img :src="frame.src" :alt="'Frame ' + frame.frameNumber" />
          <div class="tooltip">{{ 'Frame: ' + frame.frameNumber + ', Time: ' + frame.time + 's' }}</div>
        </div>
      </div>

      <div class="timeline-buttons">
        <button @click="unselectAllFrames">Unselect All Frames</button>
        <button @click="sendUnselectedFrames">Send Unselected Frames for Image generation</button>

      </div>
    </div>
    <div v-if="longExposureImageUrl" class="long-exposure-image">
      <button @click="showTimeline = !showTimeline">
        {{ showTimeline ? 'Hide Timeline' : 'Show Timeline' }}
      </button>
      <h2>Long Exposure Image</h2>
      <img :src="longExposureImageUrl" />
    </div>
  </div>
</template>



<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue';
import axios from 'axios';
import VueSlider from 'vue-3-slider-component';

const BACKEND_URL = import.meta.env.VITE_BACKEND_URL;
console.log('Backend URL:', BACKEND_URL);

const videoPlayer: Ref<HTMLVideoElement | null> = ref(null);
const videoFile: Ref<File | null> = ref(null);
const uploadedVideoUUID: Ref<string> = ref('');
const usersLocalPathToUploadedVideo: Ref<string> = ref('');

// Video Cut Settings
const scale: Ref<string> = ref('');
const framesPerSecond: Ref<number | null> = ref(null);

// Timeline
const sliderValue: Ref<number[]> = ref([0, 0]);
const extractedFrames: Ref<{ src: string; frameNumber: number; time: string; }[]> = ref([]);
const selectedFrames: Ref<{ src: string; frameNumber: number; time: string; }[]> = ref([]);
const allFrames: Ref<{ src: string; frameNumber: number; time: string; }[]> = ref([]);
const lastSelectedFrame: Ref<{ src: string; frameNumber: number; time: string; } | null> = ref(null);
const showTimeline: Ref<boolean> = ref(true);

// Video Metadata
const uploadedVideoDuration: Ref<number> = ref(0);
const uploadedVideoFps: Ref<number> = ref(0);

// Long Exposure Image
const longExposureImageUrl: Ref<string | null> = ref(null);

interface UploadResponse {
  message: string;
  video_id: string;
}

const displayVideoInPlayer = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    videoFile.value = target.files[0];
    usersLocalPathToUploadedVideo.value = URL.createObjectURL(videoFile.value);
    resetTimelineRefsToDefault();
  }
};

const unselectAllFrames = () => {
  selectedFrames.value = [];
  lastSelectedFrame.value = null;
  const timelineItems = document.querySelectorAll('.timeline-item');
  timelineItems.forEach(item => {
    item.classList.remove('selected');
  });
};

const resetTimelineRefsToDefault = () => {
  allFrames.value = [];
  extractedFrames.value = [];
  selectedFrames.value = [];
  lastSelectedFrame.value = null;
  longExposureImageUrl.value = null;
};

const uploadVideoScaleAndCutIntoFrames = async () => {
  if (!videoFile.value || !framesPerSecond.value) {
    console.error('Missing required fields');
    return;
  }
  resetTimelineRefsToDefault();

  const formData = new FormData();
  formData.append('video_file', videoFile.value);
  formData.append('scale', scale.value);
  formData.append('fps', framesPerSecond.value.toString());

  try {
    const response = await axios.post(`${BACKEND_URL}/upload`, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
    console.log("Should have uploaded the video now.");
    const uploadResponse: UploadResponse = response.data;
    uploadedVideoUUID.value = uploadResponse.video_id;

    const allFramesArr = [];
    const duration = uploadedVideoDuration.value || 0;
    for (let i = 1; i <= Math.ceil(duration * framesPerSecond.value); i++) {
      allFramesArr.push({
        src: `http://localhost:8080/outputs/${uploadedVideoUUID.value}/frames/ffout_${i.toString().padStart(4, '0')}.png`,
        frameNumber: i,
        time: (i / framesPerSecond.value).toFixed(2),
      });
    }
    allFrames.value = allFramesArr;
    adjustShownTimelineImages(sliderValue.value, 1);
  } catch (error) {
    console.error('Error uploading video:', error);
  }
};

const adjustShownTimelineImages = (value: number[], index: number) => {
  sliderValue.value = value;

  if (index === 0) {
    if (videoPlayer.value) videoPlayer.value.currentTime = value[0];
  } else {
    if (videoPlayer.value) videoPlayer.value.currentTime = value[1];
  }
  const [start, end] = value;
  const startFrame = Math.ceil(start * (framesPerSecond.value || 0));
  const endFrame = Math.ceil(end * (framesPerSecond.value || 0)) > 0 ? Math.ceil(end * (framesPerSecond.value || 0)) : 1;

  extractedFrames.value = allFrames.value.slice(startFrame, endFrame);
};

const formatTime = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${minutes}:${secs < 10 ? '0' : ''}${secs}`;
};

const toggleFrameSelection = (frame: { src: string; frameNumber: number; time: string }, event: MouseEvent) => {
  const index = selectedFrames.value.findIndex(f => f.frameNumber === frame.frameNumber);

  if (event.shiftKey && lastSelectedFrame.value !== null) {
    const lastFrameIndex = extractedFrames.value.findIndex(f => f.frameNumber === lastSelectedFrame.value?.frameNumber);
    const currentFrameIndex = extractedFrames.value.findIndex(f => f.frameNumber === frame.frameNumber);

    const [start, end] = lastFrameIndex < currentFrameIndex
      ? [lastFrameIndex, currentFrameIndex]
      : [currentFrameIndex, lastFrameIndex];

    const framesToToggle = extractedFrames.value.slice(start, end + 1);
    const allSelected = framesToToggle.every(f => selectedFrames.value.some(sf => sf.frameNumber === f.frameNumber));

    framesToToggle.forEach(frameToToggle => {
      const frameIndex = selectedFrames.value.findIndex(f => f.frameNumber === frameToToggle.frameNumber);
      if (allSelected) {
        if (frameIndex !== -1) {
          selectedFrames.value.splice(frameIndex, 1);
        }
      } else {
        if (frameIndex === -1) {
          selectedFrames.value.push(frameToToggle);
        }
      }
    });
  } else {
    if (index === -1) {
      selectedFrames.value.push(frame);
    } else {
      selectedFrames.value.splice(index, 1);
    }
    lastSelectedFrame.value = frame;
  }
};

const isFrameSelected = (frame: { frameNumber: number }): boolean => {
  return selectedFrames.value.some(f => f.frameNumber === frame.frameNumber);
};

const sendUnselectedFrames = async () => {
  try {
    if (!uploadedVideoUUID.value) {
      console.error('No video uploaded yet');
      return;
    }
    if (longExposureImageUrl.value) {
      console.log('Long Exposure Image was previously created deleting it.');
      longExposureImageUrl.value = null;
    }

    const frameNumbersToInclude = allFrames.value
      .filter(frame => !selectedFrames.value.some(selectedFrame => selectedFrame.frameNumber === frame.frameNumber))
      .map(frame => frame.frameNumber);
    console.log('Frame Numbers to Include:', frameNumbersToInclude);

    const response = await axios.post(`${BACKEND_URL}/sendFrames`, {
      video_id: uploadedVideoUUID.value,
      frames_to_include: frameNumbersToInclude,
    });

    if (response.status === 200) {
      console.log('Response after selecting Frames:', response.data);
      longExposureImageUrl.value = response.data;
      showTimeline.value = false;
    } else {
      console.error('The Backend did not respond with 200, after sending it the selected frames.', selectedFrames.value, response.data);
    }
  } catch (error) {
    console.error('Error sending selected frames:', error);
  }
};

onMounted(() => {
  if (videoPlayer.value) {
    videoPlayer.value.addEventListener('loadedmetadata', () => {
      uploadedVideoDuration.value = Math.floor(videoPlayer.value?.duration || 0);
      sliderValue.value = [0, uploadedVideoDuration.value || 0];
    });
  }
});
</script>

<style scoped>
body {
  font-family: Arial, sans-serif;
  margin: 0;
  padding: 0;
}


/* Input styles */
input[type='file'],
input[type='text'],
input[type='number'] {
  margin-bottom: 10px;
  padding: 8px;
  width: 100%;
  box-sizing: border-box;
  border: 1px solid #ccc;
  border-radius: 5px;
  font-size: 16px;
}

input[type='file'] {
  cursor: pointer;
}

input[type='file']:hover {
  background-color: green;
}

input[type='file']:focus,
input[type='text']:focus,
input[type='number']:focus {
  outline: none;
  border-color: dodgerblue;
  border: 4px dashed dodgerblue;
}

input:invalid {
  border: 2px dashed red;
}

input::placeholder {
  font-weight: bold;
}

/* Video player styles */
video {
  width: 1028px;
  height: auto;
  margin-bottom: 20px;
}



/* Button styles */
button {
  padding: 10px 20px;
  background-color: dodgerblue;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
}

button:hover {
  background-color: #007acc;
}

button:active {
  background-color: #005f80;
}

.timeline-container {
  margin-top: 20px;
  
}

.timeline-buttons {
  display: flex;
  justify-content: center;
  margin-bottom: 10px;
}

.timeline {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 10px;
}

.timeline-item {
  position: relative;
  cursor: pointer;
}

.timeline-item img {
  width: 100%;
  height: auto;
  object-fit: cover;
  border: 2px solid transparent;
  transition: border-color 0.2s;
}

.timeline-item:hover img {
  border-color: dodgerblue;
}

.timeline-item.selected img {
  border-color: red;
}

.timeline-item .tooltip {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  padding: 5px;
  background-color: black;
  color: white;
  border-radius: 3px;
  opacity: 0;
  transition: opacity 0.2s;
  white-space: nowrap;
}

.timeline-item:hover .tooltip {
  opacity: 1;
}

.long-exposure-image img {
  width: 100%;
  height: auto;
  margin-top: 20px;
}
</style>
