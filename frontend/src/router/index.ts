import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'Login',
      component: () => import('../views/Login.vue'),
      meta: { guest: true },
    },
    {
      path: '/',
      component: () => import('../layouts/MainLayout.vue'),
      children: [
        {
          path: '',
          name: 'Dashboard',
          component: () => import('../views/Dashboard.vue'),
        },
        {
          path: 'search',
          name: 'Search',
          component: () => import('../views/Search.vue'),
        },
        {
          path: 'detail/imdb/:imdb_id',
          name: 'ImdbDetail',
          component: () => import('../views/Detail.vue'),
          props: true,
        },
        {
          path: 'detail/custom/:other_id',
          name: 'CustomDetail',
          component: () => import('../views/Detail.vue'),
          props: true,
        },
        {
          path: 'detail/:bangumi_id',
          name: 'Detail',
          component: () => import('../views/Detail.vue'),
          props: true,
        },
        {
          path: 'profile',
          name: 'Profile',
          component: () => import('../views/UserPanel.vue'),
        },
        {
          path: 'logs',
          name: 'Logs',
          component: () => import('../views/Logs.vue'),
        },
      ],
    },
  ],
})

router.beforeEach((to, _from, next) => {
  const auth = useAuthStore()
  if (to.meta.guest) {
    if (auth.isLoggedIn()) {
      return next('/')
    }
    return next()
  }
  if (!auth.isLoggedIn()) {
    return next('/login')
  }
  next()
})

export default router
