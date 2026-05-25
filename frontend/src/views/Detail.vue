<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { api, type BangumiItem, type GetRecordData } from '../api'
import { Message, Modal } from '@arco-design/web-vue'
import { IconArrowLeft, IconDelete } from '@arco-design/web-vue/es/icon'

const props = defineProps<{ bangumi_id: string }>()
const router = useRouter()

const info = ref<BangumiItem | null>(null)
const recorder = ref<GetRecordData | null>(null)
const loading = ref(true)
const updating = ref(false)
const removing = ref(false)
const epInput = ref<number | undefined>(undefined)
const timeInput = ref('00:00')
const userStatus = ref<number>(1)

const typeLabels: Record<number, string> = {
  1: 'TV',
  2: '剧场版',
  3: 'OVA',
  4: 'ONA',
  5: 'TV SP',
  6: 'Music',
  7: '书籍',
  8: '其他',
}

const statusLabels: Record<number, string> = {
  0: '想看',
  1: '在看',
  2: '看过',
  3: '搁置',
  4: '抛弃',
}

const progressText = computed(() => {
  if (!recorder.value?.recorder) return '暂未记录进度'
  return recorder.value.recorder
})

async function fetchData() {
  loading.value = true
  try {
    const [infoRes, recordRes] = await Promise.all([
      api.searchBangumiById(parseInt(props.bangumi_id)),
      api.getRecordByBangumi(parseInt(props.bangumi_id)),
    ])
    if (infoRes.status === 0 && infoRes.data) {
      info.value = infoRes.data
    }
    if (recordRes.status === 0 && recordRes.data) {
      recorder.value = recordRes.data
      if (recordRes.data.is_delete) {
        Message.warning('该追番记录已被删除')
        router.push('/')
        return
      }
      if (recordRes.data.recorder) {
        const parts = recordRes.data.recorder.split('|')
        epInput.value = parseInt(parts[0]) || undefined
        timeInput.value = parts[1] || '00:00'
      } else {
        epInput.value = undefined
        timeInput.value = '00:00'
      }
      if (recordRes.data.user_status !== undefined && recordRes.data.user_status !== null) {
        userStatus.value = recordRes.data.user_status
      }
    }
  } catch {
    Message.error('获取数据失败')
  } finally {
    loading.value = false
  }
}

