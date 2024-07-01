<template>
  <div>
    <form @submit.prevent="uploadVideoScaleAndCutIntoFrames">
      <input type="file" @change="displayVideoInPlayer" accept="video/*" required />
      <input type="text" v-model="scale" placeholder="Scale (default, 1600:-1)" />
      <input type="number" v-model.number="framesPerSecond" :min="1" :max="60" placeholder="Frames per Second (f.e. 24)"
        required />
      <button type="submit">Upload</button>
      <div v-if="progressVisible" class="progress">
        <div class="progress-bar" :style="{ width: uploadProgress + '%' }">{{ uploadProgress }}%</div>
        <!-- <progress :value="uploadProgress" max="100"></progress>
        <span>{{ uploadProgress }}%</span> -->
      </div>
      <p>Uploaded Video ID: {{ uploadedVideoUUID }}</p>
    </form>
    <video ref="videoPlayer" :src="usersLocalPathToUploadedVideo" controls></video>

    <div class="time-range">
      Time frame: <span>{{ formatTime(0) }} - {{ formatTime(uploadedVideoDuration) }}</span>
    </div>
    <VueSlider v-model="sliderValue" :min="0" :max="uploadedVideoDuration" :tooltip="'always'"
      @change="adjustShownTimelineImages"></VueSlider>
    <p>Slider Value {{ sliderValue }}</p>
    <button v-if="videoWasCutIntoFrames" @click="showTimeline = !showTimeline">
      {{ showTimeline ? 'Hide Timeline' : 'Show Timeline' }}
    </button>
    <div v-if="videoWasCutIntoFrames && showTimeline" class="timeline-container">
      <div class="timeline-buttons">
        <button @click="unselectAllFrames">Unselect All Frames</button>
        <button @click="sendUnselectedFrames">Send Unselected Frames for Image generation</button>

      </div>

      <div class="timeline">
        <div v-for="frame in extractedFrames" :key="frame.frameNumber"
          :class="['timeline-item', { selected: isFrameSelected(frame) }]" @click="toggleFrameSelection(frame, $event)">
          <img :src="frame.src" loading="lazy" :alt="'Frame ' + frame.frameNumber" />
          <div class="tooltip">{{ 'Frame: ' + frame.frameNumber + ', Time: ' + frame.time + 's' }}</div>
          <div class="frame-details">
            <label for="weight-{{frame.frameNumber}}">Weight:</label>
            <input type="number" v-model.number="frame.weight" min="0" step="0.1" id="weight-{{frame.frameNumber}}"
              @input="updateFrameWeight(frame)" @click.stop @mousedown.stop />
          </div>
        </div>
      </div>

      <div class="timeline-buttons">
        <button @click="unselectAllFrames">Unselect All Frames</button>
        <button @click="sendUnselectedFrames">Send Unselected Frames for Image generation</button>

      </div>
    </div>
    <div v-if="longExposureImageUrl" class="long-exposure-image">
      <h2>Long Exposure Image</h2>
      <button v-if="showTimeline" @click="showTimeline = !showTimeline">
        {{ showTimeline ? 'Hide Timeline' : 'Show Timeline' }}
      </button>
      <img :src="longExposureImageUrl" />
    </div>
  </div>
</template>



<script setup lang="ts">
import { ref, onMounted, computed, type Ref } from 'vue';
import axios from 'axios';
import type { AxiosResponse } from 'axios';
import VueSlider from 'vue-3-slider-component';


interface Frame {
  src: string;
  frameNumber: number;
  time: string;
  weight: number;
}

interface FrameToInclude {
  frame_number: number;
  frame_weight: number;
}

interface CreateLongExposureImageRequest {
  video_id: string;
  frames_to_include: FrameToInclude[];
}


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
const extractedFrames: Ref<Frame[]> = ref([]);
const selectedFrames: Ref<Frame[]> = ref([]);
const allFrames: Ref<Frame[]> = ref([]);
const videoWasCutIntoFrames: Ref<boolean> = computed(() => {
  return allFrames.value.length > 0;
});
const lastSelectedFrame: Ref<Frame | null> = ref(null);
const showTimeline: Ref<boolean> = ref(true);

// Video Metadata
const uploadedVideoDuration: Ref<number> = ref(0);
const uploadProgress: Ref<number> = ref(0);
const progressVisible: Ref<boolean> = ref(false);


// Long Exposure Image
const longExposureImageUrl: Ref<string | null> = ref(null);

interface UploadResponse {
  message: string;
  video_id: string;
}


