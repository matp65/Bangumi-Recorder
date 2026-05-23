<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { api, type BangumiSearchItem, type LocalSearchItem } from '../api'
import { Message } from '@arco-design/web-vue'
import { IconSearch, IconPlus } from '@arco-design/web-vue/es/icon'

const router = useRouter()

const activeTab = ref<'search' | 'custom'>('search')
const useLocal = ref(false)
const keyword = ref('')
const idSearch = ref('')
const searching = ref(false)
const adding = ref<Record<string, boolean>>({})
const results = ref<(BangumiSearchItem | LocalSearchItem)[]>([])
const hasSearched = ref(false)

const currentPage = ref(1)
const totalResults = ref(0)
const pageSize = 20

const customForm = ref({
  title: '',
  description: '',
  cover: '',
  maxNumber: undefined as number | undefined,
  status: 2,
  recorder: '',
})
const creating = ref(false)

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

const statusOptions = [
  { value: 0, label: '想看', color: 'gray' },
  { value: 1, label: '在看', color: 'arcoblue' },
  { value: 2, label: '看过', color: 'green' },
  { value: 3, label: '搁置', color: 'orange' },
  { value: 4, label: '抛弃', color: 'red' },
]

function isBangumi(item: BangumiSearchItem | LocalSearchItem): item is BangumiSearchItem {
  return 'alias' in item
}

function getItemBangumiId(item: BangumiSearchItem | LocalSearchItem) {
  if (isBangumi(item)) return item.bangumi_id
  return item.bangumi_id || null
}

function getItemTitle(item: BangumiSearchItem | LocalSearchItem) {
  return item.title
}

function getItemCover(item: BangumiSearchItem | LocalSearchItem) {
  if (isBangumi(item)) return item.cover
  return item.cover || undefined
}

function getItemType(item: BangumiSearchItem | LocalSearchItem) {
  if (isBangumi(item)) return typeLabels[item.type] || '其他'
  return item.type || '其他'
}

function getItemInfo(item: BangumiSearchItem | LocalSearchItem) {
  if (isBangumi(item)) return item.info || item.alias || ''
  return item.info || ''
}

async function doSearch() {
  if (!keyword.value.trim()) {
    Message.warning('请输入番剧名称')
    return
  }
  searching.value = true
  hasSearched.value = true
  try {
    if (useLocal.value) {
      const res = await api.searchLocal(keyword.value.trim(), undefined, currentPage.value, pageSize)
      if (res.status === 0 && res.data) {
        results.value = res.data
        totalResults.value = res.total || 0
        if (res.data.length === 0) {
          Message.info('本地缓存未找到相关条目')
        }
      } else {
        Message.error('搜索失败')
        results.value = []
        totalResults.value = 0
      }
    } else {
      const res = await api.searchBangumi(keyword.value.trim(), currentPage.value)
      if (res.status === 0 && res.data) {
        results.value = res.data
        totalResults.value = 0
        if (res.data.length === 0 && currentPage.value === 1) {
          Message.info('未找到相关番剧')
        }
      } else {
        Message.error('搜索失败')
        results.value = []
        totalResults.value = 0
      }
    }
  } catch {
    Message.error('网络请求失败')
    results.value = []
    totalResults.value = 0
  } finally {
    searching.value = false
  }
}

async function handleSearch() {
  currentPage.value = 1
  await doSearch()
}

async function handlePageChange(page: number) {
  currentPage.value = page
  await doSearch()
}

async function handleIdSearch() {
  const id = parseInt(idSearch.value.trim())
  if (!id || isNaN(id)) {
    Message.warning('请输入有效的 ID')
    return
  }
  currentPage.value = 1
  searching.value = true
  hasSearched.value = true
  try {
    if (useLocal.value) {
      const res = await api.searchLocal(undefined, id, 1, pageSize)
      if (res.status === 0 && res.data && res.data.length > 0) {
        results.value = res.data
        totalResults.value = res.total || 0
      } else {
        Message.info('本地缓存未找到该 ID')
        results.value = []
        totalResults.value = 0
      }
    } else {
      const res = await api.searchBangumiById(id)
      if (res.status === 0 && res.data) {
        results.value = [{
          bangumi_id: res.data.bangumi_id,
          title: res.data.title,
          alias: '',
          cover: res.data.cover_url,
          info: `${typeLabels[res.data.type] || '其他'} · ${res.data.episodes}话`,
          type: res.data.type,
        }]
        totalResults.value = 0
      } else {
        Message.error('未找到该番剧')
        results.value = []
        totalResults.value = 0
      }
    }
  } catch {
    Message.error('网络请求失败')
    results.value = []
    totalResults.value = 0
  } finally {
    searching.value = false
  }
}