async function handleUpdate() {
  const ep = epInput.value
  if (ep === undefined || ep < 0) {
    Message.warning('请输入有效的集数')
    return
  }
  if (!timeInput.value.match(/^\d{1,2}:\d{2}$/)) {
    Message.warning('请输入有效的时间格式 (mm:ss)')
    return
  }

  const recorderValue = `${ep}|${timeInput.value}`
  updating.value = true
  try {
    const res = await api.updateRecord(parseInt(props.bangumi_id), recorderValue)
    if (res.status === 0) {
      Message.success('进度更新成功')
      recorder.value = { ...recorder.value, recorder: recorderValue } as GetRecordData
    } else {
      Message.error(res.message || '更新失败')
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    updating.value = false
  }
}

async function handleStatusChange(value: number) {
  userStatus.value = value
  const res = await api.updateRecord(parseInt(props.bangumi_id), undefined, value)
  if (res.status === 0) {
    Message.success('状态更新成功')
  } else {
    Message.error(res.message || '状态更新失败')
  }
}

async function handleDelete() {
  Modal.warning({
    title: '确认删除',
    content: `确定要删除「${info.value?.title || '该番剧'}」的追番记录吗？`,
    hideCancel: false,
    async onOk() {
      removing.value = true
      try {
        const res = await api.deleteRecordByBangumi(parseInt(props.bangumi_id))
        if (res.status === 0) {
          Message.success('删除成功')
          router.push('/')
        } else {
          Message.error(res.message || '删除失败')
        }
      } catch {
        Message.error('网络请求失败')
      } finally {
        removing.value = false
      }
    },
  })
}

onMounted(fetchData)
</script>

<template>
  <div>
    <div style="margin-bottom: 24px">
      <a-button type="text" @click="router.push('/')">
        <template #icon><icon-arrow-left /></template>
        返回追番列表
      </a-button>
    </div>

    <a-spin :loading="loading" style="min-height: 300px">
      <div v-if="info" style="display: flex; gap: 24px; flex-wrap: wrap">
        <div style="flex-shrink: 0">
          <img
            v-if="info.cover_url"
            :src="info.cover_url"
            :alt="info.title"
            class="detail-cover"
            style="width: 200px; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.15)"
          />
          <div
            v-else
            class="detail-cover"
            style="width: 200px; height: 280px; background: #f2f3f5; border-radius: 8px; display: flex; align-items: center; justify-content: center; color: #c9cdd4; font-size: 48px"
          >
            🎬
          </div>
        </div>
        <div style="flex: 1; min-width: 300px">
          <h1 style="font-size: 24px; color: #1d2129; margin-bottom: 8px">
            {{ info.title || '番剧详情' }}
          </h1>

          <a-descriptions :column="1" style="margin-bottom: 24px" size="large">
            <a-descriptions-item label="类型">
              <a-tag :color="info.type === 1 ? 'arcoblue' : 'purple'">
                {{ typeLabels[info.type] || '其他' }}
              </a-tag>
            </a-descriptions-item>
            <a-descriptions-item v-if="info.author" label="原作">
              {{ info.author }}
            </a-descriptions-item>
            <a-descriptions-item v-if="info.release_date" label="放送日期">
              {{ info.release_date }}
            </a-descriptions-item>
            <a-descriptions-item v-if="info.episodes" label="话数">
              {{ info.episodes }}
            </a-descriptions-item>
          </a-descriptions>

          <div v-if="info.description" style="margin-bottom: 24px">
            <h3 style="font-size: 16px; color: #1d2129; margin-bottom: 8px">简介</h3>
            <p style="font-size: 14px; color: #4e5969; line-height: 1.8">
              {{ info.description }}
            </p>
          </div>

          <a-divider />

          <div style="display: flex; gap: 12px; margin-bottom: 24px">
            <a-button
              type="outline"
              status="danger"
              :loading="removing"
              @click="handleDelete"
            >
              <template #icon><icon-delete /></template>
              删除此追番
            </a-button>
          </div>

          <div style="margin-top: 24px">
            <h3 style="font-size: 16px; color: #1d2129; margin-bottom: 16px">观看状态</h3>
            <div style="display: flex; align-items: center; gap: 12px">
              <a-select
                :model-value="userStatus"
                :style="{ width: '120px' }"
                @change="(v: any) => handleStatusChange(Number(v))"
              >
                <a-option v-for="(label, val) in statusLabels" :key="Number(val)" :value="Number(val)">
                  {{ label }}
                </a-option>
              </a-select>
              <a-tag :color="userStatus === 2 ? 'green' : userStatus === 1 ? 'arcoblue' : 'gray'">
                {{ statusLabels[userStatus] || '未知' }}
              </a-tag>
            </div>
          </div>

          <div style="margin-top: 24px">
            <h3 style="font-size: 16px; color: #1d2129; margin-bottom: 16px">追番进度</h3>

            <div v-if="recorder" style="display: flex; align-items: center; gap: 16px; flex-wrap: wrap">
              <div style="display: flex; align-items: center; gap: 8px; font-size: 15px">
                <span style="color: #4e5969">第</span>
                <a-input-number
                  v-model="epInput"
                  :min="0"
                  :max="info.episodes || 9999"
                  :style="{ width: '80px' }"
                  placeholder="集"
                />
                <span style="color: #4e5969">集</span>
              </div>
              <div style="display: flex; align-items: center; gap: 8px; font-size: 15px">
                <span style="color: #4e5969">进度</span>
                <a-input
                  v-model="timeInput"
                  :style="{ width: '100px' }"
                  placeholder="如 2:12"
                />
              </div>
              <a-button
                type="primary"
                :loading="updating"
                @click="handleUpdate"
              >
                更新进度
              </a-button>
            </div>

            <div v-if="progressText !== '暂未记录进度'" style="margin-top: 12px">
              <a-tag color="green" size="large">
                当前进度: {{ progressText }}
              </a-tag>
            </div>
            <div v-else style="margin-top: 12px">
              <a-tag color="gray" size="large">暂未记录进度</a-tag>
            </div>
          </div>
        </div>
      </div>
    </a-spin>
  </div>
</template>