const displayVideoInPlayer = (event: Event) => {
  uploadedVideoUUID.value = 'Will be set after uploading the video';
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
  progressVisible.value = true;

  const formData = new FormData();
  formData.append('video_file', videoFile.value);
  formData.append('scale', scale.value);
  formData.append('fps', framesPerSecond.value.toString());

  try {
    const response = await axios.post(`${BACKEND_URL}/upload`, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
      onUploadProgress: (progressEvent) => {
        if (progressEvent.lengthComputable) {
          uploadProgress.value = Math.round((progressEvent.loaded / (progressEvent.total ?? 100)) * 100);
        }
      }
    });
    console.log("Should have uploaded the video now.");
    const uploadResponse: UploadResponse = response.data;
    uploadedVideoUUID.value = uploadResponse.video_id;

    const allFramesArr: Frame[] = [];
    const duration = uploadedVideoDuration.value || 0;
    for (let i = 1; i <= Math.ceil(duration * framesPerSecond.value); i++) {
      allFramesArr.push({
        src: `${BACKEND_URL}/outputs/${uploadedVideoUUID.value}/frames/ffout_thumbnail_${i.toString().padStart(4, '0')}.webp`,
        frameNumber: i,
        time: (i / framesPerSecond.value).toFixed(2),
        weight: 1.0,
      });
    }
    allFrames.value = allFramesArr;
    adjustShownTimelineImages(sliderValue.value, 1);
  } catch (error) {
    console.error('Error uploading video:', error);
  }
  finally {
    progressVisible.value = false;
    uploadProgress.value = 0;
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

const toggleFrameSelection = (frame: Frame, event: MouseEvent) => {
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


    const includedFrames: FrameToInclude[] = allFrames.value
      .filter(frame => !selectedFrames.value.some(selectedFrame => selectedFrame.frameNumber === frame.frameNumber))
      .map(frame => ({ frame_number: frame.frameNumber, frame_weight: frame.weight }));
    console.log('Frame Numbers to Include:', includedFrames);

    const payload: CreateLongExposureImageRequest = {
      video_id: uploadedVideoUUID.value,
      frames_to_include: includedFrames,
    };
    const response: AxiosResponse<string> = await axios.post<CreateLongExposureImageRequest,AxiosResponse<string>>(`${BACKEND_URL}/sendFrames`, payload);

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

const updateFrameWeight = (frame: Frame) => {
  // Find the frame in the selected frames and update its weight
  const frameIndex = allFrames.value.findIndex(f => f.frameNumber === frame.frameNumber);
  if (frameIndex !== -1) {
    allFrames.value[frameIndex].weight = frame.weight;
    // If the frame weight is 0, add it to the selected frames
    if (frame.weight === 0) {
      // Check if the frame is already in the selected frames
      const selectedFrameIndex = selectedFrames.value.findIndex(f => f.frameNumber === frame.frameNumber);
      if (selectedFrameIndex === -1) {
        selectedFrames.value.push(frame);
      }
    }
  }
  // Log the updated frame weight (optional)
  console.log(`Frame ${frame.frameNumber} weight updated to ${frame.weight}`);
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
.progress {
  width: 100%;
  background-color: #f3f3f3;
  border-radius: 5px;
  overflow: hidden;
}

.progress-bar {
  height: 24px;
  background-color: #4caf50;
  text-align: center;
  color: white;
  transition: width 0.3s;
}


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
  transition: background-color 0.3s;
}

button:hover {
  background-color: #0056b3;
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
  gap: 10px;
  margin-top: 10px;
  margin-bottom: 10px;
}

.timeline {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 10px;
}

/* Individual timeline items */
.timeline-item {
  position: relative;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0px;
  background-color: #f9f9f9;
  transition: box-shadow 0.2s, transform 0.2s;
}

/* Images inside timeline items */
.timeline-item img {
  width: 100%;
  height: auto;
  object-fit: cover;
  border: 2px solid transparent;
  /* Set initial border to reserve space */
  transition: border-color 0.2s, transform 0.2s;
  box-sizing: border-box;
  /* Ensure border is included in the element's width and height */
}

/* Hover effect on images */
.timeline-item:hover img {
  border-color: dodgerblue;
}

/* Selected state for images */
.timeline-item.selected img {
  border-color: red;
}

/* Weight input fields */
.timeline-item input[type="number"] {
  margin-top: 5px;
  width: 60px;
  border: 1px solid #ccc;
  border-radius: 4px;
  padding: 2px;
  text-align: center;
  background-color: #fff;
  font-weight: bold;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: border-color 0.2s;
  z-index: 2;
  /* Ensure it is above other elements */
  position: relative;
}

.timeline-item input[type="number"]:focus {
  border-color: dodgerblue;
}

/* Labels for weight input fields */
.timeline-item label {
  font-size: 14px;
  font-weight: bold;
  margin-top: 5px;
  color: #000000;
  display: block;
  text-align: center;
}

/* Tooltips for timeline items */
.timeline-item .tooltip {
  position: absolute;
  bottom: 100%;
  padding: 5px;
  background-color: black;
  color: white;
  border-radius: 3px;
  opacity: 0;
  transition: opacity 0.3s;
  white-space: nowrap;
  pointer-events: none;
}

/* Tooltip visibility on hover */
.timeline-item:hover .tooltip {
  opacity: 1;
}

.long-exposure-image img {
  width: 100%;
  height: auto;
  margin-top: 20px;
}
</style>
