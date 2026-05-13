import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '../api'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('token'))
  const username = ref<string | null>(localStorage.getItem('username'))

  function isLoggedIn() {
    return !!token.value
  }

  async function login(uname: string, password: string) {
    const res = await api.login(uname, password)
    if (res.status === 0 && res.token) {
      token.value = res.token
      username.value = uname
      localStorage.setItem('token', res.token)
      localStorage.setItem('username', uname)
      return true
    }
    return false
  }

  async function register(uname: string, password: string) {
    const res = await api.register(uname, password)
    if (res.status === 0 && res.token) {
      token.value = res.token
      username.value = uname
      localStorage.setItem('token', res.token)
      localStorage.setItem('username', uname)
      return true
    }
    return false
  }

  async function getConfig() {
    return api.getConfig()
  }

  function logout() {
    token.value = null
    username.value = null
    localStorage.removeItem('token')
    localStorage.removeItem('username')
  }

  return { token, username, isLoggedIn, login, register, getConfig, logout }
})
