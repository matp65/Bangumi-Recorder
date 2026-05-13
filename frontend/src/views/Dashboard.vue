<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { api, type DetailListItem } from '../api'
import { Message, Modal } from '@arco-design/web-vue'
import { IconDelete } from '@arco-design/web-vue/es/icon'
import dayjs from 'dayjs'

const router = useRouter()
const loading = ref(true)
const records = ref<DetailListItem[]>([])

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
  1: '想看',
  2: '在看',
  3: '看过',
  4: '搁置',
  5: '抛弃',
}

function getTypeLabel(type: number | null) {
  return type ? typeLabels[type] || '其他' : '未知'
}

function getCoverUrl(item: DetailListItem) {
  if (item.cover_url) return item.cover_url
  return undefined
}

async function fetchRecords() {
  loading.value = true
  try {
    const res = await api.getDetailList()
    if (res.status === 0 && res.data) {
      records.value = res.data
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

const deleting = ref<Record<string, boolean>>({})

async function handleDelete(item: DetailListItem) {
  if (!item.bangumi_id) return
  Modal.warning({
    title: '确认删除',
    content: `确定要删除「${item.title || '未知标题'}」的追番记录吗？`,
    hideCancel: false,
    async onOk() {
      deleting.value[item.id] = true
      try {
        const res = await api.deleteRecord(parseInt(item.bangumi_id!))
        if (res.status === 0) {
          Message.success('删除成功')
          records.value = records.value.filter(r => r.id !== item.id)
        } else {
          Message.error(res.message || '删除失败')
        }
      } catch {
        Message.error('网络请求失败')
      } finally {
        deleting.value[item.id] = false
      }
    },
  })
}

onMounted(fetchRecords)
</script>

<template>
  <div>
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px">
      <div>
        <h2 style="font-size: 20px; color: #1d2129">我的追番</h2>
        <p style="color: #86909c; font-size: 14px; margin-top: 4px">共 {{ records.length }} 部</p>
      </div>
      <a-button type="primary" @click="router.push({ name: 'Search' })">
        搜索并添加番剧
      </a-button>
    </div>

    <a-spin :loading="loading" style="min-height: 200px">
      <div v-if="records.length === 0" style="text-align: center; padding: 80px 0">
        <a-empty description="还没有追番记录，去搜索添加吧" />
      </div>
      <div class="card-grid" v-else>
        <a-card
          v-for="item in records"
          :key="item.id"
          hoverable
          class="bangumi-card"
          @click="item.bangumi_id && goDetail(item.bangumi_id)"
          :body-style="{ padding: '12px' }"
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
              <div style="font-weight: 600; font-size: 14px; color: #1d2129; margin-bottom: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
                {{ item.title || '未知标题' }}
              </div>
              <div style="font-size: 12px; color: #86909c; margin-bottom: 8px">
                {{ getTypeLabel(item.type) }} · {{ item.episodes ? item.episodes + '话' : '' }}
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
                :loading="deleting[item.id]"
                @click.stop="handleDelete(item)"
              >
                <template #icon><icon-delete /></template>
              </a-button>
            </div>
          </div>
        </a-card>
      </div>
    </a-spin>
  </div>
</template>