async function handleAdd(item: BangumiSearchItem | LocalSearchItem) {
  const id = getItemBangumiId(item)
  const key = id || `other_${Date.now()}`
  adding.value[key] = true
  try {
    if (isBangumi(item)) {
      const res = await api.addRecord({ bangumi_id: parseInt(item.bangumi_id), user_status: 2 })
      if (res.status === 0) {
        Message.success(`已添加「${item.title}」到追番列表`)
      } else if (res.status === -3) {
        Message.warning(`「${item.title}」已经在追番列表中`)
      } else if (res.status === -2) {
        Message.error('番剧信息未找到，请先搜索ID获取详情后再添加')
      } else {
        Message.error('添加失败')
      }
    } else if (item.other_id) {
      const res = await api.addRecord({ other_id: item.other_id, user_status: 2 })
      if (res.status === 0) {
        Message.success(`已添加「${item.title}」到追番列表`)
      } else if (res.status === -3) {
        Message.warning(`「${item.title}」已经在追番列表中`)
      } else {
        Message.error('添加失败')
      }
    } else if (item.bangumi_id) {
      const res = await api.addRecord({ bangumi_id: parseInt(item.bangumi_id), user_status: 2 })
      if (res.status === 0) {
        Message.success(`已添加「${item.title}」到追番列表`)
      } else if (res.status === -3) {
        Message.warning(`「${item.title}」已经在追番列表中`)
      } else {
        Message.error('添加失败')
      }
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    adding.value[key] = false
  }
}

async function handleAddCustom() {
  if (!customForm.value.title.trim()) {
    Message.warning('请输入条目名称')
    return
  }
  creating.value = true
  try {
    const res = await api.addRecord({
      other_title: customForm.value.title.trim(),
      other_description: customForm.value.description || undefined,
      other_cover: customForm.value.cover || undefined,
      other_max_number: customForm.value.maxNumber,
      other_status: customForm.value.status,
      user_status: customForm.value.status,
      recorder: customForm.value.recorder || undefined,
    })
    if (res.status === 0) {
      Message.success(`已添加「${customForm.value.title}」到追番列表`)
      customForm.value = { title: '', description: '', cover: '', maxNumber: undefined, status: 2, recorder: '' }
    } else if (res.status === -3) {
      Message.warning('该条目已在追番列表中')
    } else {
      Message.error('添加失败')
    }
  } catch {
    Message.error('网络请求失败')
  } finally {
    creating.value = false
  }
}

function goDetail(bangumiId: string) {
  router.push({ name: 'Detail', params: { bangumi_id: bangumiId } })
}
</script>

<template>
  <div>
    <a-tabs v-model:active-key="activeTab" type="card" style="margin-bottom: 24px">
      <a-tab-pane key="search" title="搜索番剧">
        <div class="search-hero">
          <h1>{{ useLocal ? '本地数据搜索' : '搜索番剧' }}</h1>
          <p v-if="useLocal">搜索本地缓存的番剧和自定义条目</p>
          <p v-else>搜索 Bangumi 上的番剧，添加到你到追番列表</p>

          <div style="display: flex; align-items: center; justify-content: center; gap: 16px; margin-bottom: 16px">
            <span style="font-size: 14px; color: #86909c">在线搜索</span>
            <a-switch :model-value="useLocal" @change="(v: any) => useLocal = !!v" size="medium">
              <template #checked>本地</template>
              <template #unchecked>在线</template>
            </a-switch>
          </div>

          <div class="search-input-wrapper">
            <a-input-search
              v-model="keyword"
              :placeholder="useLocal ? '输入关键词搜索本地缓存...' : '输入番剧名称，如「Re:0」「鬼灭之刃」'"
              size="large"
              :search-icon="IconSearch"
              :loading="searching"
              search-button
              button-text="搜索"
              @search="handleSearch"
              @press-enter="handleSearch"
            />
          </div>

          <div style="margin-top: 12px; display: flex; gap: 8px; justify-content: center; align-items: center">
            <span style="font-size: 14px; color: #86909c">或输入 ID:</span>
            <a-input
              v-model="idSearch"
              :placeholder="useLocal ? '本地条目 ID...' : 'Bangumi ID，如 425998'"
              :style="{ width: '200px' }"
              size="large"
              @press-enter="handleIdSearch"
            />
            <a-button type="outline" size="large" :loading="searching" @click="handleIdSearch">
              <template #icon><icon-search /></template>
              ID 搜索
            </a-button>
          </div>
        </div>

        <a-spin :loading="searching" style="min-height: 200px">
          <div v-if="hasSearched && results.length === 0" style="text-align: center; padding: 40px 0">
            <a-empty description="未找到相关番剧，换个关键词试试" />
          </div>

          <div class="card-grid" v-if="results.length > 0">
            <a-card
              v-for="item in results"
              :key="getItemBangumiId(item) || `other_${(item as any).other_id || ''}`"
              hoverable
              :body-style="{ padding: '16px' }"
              @click="getItemBangumiId(item) && goDetail(getItemBangumiId(item)!)"
              :style="{ cursor: getItemBangumiId(item) ? 'pointer' : 'default' }"
            >
              <div style="display: flex; gap: 12px">
                <div style="flex-shrink: 0; width: 80px">
                  <img
                    v-if="getItemCover(item)"
                    :src="getItemCover(item)"
                    :alt="getItemTitle(item)"
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
                  <div style="display: flex; align-items: center; gap: 6px; margin-bottom: 4px">
                    <span style="font-weight: 600; font-size: 14px; color: #1d2129; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
                      {{ getItemTitle(item) }}
                    </span>
                    <a-tag v-if="!isBangumi(item) && (item as LocalSearchItem).other_id" color="purple" size="small">自定义</a-tag>
                  </div>
                  <div v-if="isBangumi(item) && (item as BangumiSearchItem).alias" style="font-size: 12px; color: #86909c; margin-bottom: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
                    {{ (item as BangumiSearchItem).alias }}
                  </div>
                  <div style="font-size: 12px; color: #c9cdd4; margin-bottom: 8px">
                    {{ getItemType(item) }} · {{ getItemInfo(item) }}
                  </div>
                  <a-button
                    type="primary"
                    size="small"
                    :loading="adding[getItemBangumiId(item) || `other_${(item as any).other_id || ''}`]"
                    @click.stop="handleAdd(item)"
                  >
                    添加追番
                  </a-button>
                </div>
              </div>
            </a-card>
          </div>

          <div v-if="results.length > 0 && hasSearched" style="display: flex; justify-content: center; margin-top: 24px">
            <a-pagination
              v-if="useLocal && totalResults > pageSize"
              :current="currentPage"
              :total="totalResults"
              :page-size="pageSize"
              show-total
              @change="handlePageChange"
            />
            <a-pagination
              v-else-if="!useLocal"
              :current="currentPage"
              :total="200"
              :page-size="pageSize"
              show-total
              @change="handlePageChange"
            />
          </div>
        </a-spin>
      </a-tab-pane>

      <a-tab-pane key="custom" title="自定义条目">
        <div class="search-hero">
          <h1>自定义条目</h1>
          <p>添加不属于 Bangumi 的个人追踪条目</p>
        </div>

        <div style="max-width: 600px; margin: 0 auto">
          <a-card :body-style="{ padding: '24px' }">
            <a-form :model="customForm" layout="vertical" size="large">
              <a-form-item label="条目名称" required>
                <a-input
                  v-model="customForm.title"
                  placeholder="如：健身计划、读书清单"
                  :max-length="255"
                />
              </a-form-item>

              <a-form-item label="描述">
                <a-textarea
                  v-model="customForm.description"
                  placeholder="简要描述该条目"
                  :auto-size="{ minRows: 2, maxRows: 4 }"
                  :max-length="2000"
                  show-word-limit
                />
              </a-form-item>

              <a-form-item label="封面图片URL">
                <a-input v-model="customForm.cover" placeholder="https://..." />
              </a-form-item>

              <a-form-item label="总数">
                <a-input-number
                  v-model="customForm.maxNumber"
                  :min="0"
                  :style="{ width: '100%' }"
                  placeholder="如：12话、24集，留空未知"
                />
              </a-form-item>

              <a-form-item label="状态">
                <a-select v-model="customForm.status">
                  <a-option v-for="opt in statusOptions" :key="opt.value" :value="opt.value">
                    {{ opt.label }}
                  </a-option>
                </a-select>
              </a-form-item>

              <a-form-item label="追番进度 (可选)">
                <a-input
                  v-model="customForm.recorder"
                  placeholder="格式：集数|时间，如 5|2:12"
                />
              </a-form-item>

              <a-form-item>
                <a-button
                  type="primary"
                  long
                  :loading="creating"
                  @click="handleAddCustom"
                >
                  <template #icon><icon-plus /></template>
                  添加条目
                </a-button>
              </a-form-item>
            </a-form>
          </a-card>
        </div>
      </a-tab-pane>
    </a-tabs>
  </div>
</template>
