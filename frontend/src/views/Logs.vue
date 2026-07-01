<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Message } from '@arco-design/web-vue'
import { IconDownload, IconRefresh } from '@arco-design/web-vue/es/icon'
import { api, type RecordingLogItem, type SystemLogItem } from '../api'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const recordingLogs = ref<RecordingLogItem[]>([])
const systemLogs = ref<SystemLogItem[]>([])
const loadingRecording = ref(false)
const loadingSystem = ref(false)
const exportingRecording = ref(false)
const activeLogTab = ref(localStorage.getItem('logs.activeTab') || 'recording')
const recordingTimeRange = ref<string[]>([])
const recordingTarget = ref('')
const recordingAction = ref('')
const systemTimeRange = ref<string[]>([])
const systemCategory = ref('')
const systemAction = ref('')
const systemUsername = ref('')

const recordingActionOptions = [
  { label: '进度变更', value: 'recorder_changed' },
  { label: '状态变更', value: 'status_changed' },
  { label: '创建记录', value: 'recording_created' },
  { label: '自定义条目变更', value: 'other_metadata_changed' },
  { label: '剧集记录创建', value: 'episode_created' },
  { label: '剧集记录更新', value: 'episode_updated' },
  { label: '删除记录', value: 'recording_deleted' },
  { label: '硬删除记录', value: 'recording_hard_deleted' },
  { label: '删除自定义条目', value: 'other_recording_deleted' },
]

const systemCategoryOptions = [
  { label: '认证', value: 'auth' },
  { label: 'API Token', value: 'api_token' },
  { label: '日志', value: 'logs' },
  { label: '设置', value: 'settings' },
  { label: '清理', value: 'cleanup' },
]

const systemActionOptions = [
  { label: '登录', value: 'jwt_issued' },
  { label: '创建 API Token', value: 'api_token_created' },
  { label: '修改 API Token', value: 'api_token_updated' },
  { label: '删除 API Token', value: 'api_token_deleted' },
  { label: '读取记录日志', value: 'recording_logs_read' },
  { label: '读取系统日志', value: 'system_logs_read' },
]

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return '-'
  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') return String(value)
  return JSON.stringify(value)
}

function metadataExtra(value: unknown): Record<string, any> {
  if (!value || typeof value !== 'object') return {}
  const metadata = value as Record<string, any>
  return metadata.extra && typeof metadata.extra === 'object' ? metadata.extra : {}
}

function formatSystemMetadata(value: unknown): string {
  if (!value || typeof value !== 'object') return '-'
  const metadata = value as Record<string, any>
  const extra = metadataExtra(value)
  const parts = [
    metadata.auth_type ? `认证: ${metadata.auth_type}` : '',
    metadata.ip ? `IP: ${metadata.ip}` : '',
    extra.username ? `用户: ${extra.username}` : '',
    extra.name ? `Token: ${extra.name}` : '',
    extra.token_id ? `Token ID: ${extra.token_id}` : '',
    extra.permissions !== undefined ? `权限: ${extra.permissions}` : '',
  ].filter(Boolean)

  if (extra.old || extra.new) {
    parts.push(`变更: ${JSON.stringify({ old: extra.old, new: extra.new })}`)
  }

  return parts.length ? parts.join('；') : formatValue(value)
}

function actionLabel(action: string): string {
  const labels: Record<string, string> = {
    recorder_changed: '进度变更',
    status_changed: '状态变更',
    recording_created: '创建记录',
    other_metadata_changed: '自定义条目变更',
    episode_created: '剧集记录创建',
    episode_updated: '剧集记录更新',
    recording_deleted: '删除记录',
    recording_hard_deleted: '硬删除记录',
    other_recording_deleted: '删除自定义条目',
    jwt_issued: '登录',
    api_token_created: '创建 API Token',
    api_token_updated: '修改 API Token',
    api_token_deleted: '删除 API Token',
    recording_logs_read: '读取记录日志',
    system_logs_read: '读取系统日志',
  }
  return labels[action] || action
}

function targetLabel(record: RecordingLogItem): string {
  return record.target_title || `${record.target_type} #${record.target_id || record.recording_id || '-'}`
}

