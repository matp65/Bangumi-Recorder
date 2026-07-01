<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { api, type DetailListItem } from '../api'
import { Message, Modal } from '@arco-design/web-vue'
import { IconDelete } from '@arco-design/web-vue/es/icon'
import dayjs from 'dayjs'

const router = useRouter()
const loading = ref(true)
const records = ref<DetailListItem[]>([])
const filterStatus = ref<number>(Number(localStorage.getItem('dashboard.filterStatus') ?? -1))
const searchKeyword = ref(localStorage.getItem('dashboard.searchKeyword') ?? '')
const sortBy = ref<'name' | 'time'>((localStorage.getItem('dashboard.sortBy') as 'name' | 'time') || 'time')
const sortOrder = ref<'asc' | 'desc'>((localStorage.getItem('dashboard.sortOrder') as 'asc' | 'desc') || 'desc')

const filterOptions = [
  { label: '全部', value: -1 },
  { label: '想看', value: 0 },
  { label: '在看', value: 1 },
  { label: '看过', value: 2 },
  { label: '搁置', value: 3 },
  { label: '抛弃', value: 4 },
]

const filteredRecords = computed(() => {
  const keyword = searchKeyword.value.trim().toLowerCase()
  const filtered = records.value.filter(r => {
    const statusMatched = filterStatus.value === -1 || r.user_status === filterStatus.value
    const keywordMatched = !keyword || [r.title, r.bangumi_id, r.imdb_id, r.other_id]
      .filter(v => v !== null && v !== undefined)
      .some(v => String(v).toLowerCase().includes(keyword))
    return statusMatched && keywordMatched
  })

  return [...filtered].sort((a, b) => {
    const factor = sortOrder.value === 'asc' ? 1 : -1
    if (sortBy.value === 'name') {
      return factor * (a.title || '').localeCompare(b.title || '', 'zh-Hans-CN')
    }
    return factor * (dayjs(a.updated_at).valueOf() - dayjs(b.updated_at).valueOf())
  })
})

watch(filterStatus, value => localStorage.setItem('dashboard.filterStatus', String(value)))
watch(searchKeyword, value => localStorage.setItem('dashboard.searchKeyword', value))
watch(sortBy, value => localStorage.setItem('dashboard.sortBy', value))
watch(sortOrder, value => localStorage.setItem('dashboard.sortOrder', value))

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

function getStatusLabel(status: number | null | undefined) {
  if (status === null || status === undefined) return ''
  return statusLabels[status] || ''
}

function getStatusColor(status: number | null | undefined) {
  if (status === 2) return 'green'
  if (status === 1) return 'arcoblue'
  if (status === 3) return 'orange'
  if (status === 4) return 'red'
  return 'gray'
}

function getTypeLabel(type: number | null) {
  return type ? typeLabels[type] || '其他' : '未知'
}

function getCoverUrl(item: DetailListItem) {
  if (item.cover_url) return item.cover_url
  return undefined
}

function isBangumi(item: DetailListItem) {
  return item.source === 'bangumi' || !!item.bangumi_id
}

function isImdb(item: DetailListItem) {
  return item.source === 'imdb' || !!item.imdb_id
}

