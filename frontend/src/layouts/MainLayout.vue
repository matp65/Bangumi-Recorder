<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { IconList, IconSearch, IconUser, IconPoweroff } from '@arco-design/web-vue/es/icon'

const router = useRouter()
const auth = useAuthStore()

function handleLogout() {
  auth.logout()
  router.push('/login')
}
</script>

<template>
  <a-layout style="min-height: 100vh">
    <a-layout-header style="background: #fff; border-bottom: 1px solid #e5e6eb; padding: 0 24px; display: flex; align-items: center; justify-content: space-between">
      <div style="display: flex; align-items: center; gap: 24px">
        <a-link style="font-size: 18px; font-weight: 700; color: #1d2129; text-decoration: none" @click="router.push('/')">
          🎬 Bangumi Recorder
        </a-link>
        <a-menu mode="horizontal" :selected-keys="[router.currentRoute.value.name as string]" @menu-item-click="(key: string) => router.push({ name: key })" style="border-bottom: none; background: transparent">
          <a-menu-item key="Dashboard">
            <template #icon><icon-list /></template>
            我的追番
          </a-menu-item>
          <a-menu-item key="Search">
            <template #icon><icon-search /></template>
            搜索番剧
          </a-menu-item>
        </a-menu>
      </div>
      <div style="display: flex; align-items: center; gap: 12px">
        <a-tag color="arcoblue">
          <template #icon><icon-user /></template>
          {{ auth.username }}
        </a-tag>
        <a-button type="text" @click="handleLogout">
          <template #icon><icon-poweroff /></template>
        </a-button>
      </div>
    </a-layout-header>
    <a-layout-content style="padding: 24px; max-width: 1200px; margin: 0 auto; width: 100%">
      <router-view />
    </a-layout-content>
  </a-layout>
</template>
