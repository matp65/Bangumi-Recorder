<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { Message } from '@arco-design/web-vue'
import { IconUser, IconLock } from '@arco-design/web-vue/es/icon'

const router = useRouter()
const auth = useAuthStore()

const isLogin = ref(true)
const allowRegister = ref(true)
const rememberAccount = ref(auth.rememberAccount)

const form = reactive({
  username: '',
  password: '',
})

const loading = ref(false)

onMounted(async () => {
  try {
    const config = await auth.getConfig()
    allowRegister.value = config.allow_register
  } catch (error) {
    console.error('Failed to fetch config:', error)
    allowRegister.value = true
  }
  const remembered = auth.getRememberedUsername()
  if (remembered) {
    form.username = remembered
  }
})

async function handleSubmit() {
  if (!form.username || !form.password) {
    Message.warning('请输入用户名和密码')
    return
  }
  loading.value = true
  try {
    let success: boolean
    if (isLogin.value) {
      success = await auth.login(form.username, form.password)
    } else {
      success = await auth.register(form.username, form.password)
    }
    if (success) {
      auth.saveRememberedUsername(form.username)
      Message.success(isLogin.value ? '登录成功' : '注册成功')
      router.push('/')
    } else {
      Message.error(isLogin.value ? '登录失败，请检查用户名和密码' : '注册失败，用户名可能已存在')
    }
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="login-container">
    <a-card class="login-card" :bordered="false">
      <div class="login-header">
        <h1>🎬 Bangumi Recorder</h1>
        <p>{{ isLogin ? '登录以管理你的追番记录' : '创建新账户开始追番' }}</p>
      </div>

      <a-form :model="form" layout="vertical" @submit="handleSubmit">
        <a-form-item field="username">
          <a-input
            v-model="form.username"
            placeholder="用户名"
            :prefix="IconUser"
            size="large"
            allow-clear
          />
        </a-form-item>
        <a-form-item field="password">
          <a-input-password
            v-model="form.password"
            placeholder="密码"
            :prefix="IconLock"
            size="large"
            allow-clear
            @keyup.enter="handleSubmit"
          />
        </a-form-item>
        <a-form-item>
          <a-button
            type="primary"
            html-type="submit"
            size="large"
            long
            :loading="loading"
          >
            {{ isLogin ? '登录' : '注册' }}
          </a-button>
        </a-form-item>
        <a-form-item v-if="isLogin">
          <a-checkbox
            :model-value="rememberAccount"
            @change="(v: any) => { rememberAccount = v; auth.setRememberAccount(v) }"
          >
            记住账号
          </a-checkbox>
        </a-form-item>
      </a-form>

      <div style="text-align: center">
        <a-link v-if="allowRegister" @click="isLogin = !isLogin">
          {{ isLogin ? '没有账户？去注册' : '已有账户？去登录' }}
        </a-link>
      </div>
    </a-card>
  </div>
</template>
