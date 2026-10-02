import { createRouter, createWebHistory } from 'vue-router'
import FriendsView from '../views/FriendsView.vue'
import HomeView from '../views/HomeView.vue'
import LoginView from '../views/LoginView.vue'
import ProfileView from '../views/ProfileView.vue'
import SignupView from '../views/SignupView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: HomeView },
    { path: '/login', component: LoginView },
    { path: '/signup', component: SignupView },
    { path: '/profile', component: ProfileView },
    { path: '/friends', component: FriendsView },
  ],
})

export default router
