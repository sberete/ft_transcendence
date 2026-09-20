import { ref } from 'vue'

const token = ref(localStorage.getItem('token'))

export function useAuth() {
  function setToken(value) {
    token.value = value
    localStorage.setItem('token', value)
  }

  function clearToken() {
    token.value = null
    localStorage.removeItem('token')
  }

  return { token, setToken, clearToken }
}