async function fetchRecords() {
  loading.value = true
  try {
    const res = await api.getDetailList()
    if (res.status === 0 && res.data) {
      records.value = res.data.filter(r => !r.is_delete)
    } else {
      Message.error('获取追番列表失败')
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    loading.value = false
  }
}

function goDetail(bangumiId: string) {
  router.push({ name: 'Detail', params: { bangumi_id: bangumiId } })
}

function goItemDetail(item: DetailListItem) {
  if (isBangumi(item) && item.bangumi_id) {
    router.push({ name: 'Detail', params: { bangumi_id: item.bangumi_id } })
  } else if (isImdb(item) && item.imdb_id) {
    router.push({ name: 'ImdbDetail', params: { imdb_id: item.imdb_id } })
  } else if (item.other_id) {
    router.push({ name: 'CustomDetail', params: { other_id: item.other_id } })
  }
}

const deleting = ref<Record<string, boolean>>({})

async function handleDelete(item: DetailListItem, hardDelete = false) {
  if (isBangumi(item)) {
    if (!item.bangumi_id) return
    Modal.warning({
      title: hardDelete ? '确认硬删除' : '确认软删除',
      content: hardDelete
        ? `确定要永久删除「${item.title || '未知标题'}」的追踪记录吗？此操作不可恢复。`
        : `确定要软删除「${item.title || '未知标题'}」的追踪记录吗？之后重新添加可恢复。`,
      hideCancel: false,
      async onOk() {
        deleting.value[String(item.id)] = true
        try {
          const res = await api.deleteRecordByBangumi(parseInt(item.bangumi_id!), hardDelete)
          if (res.status === 0) {
            Message.success('删除成功')
            records.value = records.value.filter(r => r.id !== item.id)
          } else {
            Message.error(res.message || '删除失败')
          }
        } catch {
          Message.error('网络请求失败')
        } finally {
          deleting.value[String(item.id)] = false
        }
      },
    })
  } else if (isImdb(item)) {
    if (!item.imdb_id) return
    Modal.warning({
      title: hardDelete ? '确认硬删除' : '确认软删除',
      content: hardDelete
        ? `确定要永久删除「${item.title || '未知标题'}」的 IMDb 追踪记录吗？此操作不可恢复。`
        : `确定要软删除「${item.title || '未知标题'}」的 IMDb 追踪记录吗？之后重新添加可恢复。`,
      hideCancel: false,
      async onOk() {
        deleting.value[String(item.id)] = true
        try {
          const res = await api.deleteRecordByImdb(item.imdb_id!, hardDelete)
          if (res.status === 0) {
            Message.success('删除成功')
            records.value = records.value.filter(r => r.id !== item.id)
          } else {
            Message.error(res.message || '删除失败')
          }
        } catch {
          Message.error('网络请求失败')
        } finally {
          deleting.value[String(item.id)] = false
        }
      },
    })
  } else {
    if (!item.other_id) return
    Modal.warning({
      title: hardDelete ? '确认硬删除' : '确认软删除',
      content: hardDelete
        ? `确定要永久删除「${item.title || '未知标题'}」的自定义条目记录吗？此操作不可恢复。`
        : `确定要软删除「${item.title || '未知标题'}」的自定义条目记录吗？之后重新添加可恢复。`,
      hideCancel: false,
      async onOk() {
        deleting.value[String(item.id)] = true
        try {
          const res = await api.deleteRecordByCustom(item.other_id!, hardDelete)
          if (res.status === 0) {
            Message.success('删除成功')
            records.value = records.value.filter(r => r.id !== item.id)
          } else {
            Message.error(res.message || '删除失败')
          }
        } catch {
          Message.error('网络请求失败')
        } finally {
          deleting.value[String(item.id)] = false
        }
      },
    })
  }
}

onMounted(fetchRecords)
</script>

<template>
  <div>
    <div style="margin-bottom: 24px">
      <h2 style="font-size: 20px; color: #1d2129; margin: 0">我的追踪</h2>
      <p style="color: #86909c; font-size: 14px; margin-top: 4px">共 {{ filteredRecords.length }} 个条目</p>
    </div>

    <div style="display: flex; flex-wrap: wrap; align-items: center; gap: 12px; margin-bottom: 20px">
      <a-radio-group
        type="button"
        :model-value="filterStatus"
        @change="(val: any) => filterStatus = val as number"
      >
        <a-radio
          v-for="opt in filterOptions"
          :key="opt.value"
          :value="opt.value"
        >
          {{ opt.label }}
        </a-radio>
      </a-radio-group>

      <a-input-search
        v-model="searchKeyword"
        placeholder="搜索标题或 ID"
        allow-clear
        style="width: 240px"
      />

      <a-select v-model="sortBy" style="width: 120px">
        <a-option value="time">按时间</a-option>
        <a-option value="name">按名字</a-option>
      </a-select>

      <a-select v-model="sortOrder" style="width: 110px">
        <a-option value="desc">降序</a-option>
        <a-option value="asc">升序</a-option>
      </a-select>
    </div>

    <a-spin :loading="loading" style="min-height: 200px">
      <div v-if="filteredRecords.length === 0" style="text-align: center; padding: 80px 0">
        <a-empty :description="filterStatus !== -1 ? '没有符合条件的追番记录' : '还没有追番记录，去搜索添加吧'" />
      </div>
      <div class="card-grid" v-else>
        <a-card
          v-for="item in filteredRecords"
          :key="item.id"
          hoverable
          class="bangumi-card"
          :class="{ 'is-other': !isBangumi(item) }"
          @click="goItemDetail(item)"
          :body-style="{ padding: '16px' }"
        >
          <div style="display: flex; gap: 12px">
            <div style="flex-shrink: 0; width: 100px">
              <img
                v-if="item.cover_url"
                :src="item.cover_url"
                :alt="item.title || ''"
                style="width: 100%; aspect-ratio: 3/4; object-fit: cover; border-radius: 4px; background: #f2f3f5"
                @error="(e: Event) => { (e.target as HTMLImageElement).style.display = 'none' }"
              />
              <div
                v-else
                style="width: 100%; aspect-ratio: 3/4; background: #f2f3f5; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #c9cdd4; font-size: 24px"
              >
                🎬
              </div>
            </div>
            <div style="flex: 1; min-width: 0">
              <div style="display: flex; align-items: center; gap: 6px; margin-bottom: 4px">
                <span style="font-weight: 600; font-size: 14px; color: #1d2129; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
                  {{ item.title || '未知标题' }}
                </span>
                <a-tag v-if="isImdb(item)" color="orangered" size="small">IMDb</a-tag>
                <a-tag v-else-if="!isBangumi(item)" color="purple" size="small">自定义</a-tag>
              </div>
              <div style="font-size: 12px; color: #86909c; margin-bottom: 8px">
                <a-tag :color="getStatusColor(item.user_status)" size="small" style="margin-right: 4px">{{ getStatusLabel(item.user_status) }}</a-tag>
                <template v-if="isBangumi(item)">
                  {{ getTypeLabel(item.type) }} · {{ item.episodes ? item.episodes + '话' : '' }}
                </template>
                <template v-else-if="isImdb(item)">
                  IMDb · {{ getTypeLabel(item.type) }}
                </template>
                <template v-else>
                  {{ item.episodes ? item.episodes + '项' : '' }}
                </template>
              </div>
              <div v-if="item.recorder" style="margin-bottom: 4px">
                <a-tag color="arcoblue" size="small">进度: {{ item.recorder }}</a-tag>
              </div>
              <div style="font-size: 12px; color: #c9cdd4">
                {{ dayjs(item.updated_at).format('YYYY-MM-DD') }}
              </div>
            </div>
            <div style="flex-shrink: 0; display: flex; align-items: flex-start">
              <a-button
                type="text"
                status="danger"
                size="small"
                :loading="deleting[String(item.id)]"
                @click.stop="handleDelete(item, false)"
              >
                <template #icon><icon-delete /></template>
              </a-button>
              <a-button
                type="text"
                status="danger"
                size="small"
                :loading="deleting[String(item.id)]"
                @click.stop="handleDelete(item, true)"
              >
                硬删
              </a-button>
            </div>
          </div>
        </a-card>
      </div>
    </a-spin>
  </div>
</template>
