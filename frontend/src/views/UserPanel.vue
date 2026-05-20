<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api, type UserInfo } from '../api'
import { useAuthStore } from '../stores/auth'
import { Message } from '@arco-design/web-vue'
import { IconCopy, IconRefresh } from '@arco-design/web-vue/es/icon'

const auth = useAuthStore()
const apiToken = ref<string | null>(null)
const loadingToken = ref(false)

const userInfo = ref<UserInfo | null>(null)
const loadingInfo = ref(true)
const savingInfo = ref(false)
const editNickname = ref('')
const editAvatar = ref('')

const oldPassword = ref('')
const newPassword = ref('')
const changingPassword = ref(false)

onMounted(async () => {
  loadingInfo.value = true
  try {
    const info = await api.getUserInfo()
    if (info.id) {
      userInfo.value = info
      editNickname.value = info.nickname || ''
      editAvatar.value = info.avatar || ''
    }
  } catch {
    Message.error('获取用户信息失败')
  } finally {
    loadingInfo.value = false
  }
})

  async function handleSaveInfo() {
    savingInfo.value = true
    try {
      const res = await api.updateUserInfo(editNickname.value || undefined, editAvatar.value || undefined)
      if (res.status === 0) {
        Message.success('个人信息已更新')
        if (userInfo.value) {
          userInfo.value.nickname = editNickname.value
          userInfo.value.avatar = editAvatar.value
        }
        auth.nickname = editNickname.value || null
        auth.avatar = editAvatar.value || null
      } else {
        Message.error(res.message || '更新失败')
      }
    } catch {
      Message.error('网络请求失败')
    } finally {
      savingInfo.value = false
    }
  }

async function handleChangePassword() {
  if (!oldPassword.value || !newPassword.value) {
    Message.warning('请填写原密码和新密码')
    return
  }
  if (newPassword.value.length < 6) {
    Message.warning('新密码至少需要6位')
    return
  }
  changingPassword.value = true
  try {
    const res = await api.updatePassword(oldPassword.value, newPassword.value)
    if (res.status === 0) {
      Message.success('密码已更新，请重新登录')
      oldPassword.value = ''
      newPassword.value = ''
      setTimeout(() => {
        auth.logout()
        window.location.href = '/login'
      }, 1500)
    } else {
      Message.error(res.message || '修改密码失败')
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    changingPassword.value = false
  }
}

async function handleRegenerate() {
  loadingToken.value = true
  try {
    const res = await api.regenerateToken()
    if (res.status === 0 && res.api_token) {
      apiToken.value = res.api_token
      Message.success('API Token 已重新生成')
    } else {
      Message.error(res.message || '生成失败')
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    loadingToken.value = false
  }
}

async function handleCopy() {
  if (!apiToken.value) return
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(apiToken.value)
    } else {
      const textarea = document.createElement('textarea')
      textarea.value = apiToken.value
      textarea.style.position = 'fixed'
      textarea.style.opacity = '0'
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
    }
    Message.success('已复制到剪贴板')
  } catch {
    Message.error('复制失败')
  }
}
</script>

<template>
  <div style="max-width: 600px">
    <h2 style="font-size: 20px; color: #1d2129; margin-bottom: 24px">用户设置</h2>

    <a-spin :loading="loadingInfo">
      <a-card :bordered="false" style="margin-bottom: 16px">
        <template #title>基本信息</template>
        <div style="display: flex; align-items: center; gap: 16px; margin-bottom: 20px">
          <div style="width: 48px; height: 48px; border-radius: 50%; background: #165dff; display: flex; align-items: center; justify-content: center; color: #fff; font-size: 20px; font-weight: 600">
            {{ (auth.username || '?')[0].toUpperCase() }}
          </div>
          <div>
            <div style="font-weight: 600; font-size: 16px; color: #1d2129">{{ auth.username }}</div>
            <div v-if="userInfo?.reg_time" style="font-size: 12px; color: #86909c">注册于 {{ userInfo.reg_time }}</div>
          </div>
        </div>

        <a-form layout="vertical" :model="{}" :style="{ marginBottom: 0 }">
          <a-form-item label="昵称">
            <a-input v-model="editNickname" placeholder="设置昵称" />
          </a-form-item>
          <a-form-item v-if="userInfo?.email" label="邮箱">
            <a-input :model-value="userInfo.email" disabled />
          </a-form-item>
          <a-form-item label="头像 URL">
            <a-input v-model="editAvatar" placeholder="头像图片链接（可选）" />
          </a-form-item>
          <a-form-item>
            <a-button type="primary" :loading="savingInfo" @click="handleSaveInfo">保存</a-button>
          </a-form-item>
        </a-form>
      </a-card>

      <a-card :bordered="false" style="margin-bottom: 16px">
        <template #title>修改密码</template>
        <a-form layout="vertical" :model="{}" :style="{ marginBottom: 0 }">
          <a-form-item label="原密码">
            <a-input-password v-model="oldPassword" placeholder="输入原密码" />
          </a-form-item>
          <a-form-item label="新密码">
            <a-input-password v-model="newPassword" placeholder="输入新密码（至少6位）" />
          </a-form-item>
          <a-form-item>
            <a-button type="primary" :loading="changingPassword" @click="handleChangePassword">修改密码</a-button>
          </a-form-item>
        </a-form>
      </a-card>
    </a-spin>

    <a-card :bordered="false">
      <template #title>API Token</template>
      <template #extra>
        <a-button type="primary" size="small" :loading="loadingToken" @click="handleRegenerate">
          <template #icon><icon-refresh /></template>
          重新生成
        </a-button>
      </template>

      <div v-if="!apiToken" style="padding: 24px 0; text-align: center; color: #86909c">
        <p>API Token 仅在生成时显示一次，请点击「重新生成」来获取新的 Token</p>
        <p style="font-size: 12px; margin-top: 8px; color: #c9cdd4">用于 Open API 鉴权，使用 ?token= 参数传递</p>
      </div>

      <div v-else>
        <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 12px">
          <a-tag color="green">新生成的 Token</a-tag>
        </div>
        <div style="display: flex; align-items: center; gap: 8px">
          <a-input
            :model-value="apiToken"
            readonly
            style="font-family: monospace; font-size: 13px"
          />
          <a-button @click="handleCopy">
            <template #icon><icon-copy /></template>
          </a-button>
        </div>
        <div style="margin-top: 12px; font-size: 12px; color: #e6a23c">
          请立即复制并妥善保存，关闭页面后将无法再次查看
        </div>
      </div>
    </a-card>
  </div>
</template>
