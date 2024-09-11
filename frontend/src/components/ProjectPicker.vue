<template>
  <div class="container">
    <h1 class="title">Project Picker</h1>

    <div class="new-projects">
      <router-link :to="{ name: 'NewProject' }">
        <button class="primary-button">Create New Project</button>
      </router-link>
    </div>

    <!-- Display projects if available -->
    <div class="projects-grid">
      <div v-for="project in projects" :key="project.id" class="project-card">
        <div class="delete-button" @click.stop="deleteProject(project.id)">
          <delete-icon/>
        </div>
        <router-link :to="{ name: 'ProjectEditor', params: { id: project.id } }" class="project-link">
          <img :src="project.thumbnail_path" loading="lazy" :alt="'Project ' + project.id" class="project-thumbnail"/>
          <h3 class="project-title">{{ project.id }}</h3>
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import axios from "axios";
import DeleteIcon from './DeleteIcon.vue';


const BACKEND_URL = import.meta.env.VITE_BACKEND_URL;
console.log('Backend URL:', BACKEND_URL);

let projects = ref<Array<ProjectResponse>>([]);

const getProjects = async () => {
  try {
    const res = await axios.get(BACKEND_URL + "/projects");
    projects.value = res.data.projects;
    console.log('Projects:', projects.value);
  } catch (error) {
    console.error('Error fetching projects:', error);
  }
}

const deleteProject = async (projectId: string) => {
  try {
    projects.value = projects.value.filter(project => project.id !== projectId);
    await axios.delete(`${BACKEND_URL}/projects/${projectId}`);
    console.log(`Project ${projectId} deleted successfully`);
  } catch (error) {
    console.error(`Error deleting project ${projectId}:`, error);
    // If the delete request fails, add the project back to the list
    await getProjects();
  }
}

onMounted(() => {
  getProjects();
});

interface ProjectResponse {
  id: string,
  thumbnail_path: string,
}
</script>
<style scoped>
body {
  background-color: #1e1e1e;
  color: #ddd;
  font-family: Arial, sans-serif;
  margin: 0;
  padding: 0;
  min-height: 100vh;
}

/* Container styles */
.container {
  max-width: 1200px;
  width: 90%;
  margin: 0 auto;
  padding: 20px;
  box-sizing: border-box;
}

/* Centered text for titles and messages */
.title, .new-projects {
  text-align: center;
}

.title {
  font-size: 2.5rem;
  margin-bottom: 30px;
  color: #fff;
}

.new-projects {
  margin-top: 50px;
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
}

/* Project card styles */
.project-card {
  position: relative;
  display: inline-block;
  background-color: #2c2c2c;
  border-radius: 10px;
  overflow: hidden;
  max-width: 720px;
  margin: 15px;
  transition: box-shadow 0.3s ease;
  vertical-align: top;
  width: 100%;
}

/* Only apply box shadow if not hovering over the trash bin */
.project-card:hover:not(:hover .delete-button) {
  box-shadow: 0 0 0 3px #007bff;
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
  z-index: 1;
}

.delete-button:hover {
  background-color: #d50000;
}

.delete-button svg {
  width: 18px;
  height: 18px;
}

/* Project link styles */
.project-link {
  text-decoration: none;
  color: inherit;
  display: block;
}

/* Project thumbnail styles */
.project-thumbnail {
  width: 100%;
  height: 300px;
  object-fit: cover;
}

/* Project title styles */
.project-title {
  padding: 15px;
  font-size: 1.25rem;
  margin: 0;
  background-color: rgba(0, 0, 0, 0.1);
}
</style>
