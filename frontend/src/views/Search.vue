<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { api, type BangumiSearchItem } from '../api'
import { Message } from '@arco-design/web-vue'
import { IconSearch } from '@arco-design/web-vue/es/icon'

const router = useRouter()

const keyword = ref('')
const searching = ref(false)
const adding = ref<Record<string, boolean>>({})
const results = ref<BangumiSearchItem[]>([])
const hasSearched = ref(false)

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

async function handleSearch() {
  if (!keyword.value.trim()) {
    Message.warning('请输入番剧名称')
    return
  }
  searching.value = true
  hasSearched.value = true
  try {
    const res = await api.searchBangumi(keyword.value.trim())
    if (res.status === 0 && res.data) {
      results.value = res.data
      if (res.data.length === 0) {
        Message.info('未找到相关番剧')
      }
    } else {
      Message.error('搜索失败')
      results.value = []
    }
  } catch {
    Message.error('网络请求失败')
    results.value = []
  } finally {
    searching.value = false
  }
}

async function handleAdd(item: BangumiSearchItem) {
  const bangumiId = parseInt(item.bangumi_id)
  adding.value[item.bangumi_id] = true
  try {
    const res = await api.addRecord(bangumiId, 2)
    if (res.status === 0) {
      Message.success(`已添加「${item.title}」到追番列表`)
    } else if (res.status === -3) {
      Message.warning(`「${item.title}」已经在追番列表中`)
    } else if (res.status === -2) {
      Message.error('番剧信息未找到，请先搜索ID获取详情后再添加')
    } else {
      Message.error('添加失败')
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    adding.value[item.bangumi_id] = false
  }
}

function goDetail(bangumiId: string) {
  router.push({ name: 'Detail', params: { bangumi_id: bangumiId } })
}
</script>

<template>
  <div>
    <div class="search-hero">
      <h1>搜索番剧</h1>
      <p>搜索 Bangumi 上的番剧，添加到你到追番列表</p>
      <div class="search-input-wrapper">
        <a-input-search
          v-model="keyword"
          placeholder="输入番剧名称，如「Re:0」「鬼灭之刃」"
          size="large"
          :search-icon="IconSearch"
          :loading="searching"
          search-button
          button-text="搜索"
          @search="handleSearch"
          @press-enter="handleSearch"
        />
      </div>
    </div>

    <a-spin :loading="searching" style="min-height: 200px">
      <div v-if="hasSearched && results.length === 0" style="text-align: center; padding: 40px 0">
        <a-empty description="未找到相关番剧，换个关键词试试" />
      </div>

      <div class="card-grid" v-if="results.length > 0">
        <a-card
          v-for="item in results"
          :key="item.bangumi_id"
          hoverable
          :body-style="{ padding: '16px' }"
          @click="goDetail(item.bangumi_id)"
        >
          <div style="display: flex; gap: 12px">
            <div style="flex-shrink: 0; width: 80px">
              <img
                v-if="item.cover"
                :src="item.cover"
                :alt="item.title"
                style="width: 100%; aspect-ratio: 3/4; object-fit: cover; border-radius: 4px; background: #f2f3f5"
                @error="(e: Event) => { (e.target as HTMLImageElement).style.display = 'none' }"
              />
              <div
                v-else
                style="width: 100%; aspect-ratio: 3/4; background: #f2f3f5; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #c9cdd4; font-size: 20px"
              >
                🎬
              </div>
            </div>
            <div style="flex: 1; min-width: 0">
              <div style="font-weight: 600; font-size: 14px; color: #1d2129; margin-bottom: 4px">
                {{ item.title }}
              </div>
              <div v-if="item.alias" style="font-size: 12px; color: #86909c; margin-bottom: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
                {{ item.alias }}
              </div>
              <div style="font-size: 12px; color: #c9cdd4; margin-bottom: 8px">
                {{ typeLabels[item.type] || '其他' }} · {{ item.info || '' }}
              </div>
              <a-button
                type="primary"
                size="small"
                :loading="adding[item.bangumi_id]"
                @click.stop="handleAdd(item)"
              >
                添加追番
              </a-button>
            </div>
          </div>
        </a-card>
      </div>
    </a-spin>
  </div>
</template>
