<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { api, type BangumiItem, type ImdbItem, type GetRecordData, type EpisodeItem } from '../api'
import { Message, Modal } from '@arco-design/web-vue'
import { IconArrowLeft, IconDelete, IconDown, IconUp, IconRefresh } from '@arco-design/web-vue/es/icon'

const props = defineProps<{ bangumi_id?: string; imdb_id?: string }>()
const router = useRouter()

const info = ref<BangumiItem | ImdbItem | null>(null)
const recorder = ref<GetRecordData | null>(null)
const loading = ref(true)
const updating = ref(false)
const adding = ref(false)
const removing = ref(false)
const epInput = ref<number | undefined>(undefined)
const timeInput = ref('')
const userStatus = ref<number>(1)

const isImdb = computed(() => !!props.imdb_id)
const mediaId = computed(() => props.imdb_id || props.bangumi_id || '')
const hasRecord = computed(() => !!recorder.value?.date && recorder.value?.is_delete === false)

const typeLabels: Record<number, string> = {
  1: 'TV',
  2: '剧场版',
  3: 'OVA',
  4: 'ONA',
  5: 'TV SP',
  6: 'Music',
  7: '书籍',
  8: '其他',
  9: '游戏',
  10: '三次元',
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

const episodeExpanded = ref(false)
const episodeList = ref<EpisodeItem[]>([])
const episodeLoading = ref(false)

async function loadEpisodes(force = false) {
  if (isImdb.value || !props.bangumi_id) return
  if (!force && episodeList.value.length > 0) {
    episodeExpanded.value = !episodeExpanded.value
    return
  }
  episodeLoading.value = true
  episodeExpanded.value = true
  try {
    const res = await api.listEpisodes(parseInt(props.bangumi_id), force)
    if (res.status === 0 && res.data) {
      episodeList.value = res.data
    }
  } catch {
    Message.error('获取剧集列表失败')
  } finally {
    episodeLoading.value = false
  }
}

async function toggleWatched(ep: EpisodeItem) {
  if (!props.bangumi_id) return
  const newWatched = !ep.watched
  const res = await api.updateEpisode(parseInt(props.bangumi_id), ep.ordinal, { watched: newWatched })
  if (res.status === 0 && res.data) {
    ep.watched = res.data.watched
    ep.progress_seconds = res.data.progress_seconds
    ep.completed_at = res.data.completed_at
    ep.updated_at = res.data.updated_at
    await fetchData()
  } else {
    Message.error(res.message || '更新失败')
  }
}

function formatTime(sec: number | null): string {
  if (sec == null || sec <= 0) return ''
  const m = Math.floor(sec / 60)
  const s = sec % 60
  return `${m}:${s.toString().padStart(2, '0')}`
}

async function fetchData() {
  loading.value = true
  try {
    const [infoRes, recordRes] = isImdb.value
      ? await Promise.all([
          api.searchImdbById(mediaId.value),
          api.getRecordByImdb(mediaId.value),
        ])
      : await Promise.all([
          api.searchBangumiById(parseInt(props.bangumi_id || '0')),
          api.getRecordByBangumi(parseInt(props.bangumi_id || '0')),
        ])
    if (infoRes.status === 0 && infoRes.data) {
      info.value = infoRes.data
    }
    if (recordRes.status === 0 && recordRes.data) {
      recorder.value = recordRes.data
      if (recordRes.data.is_delete) {
        Message.warning('该追踪记录已被删除')
        router.push('/')
        return
      }
      if (recordRes.data.recorder) {
        const parts = recordRes.data.recorder.split('|')
        epInput.value = parseInt(parts[0]) || undefined
        timeInput.value = parts[1] || ''
      } else {
        epInput.value = undefined
        timeInput.value = ''
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
    Message.warning('请输入有效的进度编号')
    return
  }
  if (timeInput.value && !timeInput.value.match(/^\d{1,2}:\d{2}$/)) {
    Message.warning('请输入有效的时间格式 (mm:ss)')
    return
  }

  const recorderValue = timeInput.value ? `${ep}|${timeInput.value}` : String(ep)
  updating.value = true
  try {
    const res = isImdb.value
      ? await api.updateRecordByImdb(mediaId.value, recorderValue)
      : await api.updateRecord(parseInt(props.bangumi_id || '0'), recorderValue)
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
  if (!hasRecord.value) return
  const res = isImdb.value
    ? await api.updateRecordByImdb(mediaId.value, undefined, value)
    : await api.updateRecord(parseInt(props.bangumi_id || '0'), undefined, value)
  if (res.status === 0) {
    Message.success('状态更新成功')
  } else {
    Message.error(res.message || '状态更新失败')
  }
}

async function handleAddRecord() {
  if (!info.value) return
  adding.value = true
  try {
    const res = isImdb.value
      ? await api.addRecord({ source: 'imdb', external_id: mediaId.value, user_status: userStatus.value })
      : await api.addRecord({ bangumi_id: parseInt(props.bangumi_id || '0'), user_status: userStatus.value })
    if (res.status === 0) {
      Message.success('添加追踪成功')
      await fetchData()
    } else {
      Message.error(res.message || '添加失败')
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    adding.value = false
  }
}

async function handleDelete(hardDelete = false) {
  Modal.warning({
    title: hardDelete ? '确认硬删除' : '确认软删除',
    content: hardDelete
      ? `确定要永久删除「${info.value?.title || '该条目'}」的追踪记录吗？此操作不可恢复。`
      : `确定要软删除「${info.value?.title || '该条目'}」的追踪记录吗？之后重新添加可恢复。`,
    hideCancel: false,
    async onOk() {
      removing.value = true
      try {
        const res = isImdb.value
          ? await api.deleteRecordByImdb(mediaId.value, hardDelete)
          : await api.deleteRecordByBangumi(parseInt(props.bangumi_id || '0'), hardDelete)
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
        返回追踪列表
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
            {{ info.title || '条目详情' }}
          </h1>

          <a-descriptions :column="1" style="margin-bottom: 24px" size="large">
            <a-descriptions-item label="类型">
              <a-tag :color="info.type === 1 ? 'arcoblue' : 'purple'">
                {{ typeLabels[info.type] || '其他' }}
              </a-tag>
            </a-descriptions-item>
            <a-descriptions-item v-if="info.author" :label="isImdb ? '创作者' : '原作'">
              {{ info.author }}
            </a-descriptions-item>
            <a-descriptions-item v-if="info.release_date" :label="isImdb ? '发行日期' : '放送日期'">
              {{ info.release_date }}
            </a-descriptions-item>
            <a-descriptions-item v-if="info.episodes" :label="isImdb ? '季数/集数' : '话数'">
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

          <div style="display: flex; gap: 12px; margin-bottom: 24px; flex-wrap: wrap">
            <a-button
              v-if="!hasRecord"
              type="primary"
              :loading="adding"
              @click="handleAddRecord"
            >
              添加追踪
            </a-button>
            <a-button
              v-if="hasRecord"
              type="outline"
              status="danger"
              :loading="removing"
              @click="handleDelete(false)"
            >
              <template #icon><icon-delete /></template>
              软删除
            </a-button>
            <a-button
              v-if="hasRecord"
              type="outline"
              status="danger"
              :loading="removing"
              @click="handleDelete(true)"
            >
              <template #icon><icon-delete /></template>
              硬删除
            </a-button>
          </div>

          <div style="margin-top: 24px">
            <h3 style="font-size: 16px; color: #1d2129; margin-bottom: 16px">追踪状态</h3>
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
            <h3 style="font-size: 16px; color: #1d2129; margin-bottom: 16px">追踪进度</h3>

            <div v-if="hasRecord" style="display: flex; align-items: center; gap: 16px; flex-wrap: wrap">
              <div style="display: flex; align-items: center; gap: 8px; font-size: 15px">
                <span style="color: #4e5969">进度</span>
                <a-input-number
                  v-model="epInput"
                  :min="0"
                  :max="info.episodes || 9999"
                  :style="{ width: '80px' }"
                  placeholder="编号"
                />
              </div>
              <div style="display: flex; align-items: center; gap: 8px; font-size: 15px">
                <span style="color: #4e5969">时间</span>
                <a-input
                  v-model="timeInput"
                  :style="{ width: '100px' }"
                  placeholder="可为空"
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
              <a-tag color="gray" size="large">{{ hasRecord ? '暂未记录进度' : '尚未添加追踪记录' }}</a-tag>
            </div>
          </div>

          <a-divider v-if="!isImdb" />

          <div v-if="!isImdb" style="display: flex; align-items: center; gap: 8px">
            <a-button type="text" size="large" @click="loadEpisodes()">
              {{ episodeExpanded ? '收起剧集列表' : '展开剧集列表' }}
              <icon-down v-if="!episodeExpanded" />
              <icon-up v-else />
            </a-button>
            <a-tooltip content="从 bgm.tv 刷新剧集数据">
              <a-button type="text" size="small" @click="loadEpisodes(true)">
                <template #icon><icon-refresh /></template>
              </a-button>
            </a-tooltip>
          </div>

          <a-spin v-if="!isImdb" :loading="episodeLoading">
            <div v-if="episodeExpanded && episodeList.length > 0" style="margin-top: 8px">
              <div
                v-for="ep in episodeList"
                :key="ep.ordinal"
                style="display: flex; align-items: center; gap: 12px; padding: 8px 12px; border-radius: 6px; cursor: pointer; transition: background 0.2s"
                :style="{ background: ep.watched ? '#e8f5e9' : 'transparent' }"
                @mouseenter="($event) => ($event.currentTarget as HTMLElement).style.background = ep.watched ? '#c8e6c9' : '#f5f5f5'"
                @mouseleave="($event) => ($event.currentTarget as HTMLElement).style.background = ep.watched ? '#e8f5e9' : 'transparent'"
                @click="toggleWatched(ep)"
              >
                <a-checkbox :model-value="ep.watched" @click.stop="toggleWatched(ep)" />
                <span style="min-width: 32px; font-weight: 600; color: #1d2129">
                  {{ ep.label || ep.ordinal }}
                </span>
                <span style="flex: 1; color: #4e5969; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
                  {{ ep.name_cn || ep.title || `第 ${ep.label || ep.ordinal} 集` }}
                </span>
                <span v-if="ep.progress_seconds != null && ep.progress_seconds > 0" style="color: #86909c; font-size: 13px">
                  {{ formatTime(ep.progress_seconds) }}
                </span>
                <a-tag v-if="ep.watched" color="green" size="small">已看</a-tag>
                <a-tag v-else color="gray" size="small">未看</a-tag>
              </div>
            </div>
            <div v-else-if="episodeExpanded && !episodeLoading" style="padding: 16px; text-align: center; color: #86909c">
              暂无剧集数据
            </div>
          </a-spin>
        </div>
      </div>
    </a-spin>
  </div>
</template>
