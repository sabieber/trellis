import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import LibraryView from "@/views/LibraryView.vue";
import SearchView from "@/views/SearchView.vue";
import SearchDetailView from "@/views/SearchDetailView.vue";
import AuthView from "@/views/AuthView.vue";
import ShelfDetailView from "@/views/ShelfDetailView.vue";
import BookDetailView from "@/views/BookDetailView.vue";
import ReadingDetailView from "@/views/ReadingDetailView.vue";
import { useAuthStore } from '@/stores/auth';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/library',
      name: 'library',
      component: LibraryView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/search',
      name: 'search',
      component: SearchView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/search/:id',
      name: 'search-detail',
      component: SearchDetailView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/shelf/:id',
      name: 'shelf-detail',
      component: ShelfDetailView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/book/:id',
      name: 'book-detail',
      component: BookDetailView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/reading/:id',
      name: 'reading-detail',
      component: ReadingDetailView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/profile',
      name: 'profile',
      component: () => import('@/views/ProfileView.vue'),
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/login',
      name: 'login',
      component: AuthView,
      meta: { hideNav: true },
    },
    {
      path: '/register',
      name: 'register',
      component: AuthView,
      meta: { hideNav: true },
    }
  ],
})

router.beforeEach((to, _from, next) => {
  if (to.meta.requiresAuth) {
    const auth = useAuthStore()
    if (auth.isAuthenticated) {
      next()
    } else {
      next('/login')
    }
  } else {
    next()
  }
})

export default router
