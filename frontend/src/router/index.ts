import { createRouter, createWebHistory } from 'vue-router'
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
      component: () => import('@/views/ProjectEditor.vue')
    },
    {
      path: '/projects',
      name: 'Projects',
      component: () => import('@/views/ProjectPicker.vue')
    },
    {
      path: '/projects/:projectId',
      name: 'ProjectEditor',
      component: () => import('@/views/ProjectEditor.vue'),
      props: true
    }
  ]
})

export default router
