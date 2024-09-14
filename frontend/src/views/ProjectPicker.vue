<template>
  <div>
    <div>
      <h1 class="title">Project Picker</h1>

      <div class="new-projects">
        <router-link :to="{ name: 'NewProject' }">
          <button class="primary-button">Create New Project</button>
        </router-link>
      </div>

      <!-- Display projects if available -->
      <div class="projects-grid">
        <div v-for="project in projectsArr" :key="project.id" class="project-card">
          <div class="delete-button" @click.stop="deleteProject(project.id)">
            <delete-icon />
          </div>
          <router-link :to="{ name: 'ProjectEditor', params: { projectId: project.id } }" class="project-link">
            <img :src="project.thumbnail_path" loading="lazy" :alt="'Project ' + project.id"
              class="project-thumbnail" />
            <h3 class="project-title">{{ project.project_name }}</h3>
          </router-link>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import DeleteIcon from '@/components/DeleteIcon.vue';
import { api, endpoints } from '@/api'
import type { ProjectDetails, ProjectsResponse } from '@/types'


let projectsArr = ref<Array<ProjectDetails>>([]);

const getProjects = async () => {
  try {
    const { data, status }: { data: ProjectsResponse; status: number } = await api.get<ProjectsResponse>(endpoints.projects);

    if (status !== 200) {
      console.error(`Error fetching projects: Received status ${status}`);
      return;
    }

    const projects: Array<ProjectDetails> = data.projects;
    projectsArr.value = projects;

  } catch (error) {
    console.error('Error fetching projects:', error);
  }
};

const deleteProject = async (projectId: string) => {
  try {
    projectsArr.value = projectsArr.value.filter(project => project.id !== projectId);
    const { data, status } = await api.delete<string>(endpoints.specificProject(projectId));
    if (status != 200) {
      console.error(`An error occured on Server Side when trying to delete project ${projectId}. Status : ${status}`);
    }
    console.log(data);
  } catch (error) {
    console.error(`Error deleting project ${projectId}:`, error);
    // If the delete request fails, add the project back to the list
    await getProjects();
  }
}

onMounted(() => {
  getProjects();
});


</script>
<style scoped>
body {
  font-family: Arial, sans-serif;
  margin: 0;
  padding: 0;
}


/* Centered text for titles and messages */
.title {
  text-align: center;
  width: 100%;
  margin-left: 10rem;
  font-size: 2.5rem;
  color: #fff;
}

.new-projects {
  text-align: center;
  width: 100%;
  margin-left: 10rem;
  margin-top: 5%;
}

/* Button styles */
.primary-button {
  padding: 10px 20px;
  background-color: #007bff;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  transition: background-color 0.3s ease;
  font-size: 1rem;
  display: inline-block;
}

.primary-button:hover {
  background-color: #0056b3;
}

/* Projects grid layout */
.projects-grid {
  text-align: center;
  margin-top: 30px;
  margin-left: 10rem;
  width: 100%;
}

/* Project card styles */
.project-card {
  position: relative;
  display: inline-block;
  background-color: #2c2c2c;
  border-radius: 10px;
  overflow: hidden;
  width: 720px;
  max-height: auto;
  transition: box-shadow 0.3s ease, border-color 0.3s ease;
  border: 3px solid transparent; /* Default transparent border */
  vertical-align: middle;
  margin-bottom: 50px;
}

.project-title {
  padding: 15px;
  font-size: 1.25rem;
  color: #fff;
  text-align: center;
  background-color: rgba(0, 0, 0, 0.7); /* Ensure it has a background to contrast with the image */
  height: 60px; /* Explicit height for the title */
  box-sizing: border-box;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis; /* Ensure long titles are truncated */
}

.project-thumbnail {
  width: 100%; /* Ensure the image fills the card */
  height: calc(100% - 60px); /* Subtract space for the title */
  object-fit: cover; /* Ensures the image maintains its aspect ratio */
  overflow: hidden; /* Prevents any overflow */
}

/* Project link styles */
.project-link {
  text-decoration: none;
  color: #fff; /* Override the global greenish color */
  display: block;
}

.project-link:visited {
  color: #fff;
}

.project-card:hover .delete-button:hover ~ .project-card {
  border-color: #ff4136; /* Red border when hovering delete button */
}



.delete-button {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 30px;
  height: 30px;
  background-color: #ff4136;
  border-radius: 50%;
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: background-color 0.3s ease;
}


.delete-button:hover {
  background-color: #d50000;
}

.project-card:hover {
  box-shadow: 0 0 0 3px #007bff;
}

</style>
