<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api, type UserInfo, type ApiTokenItem, type PermissionLabel, type PermissionLabelsResponse } from '../api'
import { useAuthStore } from '../stores/auth'
import { Message } from '@arco-design/web-vue'
import { IconCopy, IconDelete, IconEdit, IconPlus, IconRefresh } from '@arco-design/web-vue/es/icon'

const auth = useAuthStore()

const userInfo = ref<UserInfo | null>(null)
const loadingInfo = ref(true)
const savingInfo = ref(false)
const editNickname = ref('')
const editAvatar = ref('')

const oldPassword = ref('')
const newPassword = ref('')
const changingPassword = ref(false)

// Token management
const tokens = ref<ApiTokenItem[]>([])
const loadingTokens = ref(false)
const showCreateModal = ref(false)
const showEditModal = ref(false)
const editingTokenId = ref(0)
const newTokenName = ref('')
const newTokenPermissions = ref(0)
const createdRawToken = ref('')
const permissionLabels = ref<PermissionLabel[]>([])
const permissionAllValue = ref(255)
const creatingToken = ref(false)

const editedTokenName = ref('')
const editedTokenPermissions = ref(0)

function togglePerm(perms: number, permValue: number): number {
  const av = permissionAllValue.value
  if (permValue === av) {
    return (perms & av) === av ? 0 : av
  }
  const isSet = (perms & permValue) !== 0
  return isSet ? (perms ^ permValue) : (perms | permValue)
}

function hasPerm(perms: number, permValue: number): boolean {
  const av = permissionAllValue.value
  if (permValue === av) return (perms & av) === av
  return (perms & permValue) !== 0
}

onMounted(async () => {
  await Promise.all([loadUserInfo(), loadTokens(), loadPermissionLabels()])
})

async function loadUserInfo() {
  loadingInfo.value = true
  try {
    const res = await api.getUserInfo()
    if (res.status === 0 && res.data?.id) {
      userInfo.value = res.data
      editNickname.value = res.data.nickname || ''
      editAvatar.value = res.data.avatar || ''
    }
  } catch {
    Message.error('获取用户信息失败')
  } finally {
    loadingInfo.value = false
  }
}

async function loadTokens() {
  loadingTokens.value = true
  try {
    const res = await api.listTokens()
    if (res.status === 0) {
      tokens.value = res.data || []
    }
  } catch {
    Message.error('获取 Token 列表失败')
  } finally {
    loadingTokens.value = false
  }
}

async function loadPermissionLabels() {
  try {
    const res = await api.getPermissionLabels()
    if (res.status === 0 && res.data) {
      permissionLabels.value = res.data.labels || []
      permissionAllValue.value = res.data.all_value || 255
    }
  } catch {
    // ignore
  }
}

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

function openCreateModal() {
  newTokenName.value = ''
  newTokenPermissions.value = permissionLabels.value.length > 0 ? permissionLabels.value[0].value : 0
  createdRawToken.value = ''
  showCreateModal.value = true
}

