// App route table and the global navigation guard (auth required / guest
// only). Heavy views (video player, rooms, search) are dynamically
// imported so they don't bloat the initial bundle.
import { createRouter, createWebHistory } from 'vue-router'
import FriendsView from '../views/FriendsView.vue'
import HomeView from '../views/HomeView.vue'
import LoginView from '../views/LoginView.vue'
import OAuthCallbackView from '../views/OAuthCallbackView.vue'
import ProfileView from '../views/ProfileView.vue'
import SignupView from '../views/SignupView.vue'
import { useAuth } from '../services/auth'

// meta.requiresAuth : redirects to /login if there is no token.
// meta.guestOnly    : redirects to / if already logged in (login, signup).
// meta.bare         : full-screen page with no header (auth forms).
const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: HomeView },
    { path: '/login', component: LoginView, meta: { guestOnly: true, bare: true } },
    { path: '/signup', component: SignupView, meta: { guestOnly: true, bare: true } },
    { path: '/oauth/callback', component: OAuthCallbackView, meta: { bare: true } },
    { path: '/profile', component: ProfileView, meta: { requiresAuth: true } },
    { path: '/friends', component: FriendsView, meta: { requiresAuth: true } },
    {
      path: '/search',
      component: () => import('../views/SearchView.vue'),
      meta: { requiresAuth: true },
    },
    // Dynamic import: hls.js (~500 KB) is only loaded when a player is opened.
    {
      path: '/watch/:provider/:id',
      component: () => import('../views/WatchView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/rooms',
      component: () => import('../views/RoomsView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/room/:code',
      component: () => import('../views/RoomView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/privacy',
      component: () => import('../views/PrivacyView.vue'),
    },
    {
      path: '/terms',
      component: () => import('../views/TermsView.vue'),
    },
    { path: '/media', redirect: '/' },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
  scrollBehavior(to, from, savedPosition) {
    return savedPosition || { top: 0 }
  },
})

// Enforces the requiresAuth/guestOnly rules before every navigation.
router.beforeEach((to) => {
  const { token } = useAuth()
  if (to.meta.requiresAuth && !token.value) {
    return { path: '/login', query: { redirect: to.fullPath } }
  }
  if (to.meta.guestOnly && token.value) {
    return '/'
  }
})

export default router
