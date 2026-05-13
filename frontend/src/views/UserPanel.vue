<script setup lang="ts">
import { ref } from 'vue'
import { api } from '../api'
import { useAuthStore } from '../stores/auth'
import { Message } from '@arco-design/web-vue'
import { IconCopy, IconRefresh } from '@arco-design/web-vue/es/icon'

const auth = useAuthStore()
const apiToken = ref<string | null>(null)
const loading = ref(false)

async function handleRegenerate() {
  loading.value = true
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
    loading.value = false
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

    <a-card :bordered="false" style="margin-bottom: 16px">
      <div style="display: flex; align-items: center; gap: 16px; margin-bottom: 16px">
        <div style="width: 48px; height: 48px; border-radius: 50%; background: #165dff; display: flex; align-items: center; justify-content: center; color: #fff; font-size: 20px; font-weight: 600">
          {{ (auth.username || '?')[0].toUpperCase() }}
        </div>
        <div>
          <div style="font-weight: 600; font-size: 16px; color: #1d2129">{{ auth.username }}</div>
          <div style="font-size: 13px; color: #86909c">用户</div>
        </div>
      </div>
    </a-card>

    <a-card :bordered="false">
      <template #title>API Token</template>
      <template #extra>
        <a-button type="primary" size="small" :loading="loading" @click="handleRegenerate">
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