async function handleCreateToken() {
  if (!newTokenName.value.trim()) {
    Message.warning('请输入 Token 名称')
    return
  }
  if (newTokenPermissions.value === 0) {
    Message.warning('请至少选择一个权限')
    return
  }
  creatingToken.value = true
  try {
    const res = await api.createToken(newTokenName.value.trim(), newTokenPermissions.value)
    if (res.status === 0 && res.data) {
      createdRawToken.value = res.data.raw_token
      Message.success('Token 创建成功')
      await loadTokens()
    } else {
      Message.error(res.message || '创建失败')
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    creatingToken.value = false
  }
}

function openEditModal(token: ApiTokenItem) {
  editingTokenId.value = token.id
  editedTokenName.value = token.name
  editedTokenPermissions.value = token.permissions
  showEditModal.value = true
}

async function handleEditToken() {
  if (!editedTokenName.value.trim()) {
    Message.warning('请输入 Token 名称')
    return
  }
  if (editedTokenPermissions.value === 0) {
    Message.warning('请至少选择一个权限')
    return
  }
  try {
    const res = await api.updateToken(editingTokenId.value, {
      name: editedTokenName.value.trim(),
      permissions: editedTokenPermissions.value,
    })
    if (res.status === 0) {
      Message.success('Token 已更新')
      showEditModal.value = false
      await loadTokens()
    } else {
      Message.error(res.message || '更新失败')
    }
  } catch {
    Message.error('网络请求失败')
  }
}

async function handleDeleteToken(id: number, name: string) {
  try {
    const res = await api.deleteToken(id)
    if (res.status === 0) {
      Message.success(`Token "${name}" 已删除`)
      await loadTokens()
    } else {
      Message.error(res.message || '删除失败')
    }
  } catch {
    Message.error('网络请求失败')
  }
}

function permLabels(perms: number): string {
  const av = permissionAllValue.value
  if ((perms & av) === av) return 'Allow All'
  const labels = permissionLabels.value
    .filter(p => (perms & p.value) !== 0)
    .map(p => p.label)
  return labels.join(', ') || '-'
}

async function handleCopy(text: string) {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text)
    } else {
      const textarea = document.createElement('textarea')
      textarea.value = text
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
  <div style="max-width: 800px">
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
            <div v-if="userInfo?.uuid" style="font-size: 12px; color: #86909c; font-family: monospace">UUID: {{ userInfo.uuid }}</div>
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
      <template #title>API Token 管理</template>
      <template #extra>
        <a-button type="primary" size="small" @click="openCreateModal">
          <template #icon><icon-plus /></template>
          新建 Token
        </a-button>
      </template>

      <a-spin :loading="loadingTokens">
        <a-table
          v-if="tokens.length > 0"
          :data="tokens"
          :pagination="false"
          :bordered="false"
          style="margin-top: 8px"
        >
          <template #columns>
            <a-table-column title="名称" data-index="name" :width="140" />
            <a-table-column title="权限" :width="260">
              <template #cell="{ record }">
                <span style="font-size: 12px; color: #4e5969">{{ permLabels(record.permissions) }}</span>
              </template>
            </a-table-column>
            <a-table-column title="状态" :width="70">
              <template #cell="{ record }">
                <a-tag :color="record.is_active ? 'green' : 'red'">{{ record.is_active ? '启用' : '禁用' }}</a-tag>
              </template>
            </a-table-column>
            <a-table-column title="最后使用" :width="150">
              <template #cell="{ record }">
                <span style="font-size: 12px; color: #86909c">{{ record.last_used_at || '从未使用' }}</span>
              </template>
            </a-table-column>
            <a-table-column title="创建时间" :width="150">
              <template #cell="{ record }">
                <span style="font-size: 12px; color: #86909c">{{ record.created_at }}</span>
              </template>
            </a-table-column>
            <a-table-column title="操作" :width="100">
              <template #cell="{ record }">
                <a-button type="text" size="small" @click="openEditModal(record)">
                  <template #icon><icon-edit /></template>
                </a-button>
                <a-button type="text" size="small" status="danger" @click="handleDeleteToken(record.id, record.name)">
                  <template #icon><icon-delete /></template>
                </a-button>
              </template>
            </a-table-column>
          </template>
        </a-table>
        <div v-else style="padding: 24px 0; text-align: center; color: #86909c">
          <p>暂无 API Token，请点击「新建 Token」创建</p>
        </div>
      </a-spin>
    </a-card>

    <!-- Create Token Modal -->
    <a-modal
      v-model:visible="showCreateModal"
      title="新建 API Token"
      :footer="false"
      :mask-closable="false"
      width="520px"
    >
      <div v-if="!createdRawToken">
        <a-form layout="vertical" :model="{}">
          <a-form-item label="Token 名称">
            <a-input v-model="newTokenName" placeholder="为 Token 起个名字，如「脚本用」" />
          </a-form-item>
          <a-form-item label="权限设置">
            <div style="display: flex; flex-direction: column; gap: 8px">
              <div
                style="display: flex; align-items: flex-start; gap: 8px; padding-bottom: 8px; border-bottom: 1px solid #f2f3f5; margin-bottom: 4px"
              >
                <a-checkbox
                  :checked="hasPerm(newTokenPermissions, permissionAllValue)"
                  @change="newTokenPermissions = togglePerm(newTokenPermissions, permissionAllValue)"
                />
                <div>
                  <div style="font-size: 14px; font-weight: 500">Allow All</div>
                  <div style="font-size: 12px; color: #86909c">Grant all permissions</div>
                </div>
              </div>
              <div
                v-for="p in permissionLabels"
                :key="p.value"
                style="display: flex; align-items: flex-start; gap: 8px"
              >
                <a-checkbox
                  :checked="hasPerm(newTokenPermissions, p.value)"
                  @change="newTokenPermissions = togglePerm(newTokenPermissions, p.value)"
                />
                <div>
                  <div style="font-size: 14px; font-weight: 500">{{ p.label }}</div>
                  <div v-if="p.description" style="font-size: 12px; color: #86909c">{{ p.description }}</div>
                </div>
              </div>
            </div>
          </a-form-item>
          <a-form-item>
            <div style="display: flex; gap: 8px">
              <a-button @click="showCreateModal = false">取消</a-button>
              <a-button type="primary" :loading="creatingToken" @click="handleCreateToken">创建</a-button>
            </div>
          </a-form-item>
        </a-form>
      </div>
      <div v-else>
        <a-result status="success" title="Token 创建成功">
          <template #subtitle>
            <div style="font-size: 13px; color: #e6a23c; margin-bottom: 8px">
              请立即复制并妥善保存，关闭后将无法再次查看完整 Token
            </div>
          </template>
        </a-result>
        <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 12px">
          <a-input
            :model-value="createdRawToken"
            readonly
            style="font-family: monospace; font-size: 13px"
          />
          <a-button @click="handleCopy(createdRawToken)">
            <template #icon><icon-copy /></template>
          </a-button>
        </div>
        <div style="display: flex; gap: 8px">
          <a-button @click="showCreateModal = false">关闭</a-button>
        </div>
      </div>
    </a-modal>

    <!-- Edit Token Modal -->
    <a-modal
      v-model:visible="showEditModal"
      title="编辑 Token"
      :footer="false"
      :mask-closable="false"
      width="520px"
    >
      <a-form layout="vertical" :model="{}">
        <a-form-item label="Token 名称">
          <a-input v-model="editedTokenName" placeholder="Token 名称" />
        </a-form-item>
        <a-form-item label="权限设置">
          <div style="display: flex; flex-direction: column; gap: 8px">
            <div
              style="display: flex; align-items: flex-start; gap: 8px; padding-bottom: 8px; border-bottom: 1px solid #f2f3f5; margin-bottom: 4px"
            >
              <a-checkbox
                :checked="hasPerm(editedTokenPermissions, permissionAllValue)"
                @change="editedTokenPermissions = togglePerm(editedTokenPermissions, permissionAllValue)"
              />
              <div>
                <div style="font-size: 14px; font-weight: 500">Allow All</div>
                <div style="font-size: 12px; color: #86909c">Grant all permissions</div>
              </div>
            </div>
            <div
              v-for="p in permissionLabels"
              :key="p.value"
              style="display: flex; align-items: flex-start; gap: 8px"
            >
              <a-checkbox
                :checked="hasPerm(editedTokenPermissions, p.value)"
                @change="editedTokenPermissions = togglePerm(editedTokenPermissions, p.value)"
              />
              <div>
                <div style="font-size: 14px; font-weight: 500">{{ p.label }}</div>
                <div v-if="p.description" style="font-size: 12px; color: #86909c">{{ p.description }}</div>
              </div>
            </div>
          </div>
        </a-form-item>
        <a-form-item>
          <div style="display: flex; gap: 8px">
            <a-button @click="showEditModal = false">取消</a-button>
            <a-button type="primary" @click="handleEditToken">保存</a-button>
          </div>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>