function csvCell(value: unknown): string {
  const text = formatValue(value).replace(/"/g, '""')
  return `"${text}"`
}

function recordingFilters() {
  return {
    start_time: recordingTimeRange.value?.[0],
    end_time: recordingTimeRange.value?.[1],
    target: recordingTarget.value.trim(),
    action: recordingAction.value,
  }
}

async function fetchAllRecordingLogs() {
  const pageSize = 100
  const items: RecordingLogItem[] = []
  for (let page = 1; ; page += 1) {
    const res = await api.listRecordingLogs(page, pageSize, recordingFilters())
    if (res.status !== 0 || !res.data) {
      throw new Error(res.message || '导出记录日志失败')
    }
    items.push(...res.data.items)
    if (res.data.items.length < pageSize) break
  }
  return items
}

async function exportRecordingCsv() {
  exportingRecording.value = true
  try {
    const allLogs = await fetchAllRecordingLogs()
    if (!allLogs.length) {
      Message.warning('没有可导出的记录日志')
      return
    }

    const headers = ['时间', '对象', '动作', '字段', '旧值', '新值', '扩展']
    const rows = allLogs.map(record => [
      record.created_at,
      targetLabel(record),
      actionLabel(record.action),
      record.field_name || '',
      record.old_value,
      record.new_value,
      record.metadata,
    ])
    const csv = [headers, ...rows].map(row => row.map(csvCell).join(',')).join('\n')
    const blob = new Blob([`\ufeff${csv}`], { type: 'text/csv;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `recording-logs-${new Date().toISOString().slice(0, 10)}.csv`
    link.click()
    URL.revokeObjectURL(url)
  } catch (e) {
    Message.error(e instanceof Error ? e.message : '导出记录日志失败')
  } finally {
    exportingRecording.value = false
  }
}

async function loadRecordingLogs() {
  loadingRecording.value = true
  try {
    const res = await api.listRecordingLogs(1, 50, recordingFilters())
    if (res.status === 0 && res.data) {
      recordingLogs.value = res.data.items
    } else {
      Message.error(res.message || '获取记录日志失败')
    }
  } catch {
    Message.error('获取记录日志失败')
  } finally {
    loadingRecording.value = false
  }
}

async function loadSystemLogs() {
  loadingSystem.value = true
  try {
    const res = await api.listSystemLogs(1, 50, {
      start_time: systemTimeRange.value?.[0],
      end_time: systemTimeRange.value?.[1],
      category: systemCategory.value,
      action: systemAction.value,
      username: systemUsername.value.trim(),
    })
    if (res.status === 0 && res.data) {
      systemLogs.value = res.data.items
    } else {
      Message.error(res.message || '获取系统日志失败')
    }
  } catch {
    Message.error('获取系统日志失败')
  } finally {
    loadingSystem.value = false
  }
}

function resetRecordingFilters() {
  recordingTimeRange.value = []
  recordingTarget.value = ''
  recordingAction.value = ''
  loadRecordingLogs()
}

function resetSystemFilters() {
  systemTimeRange.value = []
  systemCategory.value = ''
  systemAction.value = ''
  systemUsername.value = ''
  loadSystemLogs()
}

async function refreshLogs() {
  if (activeLogTab.value === 'system' && auth.isAdmin) {
    await loadSystemLogs()
  } else {
    await loadRecordingLogs()
  }
}

async function handleTabChange(key: string | number) {
  if (key === 'system' && !auth.isAdmin) return
  activeLogTab.value = String(key)
  localStorage.setItem('logs.activeTab', activeLogTab.value)
  await refreshLogs()
}

onMounted(async () => {
  await auth.fetchUserInfo()
  if (activeLogTab.value === 'system' && !auth.isAdmin) {
    activeLogTab.value = 'recording'
    localStorage.setItem('logs.activeTab', activeLogTab.value)
  }
  await refreshLogs()
})
</script>

<template>
  <div>
    <div style="display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; margin-bottom: 20px">
      <div>
        <h2 style="font-size: 20px; color: #1d2129; margin: 0 0 6px">记录日志</h2>
        <div style="color: #86909c; font-size: 13px">查看你的追踪记录变化，可用于导出和年度总结。</div>
      </div>
      <div style="display: flex; gap: 8px">
        <a-button v-if="activeLogTab === 'recording'" size="small" :loading="exportingRecording" @click="exportRecordingCsv">
          <template #icon><icon-download /></template>
          导出 CSV
        </a-button>
        <a-button size="small" @click="refreshLogs">
          <template #icon><icon-refresh /></template>
          刷新
        </a-button>
      </div>
    </div>

    <a-card :bordered="false">
      <a-tabs :active-key="activeLogTab" @change="handleTabChange">
        <a-tab-pane key="recording" title="记录日志">
          <div style="display: flex; flex-wrap: wrap; gap: 12px; margin-bottom: 16px">
            <a-range-picker
              v-model="recordingTimeRange"
              show-time
              value-format="YYYY-MM-DD HH:mm:ss"
              style="width: 360px"
            />
            <a-input-search
              v-model="recordingTarget"
              placeholder="对象标题 / 类型 / ID"
              allow-clear
              style="width: 220px"
              @search="loadRecordingLogs"
            />
            <a-select
              v-model="recordingAction"
              placeholder="动作"
              allow-clear
              style="width: 180px"
              @change="loadRecordingLogs"
            >
              <a-option v-for="opt in recordingActionOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </a-option>
            </a-select>
            <a-button type="primary" @click="loadRecordingLogs">搜索</a-button>
            <a-button @click="resetRecordingFilters">重置</a-button>
          </div>
          <a-table :data="recordingLogs" :loading="loadingRecording" :pagination="{ pageSize: 12 }" :bordered="false">
            <template #columns>
              <a-table-column title="时间" data-index="created_at" :width="170" />
              <a-table-column title="对象" :width="180">
                <template #cell="{ record }">
                  <a-tag>{{ targetLabel(record) }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="动作" :width="150">
                <template #cell="{ record }">{{ actionLabel(record.action) }}</template>
              </a-table-column>
              <a-table-column title="字段" data-index="field_name" :width="110" />
              <a-table-column title="旧值">
                <template #cell="{ record }">
                  <span style="font-family: monospace; font-size: 12px">{{ formatValue(record.old_value) }}</span>
                </template>
              </a-table-column>
              <a-table-column title="新值">
                <template #cell="{ record }">
                  <span style="font-family: monospace; font-size: 12px">{{ formatValue(record.new_value) }}</span>
                </template>
              </a-table-column>
              <a-table-column title="扩展">
                <template #cell="{ record }">
                  <span style="font-family: monospace; font-size: 12px">{{ formatValue(record.metadata) }}</span>
                </template>
              </a-table-column>
            </template>
          </a-table>
        </a-tab-pane>
        <a-tab-pane v-if="auth.isAdmin" key="system" title="系统日志">
          <div style="display: flex; flex-wrap: wrap; gap: 12px; margin-bottom: 16px">
            <a-range-picker
              v-model="systemTimeRange"
              show-time
              value-format="YYYY-MM-DD HH:mm:ss"
              style="width: 360px"
            />
            <a-select
              v-model="systemCategory"
              placeholder="类型"
              allow-clear
              style="width: 150px"
              @change="loadSystemLogs"
            >
              <a-option v-for="opt in systemCategoryOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </a-option>
            </a-select>
            <a-select
              v-model="systemAction"
              placeholder="动作"
              allow-clear
              style="width: 180px"
              @change="loadSystemLogs"
            >
              <a-option v-for="opt in systemActionOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </a-option>
            </a-select>
            <a-input-search
              v-model="systemUsername"
              placeholder="操作用户 / 用户 ID"
              allow-clear
              style="width: 220px"
              @search="loadSystemLogs"
            />
            <a-button type="primary" @click="loadSystemLogs">搜索</a-button>
            <a-button @click="resetSystemFilters">重置</a-button>
          </div>
          <a-table :data="systemLogs" :loading="loadingSystem" :pagination="{ pageSize: 12 }" :bordered="false">
            <template #columns>
              <a-table-column title="时间" data-index="created_at" :width="170" />
              <a-table-column title="类型" :width="130">
                <template #cell="{ record }">
                  <a-tag>{{ record.category }}</a-tag>
                </template>
              </a-table-column>
              <a-table-column title="动作" :width="170">
                <template #cell="{ record }">{{ actionLabel(record.action) }}</template>
              </a-table-column>
              <a-table-column title="用户" :width="140">
                <template #cell="{ record }">{{ record.username || (record.user_id ? `user#${record.user_id}` : '-') }}</template>
              </a-table-column>
              <a-table-column title="说明" data-index="message" />
              <a-table-column title="扩展">
                <template #cell="{ record }">
                  <span style="font-size: 12px; line-height: 1.6">{{ formatSystemMetadata(record.metadata) }}</span>
                </template>
              </a-table-column>
            </template>
          </a-table>
        </a-tab-pane>
      </a-tabs>
    </a-card>
  </div>
</template>
