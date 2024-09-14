<template>
    <div class="timeline-container">

      <div v-if="videoDuration>0 && props.showTimeline" class="time-range">
      Time frame: <span>{{ formatTime(0) }} - {{ formatTime(videoDuration) }}</span>
      <VueSlider v-model="sliderValue" :min="0" :max="videoDuration" :tooltip="'always'"
      @change="adjustShownTimelineImages"></VueSlider>
    </div>

      <div class="timeline-buttons" v-if="props.showTimeline && allFrames.length > 0 ">
        <button @click="unselectAllFrames">Unselect All Frames</button>
        <button @click="sendUnselectedFrames">Send Unselected Frames for Image Generation</button>
      </div>
  
      <div class="timeline" v-if="props.showTimeline">
        <div v-for="frame in displayedFrames" :key="frame.frameNumber"
          :class="['timeline-item', { selected: isFrameSelected(frame) }]"
          @click="toggleFrameSelection(frame, $event)">
          <img :src="frame.src" loading="lazy" :alt="'Frame ' + frame.frameNumber" />
          <div class="tooltip">{{ 'Frame: ' + frame.frameNumber + ', Time: ' + frame.time + 's' }}</div>
          <div class="frame-details">
            <label for="weight-{{frame.frameNumber}}">Weight:</label>
            <input type="number" v-model.number="frame.weight" min="0" step="0.1" id="weight-{{frame.frameNumber}}"
              @input="updateFrameWeight(frame)" @click.stop @mousedown.stop />
          </div>
        </div>
      </div>
  
      <div class="timeline-buttons" v-if="props.showTimeline && allFrames.length > 0">
        <button @click="unselectAllFrames">Unselect All Frames</button>
        <button @click="sendUnselectedFrames">Send Unselected Frames for Image Generation</button>
      </div>
    </div>
  </template>
<script setup lang="ts">
import type { Frame, FrameToInclude } from '@/types';
import VueSlider from 'vue-3-slider-component';
import {endpoints, getBackendUrlByEndpoint} from '@/api';
import { ref, watch, defineProps, defineEmits } from 'vue';

const props= defineProps<{
  projectId: string | null,
  fps: number,
  videoDuration: number,
  showTimeline: boolean,
  loadThumbnailsTrigger: number,
}>()


watch(
  [() => props.videoDuration],
  ([newVideoDuration]) => {
    // Update sliderValue if video duration changes
    if (newVideoDuration > 0) {
      sliderValue.value = [sliderValue.value[0], newVideoDuration];
    }
    loadTimelineThumbnails();
  }
);

watch(
  () => props.loadThumbnailsTrigger,
  (newVal, oldVal) => {
    if (props.projectId && props.fps > 0 && props.videoDuration > 0) {
      loadTimelineThumbnails();
    }
  }
);

const allFrames = ref<Frame[]>([])

const displayedFrames = ref<Frame[]>([])

const selectedFrames = ref<Frame[]>([]);
// For Shift Functionality
const lastSelectedFrame = ref<Frame | null>(null);


const sliderValue = ref<number[]>([0,props.videoDuration]);
const emit = defineEmits<{
  (e: 'videoPositionUpdate', newVideoPosition: number): void,
  (e: 'sendFrames', frames: FrameToInclude[]): void,
}>();

const unselectAllFrames = () => {
  selectedFrames.value = [];
};

const toggleFrameSelection = (frame: Frame, event: MouseEvent) => {
  if (event.shiftKey && lastSelectedFrame.value !== null) {
    selectMultipleFrames(frame);
  } else {
    toggleSingleFrameSelection(frame);
    lastSelectedFrame.value = frame;
  }
};

// Toggle selection for a single frame
const toggleSingleFrameSelection = (frame: Frame) => {
  const isSelected = isFrameSelected(frame);
  if (isSelected) {
    removeFrameFromSelection(frame);
  } else {
    selectedFrames.value.push(frame);
  }
};

