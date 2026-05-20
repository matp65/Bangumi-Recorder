import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '../api'

function decodeJwt(token: string): { exp: number } | null {
  try {
    const parts = token.split('.')
    if (parts.length !== 3) return null
    const payload = JSON.parse(atob(parts[1]))
    return { exp: payload.exp }
  } catch {
    return null
  }
}

export function isTokenExpired(token: string): boolean {
  const decoded = decodeJwt(token)
  if (!decoded) return true
  return Date.now() >= decoded.exp * 1000
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('token'))
  const username = ref<string | null>(localStorage.getItem('username'))
  const nickname = ref<string | null>(null)
  const avatar = ref<string | null>(null)
  const rememberAccount = ref<boolean>(localStorage.getItem('remember_account') === 'true')

  function isLoggedIn() {
    if (!token.value) return false
    if (isTokenExpired(token.value)) {
      logout()
      return false
    }
    return true
  }

  async function login(uname: string, password: string) {
    const res = await api.login(uname, password)
    if (res.status === 0 && res.token) {
      token.value = res.token
      username.value = uname
      localStorage.setItem('token', res.token)
      localStorage.setItem('username', uname)
      await fetchUserInfo()
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
      await fetchUserInfo()
      return true
    }
    return false
  }

  async function fetchUserInfo() {
    if (!token.value) return
    try {
      const info = await api.getUserInfo()
      if (info.id) {
        nickname.value = info.nickname || null
        avatar.value = info.avatar || null
      }
    } catch {
    }
  }

  async function getConfig() {
    return api.getConfig()
  }

  function setRememberAccount(value: boolean) {
    rememberAccount.value = value
    localStorage.setItem('remember_account', String(value))
    if (!value) {
      localStorage.removeItem('remembered_username')
    }
  }

  function saveRememberedUsername(uname: string) {
    if (rememberAccount.value) {
      localStorage.setItem('remembered_username', uname)
    }
  }

  function getRememberedUsername(): string | null {
    return localStorage.getItem('remembered_username')
  }

  function logout() {
    const savedUsername = rememberAccount.value ? username.value : null
    token.value = null
    username.value = null
    nickname.value = null
    avatar.value = null
    localStorage.removeItem('token')
    localStorage.removeItem('username')
    if (rememberAccount.value && savedUsername) {
      localStorage.setItem('remembered_username', savedUsername)
    }
  }

  return { token, username, nickname, avatar, rememberAccount, isLoggedIn, login, register, getConfig, fetchUserInfo, logout, setRememberAccount, saveRememberedUsername, getRememberedUsername }
})
