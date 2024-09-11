import { createRouter, createWebHistory } from 'vue-router'
import ProjectEditor from '@/components/ProjectEditor.vue'
import ProjectPicker from "@/components/ProjectPicker.vue";
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/projects'  // Redirect root path to /projects
    },
    {
      path: '/projects/new',
      name: 'NewProject',
      component: ProjectEditor
    },
    {
      path: '/projects',
      name: 'Projects',
      component: ProjectPicker
    },
    {
      path: '/projects/:id',
      name: 'ProjectEditor',
      component: ProjectEditor,
      props: true
    }
  ]
})

export default router