// Select or deselect a range of frames when Shift is pressed
const selectMultipleFrames = (currentFrame: Frame) => {
  const lastFrameIndex = findFrameIndex(lastSelectedFrame.value?.frameNumber);
  const currentFrameIndex = findFrameIndex(currentFrame.frameNumber);
  const [start, end] = getRange(lastFrameIndex, currentFrameIndex);

  const framesToToggle = displayedFrames.value.slice(start, end + 1);
  const allSelected = areAllFramesSelected(framesToToggle);

  framesToToggle.forEach(frame => {
    if (allSelected) {
      removeFrameFromSelection(frame);
    } else {
      addFrameToSelection(frame);
    }
  });
};

// Check if all frames in a range are selected
const areAllFramesSelected = (frames: Frame[]): boolean => {
  return frames.every(frame => isFrameSelected(frame));
};

// Get the index range between the last selected frame and the current frame
const getRange = (index1: number, index2: number): [number, number] => {
  return index1 < index2 ? [index1, index2] : [index2, index1];
};

// Find the index of a frame by its frame number
const findFrameIndex = (frameNumber: number | undefined): number => {
  return displayedFrames.value.findIndex(f => f.frameNumber === frameNumber);
};

// Check if a frame is already selected
const isFrameSelected = (frame: Frame): boolean => {
  return selectedFrames.value.some(f => f.frameNumber === frame.frameNumber);
};

// Add a frame to the selection
const addFrameToSelection = (frame: Frame) => {
  if (!isFrameSelected(frame)) {
    selectedFrames.value.push(frame);
  }
};

// Remove a frame from the selection
const removeFrameFromSelection = (frame: Frame) => {
  const index = selectedFrames.value.findIndex(f => f.frameNumber === frame.frameNumber);
  if (index !== -1) {
    selectedFrames.value.splice(index, 1);
  }
};


const sendUnselectedFrames = () => {
  const unselectedFrames = allFrames.value.filter(frame => !selectedFrames.value.includes(frame));

  const framesToInclude = unselectedFrames.map(frame => ({
    frame_number: frame.frameNumber,
    frame_weight: frame.weight,
  }));


  emit('sendFrames', framesToInclude); // Emit the unselected frames to the parent, to create long exposure image
};

const updateFrameWeight = (frame: Frame) => {
  const frameIndex = displayedFrames.value.findIndex(f => f.frameNumber === frame.frameNumber);
  if (frameIndex !== -1) {
    displayedFrames.value[frameIndex].weight = frame.weight;
  }
};

const adjustShownTimelineImages = (value: number[], index: number) => {
  sliderValue.value = value;

  if (index === 0) {
    emit('videoPositionUpdate', value[0]);
  } else {
    emit('videoPositionUpdate', value[1]);
  }

  const [start, end] = value;
  const startFrame = Math.ceil(start * (props.fps || 0));
  const endFrame = Math.ceil(end * (props.fps || 0)) > 0 ? Math.ceil(end * (props.fps || 0)) : 1;

  displayedFrames.value = allFrames.value.slice(startFrame, endFrame);
};


const formatTime = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${minutes}:${secs < 10 ? '0' : ''}${secs}`;
};


const loadTimelineThumbnails = () => {
  if (!props.projectId || !props.fps) {
    return
  }

  const allFramesArr: Frame[] = [];

  for (let i = 1; i <= Math.ceil(props.videoDuration * props.fps); i++) {
    allFramesArr.push({
      src: getBackendUrlByEndpoint(endpoints.frameThumbnail(props.projectId, i)),
      frameNumber: i,
      time: (i / props.fps).toFixed(2),
      weight: 1.0,
    });
  }
  allFrames.value = allFramesArr;
  displayedFrames.value = allFramesArr
}
</script>



<style scoped>

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
  padding: 0;
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
  z-index: 3;
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