<template>
  <div>

    <div>
    <router-link :to="{ name: 'Projects' }" class="back-button">
      ← Back to Project Picker
    </router-link>
    </div>

    <form @submit.prevent="uploadVideoScaleAndCutIntoFrames">
      <input type="file" @change="displayVideoInPlayer" accept="video/*" :required="!projectId" />
      <input type="text" v-model.projectName="projectName" placeholder="Name for your project" required />
      <input type="number" v-model.number="framesPerSecond" :min="1" :max="30" placeholder="Frames per Second (f.e. 24)"
      required />
      <input type="text" v-model.scale="scale" placeholder="Scale (default, 1600:-1)" />

      <button type="submit">{{ projectId ? 'Create New Frames' : 'Create New Project' }}</button>
      <div v-if="progressVisible" class="progress">
        <div class="progress-bar" :style="{ width: uploadProgress + '%' }">{{ uploadProgress }}%</div>
      </div>
      <p>Project ID:
        <span v-if="projectId">{{ projectId }}</span>
        <span v-else>Will be set after uploading the video</span>
      </p>
    </form>

    <VideoPlayer :src="videoSource" :videoCurrentPlaybackTime="videoCurrentPlaybackTime" @duration-change="videoDuration = $event" />
    
    <button v-if="projectId" @click="showTimeline = !showTimeline">
      {{ showTimeline ? 'Hide Timeline' : 'Show Timeline' }}
    </button>
    
    <TimelineComponent
    :videoDuration="videoDuration"
    :loadThumbnailsTrigger="loadThumbnailsTrigger"
    :fps="framesPerSecond"
    :project-id="projectId"
    :showTimeline="showTimeline"
    @video-position-update="updateVideoPosition"
    @send-frames="sendUnselectedFrames"
    />
    
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
import { ref, onMounted, computed, watch, type Ref } from 'vue';
import type { ApiResponse, UploadResponse, ProjectDataResponse, CreateLongExposureImageRequest, FrameToInclude, Frame } from '@/types';
import { api, uploadFile, endpoints, getBackendUrlByEndpoint } from "@/api"
import VideoPlayer from '@/components/VideoPlayer.vue';
import TimelineComponent from '@/components/TimelineComponent.vue';


const props = defineProps<{
  projectId?: string
}>()

const videoCurrentPlaybackTime = ref(0);



const projectId: Ref<string | null> = ref(props.projectId ?? null);
const videoFile: Ref<File | string | Blob | null> = ref(null);
const usersLocalPathToUploadedVideo: Ref<string> = ref('');
const videoUrlOnServer: Ref<string> = ref('');

const videoSource = computed(() => {
  return usersLocalPathToUploadedVideo.value != "" ? usersLocalPathToUploadedVideo.value : videoUrlOnServer.value;
})

// Video Cut Settings
const projectName : Ref<string> = ref('');
const scale: Ref<string> = ref('');
const framesPerSecond: Ref<number> = ref(0);

// Timeline
const showTimeline: Ref<boolean> = ref(true);
const loadThumbnailsTrigger = ref(1);

// Video Metadata
const videoDuration: Ref<number> = ref(0);
const uploadProgress: Ref<number> = ref(0);
const progressVisible: Ref<boolean> = ref(false);

// Long Exposure Image
const longExposureImageUrl: Ref<string | null> = ref(null);


const displayVideoInPlayer = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    videoFile.value = target.files[0];
    usersLocalPathToUploadedVideo.value = URL.createObjectURL(videoFile.value);
    resetTimelineRefsToDefault();
  }
};

const resetTimelineRefsToDefault = () => {
  longExposureImageUrl.value = null;
  projectId.value = null;
};

const uploadVideoScaleAndCutIntoFrames = async () => {
  if ((!videoFile.value && !projectId.value) || !framesPerSecond.value) {
    console.error('Missing required fields');
    return;
  }
  progressVisible.value = true;


  const formData = new FormData();
  if (projectId.value) {
    console.log("Video id is :" + projectId.value)
    formData.append('video_id', projectId.value)
  }
  if (videoFile.value) {
    formData.append('video_file', videoFile.value);
  }
  if (!scale.value) {
    scale.value = '1600:-1'
  }

  formData.append('scale', scale.value);
  formData.append('fps', framesPerSecond.value.toString());
  formData.append('project_name', projectName.value);

  try {
    const response: ApiResponse<UploadResponse> = await uploadFile(endpoints.projects, formData, (progress) => {
      uploadProgress.value = progress;
    });
    const uploadResponse: UploadResponse = response.data;
    const status: number = response.status;
    console.log(uploadResponse.message + ` status: ${status}`);

    projectId.value = uploadResponse.video_id;

    //Adjust Url
    const newUrl = `/projects/${projectId.value}`;
    history.replaceState(history.state, '', newUrl);
    loadThumbnailsTrigger.value +=1;

  } catch (error) {
    console.error('Error uploading video:', error);
  } finally {
    progressVisible.value = false;
    uploadProgress.value = 0;
  }
};

const updateVideoPosition = (value: number) => {
  videoCurrentPlaybackTime.value = value
};


const sendUnselectedFrames = async (unselectedFrames : FrameToInclude[]) => {
  try {
    if (!projectId.value) {
      console.error('No video uploaded yet');
      return;
    }
    if (longExposureImageUrl.value) {
      console.log('Long Exposure Image was previously created deleting it.');
      longExposureImageUrl.value = null;
    }
    const payload: CreateLongExposureImageRequest = {
      frames_to_include: unselectedFrames,
    };
    const { data, status }: { data: string; status: number } = await api.post<string>(endpoints.createLongExposureImage(projectId.value), payload);

    if (status === 200) {
      console.log('Response after selecting Frames:', data);
      longExposureImageUrl.value = data;
      showTimeline.value = false;
    } else {
      console.error('The Backend did not respond with 200, after sending it the unselected frames.', unselectedFrames, data);
    }
  } catch (error) {
    console.error('Error sending selected frames:', error);
  }
};


const loadProjectData = async () => {
  if (!projectId.value) return;
  try {
    const { data: projectData, status }: ApiResponse<ProjectDataResponse> = await api.get<ProjectDataResponse>(endpoints.specificProject(projectId.value));
    if (status != 200) {
      console.error('Error fetching projectData:', { status, data: projectData });
      return;
    }
    console.log("Data ", projectData);

    scale.value = projectData.scale;
    projectName.value = projectData.project_name
    framesPerSecond.value = projectData.fps;

    // If there's a long exposure image, show it
    if (projectData.latest_long_exposure_image_name) {
      longExposureImageUrl.value = projectData.latest_long_exposure_image_name;
      showTimeline.value = false;
    }

    // Load Video from Backend
    videoUrlOnServer.value = getBackendUrlByEndpoint(endpoints.videoFile(projectId.value, projectData.video_file_extension));
    loadThumbnailsTrigger.value +=1;
  } catch (error) {
    console.error('Error loading project data:', error);
  }
};

onMounted(() => {
  loadProjectData()
});
</script>

<style scoped>

.back-button {
  position: absolute;
  top: 10px;
  left: 10px;
  padding: 10px 20px;
  background-color: dodgerblue;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  text-decoration: none;
  font-weight: bold;
  transition: background-color 0.3s;
}

.back-button:hover {
  background-color: #0056b3;
}

.back-button:active {
  background-color: #005f80;
}


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
  border: 4px dashed dodgerblue;
}

input:invalid {
  border: 2px dashed red;
}

input::placeholder {
  font-weight: bold;
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



.long-exposure-image img {
  width: 100%;
  height: auto;
  margin-top: 20px;
}

</style>
