import { createRouter, createWebHistory } from 'vue-router'
import FriendsView from '../views/FriendsView.vue'
import HomeView from '../views/HomeView.vue'
import LoginView from '../views/LoginView.vue'
import OAuthCallbackView from '../views/OAuthCallbackView.vue'
import ProfileView from '../views/ProfileView.vue'
import SignupView from '../views/SignupView.vue'
import { useAuth } from '../services/auth'

// meta.requiresAuth : redirige vers /login si pas de token.
// meta.guestOnly    : redirige vers / si deja connecte (login, signup).
// meta.bare         : page plein ecran sans header (formulaires d'auth).
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
    // Import dynamique : hls.js (~500 Ko) n'est charge que si on ouvre un lecteur.
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
      path: '/room/:id',
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
