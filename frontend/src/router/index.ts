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
      component: import('@/components/ProjectEditor.vue')
    },
    {
      path: '/projects',
      name: 'Projects',
      component: import('@/components/ProjectPicker.vue')
    },
    {
      path: '/projects/:id',
      name: 'ProjectEditor',
      component: import('@/components/ProjectEditor.vue'),
      props: true
    }
  ]
})

export default router
