import { createRouter, createWebHistory } from 'vue-router'
import VideoUploadComponent from '@/components/VideoUploadComponent.vue'
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: VideoUploadComponent
    }
  ]
})

export default router
