import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import LibraryView from "@/views/LibraryView.vue";
import SearchView from "@/views/SearchView.vue";
import SearchDetailView from "@/views/SearchDetailView.vue";
import AuthView from "@/views/AuthView.vue";
import ShelfDetailView from "@/views/ShelfDetailView.vue";
import BookDetailView from "@/views/BookDetailView.vue";
import ReadingDetailView from "@/views/ReadingDetailView.vue";
import GoalsView from "@/views/GoalsView.vue";
import GoalDetailView from "@/views/GoalDetailView.vue";
import StatsView from "@/views/StatsView.vue";
import AuthorDetailView from "@/views/AuthorDetailView.vue";
import SeriesDetailView from "@/views/SeriesDetailView.vue";
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
      path: '/books',
      name: 'books',
      component: () => import('@/views/BooksView.vue'),
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
      path: '/goals',
      name: 'goals',
      component: GoalsView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/goals/:id',
      name: 'goal-detail',
      component: GoalDetailView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/calendar',
      name: 'calendar',
      component: () => import('@/views/CalendarView.vue'),
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/stats',
      name: 'stats',
      component: StatsView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/author/:name',
      name: 'author-detail',
      component: AuthorDetailView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/series/:key',
      name: 'series-detail',
      component: SeriesDetailView,
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

// A deploy replaces the hashed chunk files, so a tab that still runs the old
// build cannot load a lazy route any more. Reload to pick up the new build.
const STALE_CHUNK_RELOAD_KEY = 'trellis:stale-chunk-reload';

router.onError((error) => {
  if (!/dynamically imported module|Importing a module script failed/i.test(String(error))) {
    return;
  }
  // ponytail: reload once per tab. If the chunk stays unreachable for another
  // reason, a second failure must not put the app into a reload loop.
  if (sessionStorage.getItem(STALE_CHUNK_RELOAD_KEY)) {
    return;
  }
  sessionStorage.setItem(STALE_CHUNK_RELOAD_KEY, '1');
  window.location.reload();
});

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
