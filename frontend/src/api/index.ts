const API_BASE = ''

interface RequestOptions {
  method?: string
  body?: any
}

async function request<T = any>(url: string, options: RequestOptions = {}): Promise<T> {
  const { method = 'GET', body } = options
  const token = localStorage.getItem('token')

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const res = await fetch(`${API_BASE}${url}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  })

  if (res.status === 401) {
    localStorage.removeItem('token')
    localStorage.removeItem('username')
    window.location.href = '/login'
    return { status: -5 } as T
  }

  const data = await res.json()
  return data as T
}

// ---- v2 unified response wrapper (for reference) ----
// { status: 0|-1, data?: T, message?: string }

export interface ApiResponse<T> {
  status: number
  data?: T
  message?: string
}

// ---- v2 data payload types ----

export interface LoginData {
  token: string
}

export interface RegisterData {
  token: string
  api_token: string
}

export interface ConfigData {
  allow_register: boolean
  register_need_token: boolean
}

export interface BangumiSearchItem {
  source?: 'bangumi'
  bangumi_id: string
  title: string
  alias: string
  cover: string
  info: string
  type: number
}

export interface ImdbSearchItem {
  source: 'imdb'
  imdb_id: string
  external_id: string
  title: string
  year: string | null
  cover: string | null
  info: string
  type: number
}

export interface BangumiItem {
  source?: 'bangumi'
  bangumi_id: string
  title: string
  cover_url: string
  type: number
  author: string
  release_date: string | null
  episodes: number
  description: string
}

export interface ImdbItem {
  source: 'imdb'
  imdb_id: string
  external_id: string
  title: string
  cover_url: string
  type: number
  author: string
  release_date: string | null
  episodes: number
  description: string
}

export interface OtherItem {
  source: 'custom'
  other_id: number
  title: string
  cover_url: string
  type: number
  author: string
  release_date: string | null
  episodes: number
  description: string
  status?: number | null
}

export interface DetailListItem {
  id: number
  source: string | null
  external_id: string | null
  local_external_media_id: number | null
  local_bangumi_id: number | null
  other_id: number | null
  bangumi_id: string | null
  imdb_id: string | null
  title: string | null
  type: number | null
  author: string | null
  episodes: number
  cover_url: string | null
  recorder: string | null
  user_status?: number
  is_delete: boolean
  updated_at: string
  created_at: string
}

export interface RecorderItem {
  id: number
  source: string | null
  external_id: string | null
  local_external_media_id: number | null
  local_bangumi_id: number | null
  bangumi_id: string | null
  imdb_id: string | null
  recorder: string | null
  user_status: number | null
  is_delete: boolean
  updated_at: string
  date: string
}

export interface EpisodeItem {
  ordinal: number
  label: string | null
  title: string | null
  name_cn: string | null
  airdate: string | null
  duration: string | null
  watched: boolean
  progress_seconds: number | null
  duration_seconds: number | null
  completed_at: string | null
  updated_at: string | null
}

export interface BangumiEpisodeMeta {
  ordinal: number
  title: string | null
  name_cn: string | null
  airdate: string | null
  duration: string | null
}

export interface SyncRequestRecord {
  bangumi_id: string
  recorder?: string | null
  user_status?: number | null
  updated_at?: string | null
}

export interface SyncResponseRecord {
  bangumi_id: string
  recorder: string | null
  user_status: number | null
  updated_at: string
}

export interface SyncResponseData {
  records: SyncResponseRecord[]
  deleted: string[]
}

export interface AddRecordData {
  source?: string
  external_id?: string
  imdb_id?: string
  local_external_media_id?: number
  local_bangumi_id?: number
  other_id?: number
  bangumi_id?: number
  recorder?: string
  date?: string
}

export interface GetRecordData {
  source?: string
  external_id?: string
  imdb_id?: string
  local_external_media_id?: number
  local_bangumi_id?: number
  other_id?: number
  bangumi_id?: number
  recorder?: string
  user_status?: number
  is_delete?: boolean
  date?: string
}

export interface AddRecordParams {
  bangumi_id?: number
  source?: 'bangumi' | 'imdb' | 'custom'
  external_id?: string
  imdb_id?: string
  use_api?: boolean
  other_id?: number
  other_title?: string
  other_description?: string
  other_cover?: string
  other_max_number?: number
  other_status?: number
  user_status?: number
  recorder?: string
}

export interface TokenData {
  api_token: string
}

export interface UserInfo {
  id: number
  uuid: string
  username: string
  nickname: string
  email: string
  avatar: string
  status: number
  reg_time: string
}

export interface LocalSearchItem {
  source: 'bangumi' | 'imdb' | 'custom'
  bangumi_id: string | null
  imdb_id: string | null
  other_id: number | null
  title: string
  cover: string | null
  info: string | null
  type: string | null
}

export interface LocalSearchResult {
  items: LocalSearchItem[]
  total: number
  page: number
  page_size: number
}

export interface PermissionLabel {
  label: string
  value: number
  description: string
}

export interface PermissionLabelsResponse {
  labels: PermissionLabel[]
  all_value: number
}

export interface ApiTokenItem {
  id: number
  name: string
  permissions: number
  is_active: boolean
  last_used_at: string | null
  created_at: string
  updated_at: string
}

export interface CreateTokenData {
  id: number
  name: string
  raw_token: string
  permissions: number
}

// ---- v2 API endpoints ----

export const api = {
  // auth
  login(username: string, password: string) {
    return request<ApiResponse<LoginData>>('/api/v2/auth/login', {
      method: 'POST',
      body: { username, password },
    })
  },

  register(username: string, password: string, registerToken?: string) {
    return request<ApiResponse<RegisterData>>('/api/v2/auth/register', {
      method: 'POST',
      body: { username, password, register_token: registerToken },
    })
  },

  getConfig() {
    return request<ApiResponse<ConfigData>>('/api/v2/auth/config')
  },

  // search
  searchBangumi(title: string, page?: number) {
    const params = new URLSearchParams({ q: title })
    if (page) params.set('page', String(page))
    return request<ApiResponse<BangumiSearchItem[]>>(`/api/v2/search?${params}`)
  },

  searchBangumiById(id: number, force?: boolean) {
    const params = force ? `?force=true` : ''
    return request<ApiResponse<BangumiItem>>(`/api/v2/bangumi/${id}${params}`)
  },

  searchImdb(title: string, page?: number, useApi?: boolean) {
    const params = new URLSearchParams({ q: title })
    if (page) params.set('page', String(page))
    if (useApi !== undefined) params.set('use_api', String(useApi))
    return request<ApiResponse<ImdbSearchItem[]>>(`/api/v2/imdb/search?${params}`)
  },

  searchImdbById(id: string, force?: boolean, useApi?: boolean) {
    const params = new URLSearchParams()
    if (force) params.set('force', 'true')
    if (useApi !== undefined) params.set('use_api', String(useApi))
    const query = params.toString()
    return request<ApiResponse<ImdbItem>>(`/api/v2/imdb/${encodeURIComponent(id)}${query ? `?${query}` : ''}`)
  },

  getOtherById(id: number) {
    return request<ApiResponse<OtherItem>>(`/api/v2/other/${id}`)
  },

  searchLocal(keyword?: string, id?: number, page?: number, pageSize?: number) {
    const params = new URLSearchParams()
    if (keyword) params.set('q', keyword)
    if (id) params.set('id', String(id))
    params.set('page', String(page || 1))
    params.set('page_size', String(pageSize || 20))
    return request<ApiResponse<LocalSearchResult>>(`/api/v2/search/local?${params}`)
  },

  // records (RESTful resources)
  getDetailList() {
    return request<ApiResponse<DetailListItem[]>>('/api/v2/records/detail')
  },

  addRecord(params: AddRecordParams) {
    return request<ApiResponse<AddRecordData>>('/api/v2/records', {
      method: 'POST',
      body: params,
    })
  },

  getRecordByBangumi(id: number) {
    return request<ApiResponse<GetRecordData>>(`/api/v2/records/bangumi/${id}`)
  },

  getRecordByCustom(id: number) {
    return request<ApiResponse<GetRecordData>>(`/api/v2/records/custom/${id}`)
  },

  getRecordByImdb(id: string) {
    return request<ApiResponse<GetRecordData>>(`/api/v2/records/imdb/${encodeURIComponent(id)}`)
  },

  updateRecord(bangumi_id: number, recorder?: string, user_status?: number) {
    return request<ApiResponse<null>>(`/api/v2/records/bangumi/${bangumi_id}`, {
      method: 'PATCH',
      body: { recorder, user_status },
    })
  },

  updateRecordByImdb(imdbId: string, recorder?: string, user_status?: number) {
    return request<ApiResponse<null>>(`/api/v2/records/imdb/${encodeURIComponent(imdbId)}`, {
      method: 'PATCH',
      body: { recorder, user_status },
    })
  },

  updateRecordByCustom(id: number, data: { recorder?: string; user_status?: number; other_title?: string; other_description?: string; other_cover?: string; other_max_number?: number; other_status?: number }) {
    return request<ApiResponse<null>>(`/api/v2/records/custom/${id}`, {
      method: 'PATCH',
      body: data,
    })
  },

  deleteRecordByBangumi(id: number, hardDelete = false) {
    const query = hardDelete ? '?hard_delete=true' : ''
    return request<ApiResponse<null>>(`/api/v2/records/bangumi/${id}${query}`, {
      method: 'DELETE',
    })
  },

  deleteRecordByImdb(id: string, hardDelete = false) {
    const query = hardDelete ? '?hard_delete=true' : ''
    return request<ApiResponse<null>>(`/api/v2/records/imdb/${encodeURIComponent(id)}${query}`, {
      method: 'DELETE',
    })
  },

  deleteRecordByCustom(id: number, hardDelete = false) {
    const query = hardDelete ? '?hard_delete=true' : ''
    return request<ApiResponse<null>>(`/api/v2/records/custom/${id}${query}`, {
      method: 'DELETE',
    })
  },

  deleteRecordById(id: number) {
    return request<ApiResponse<null>>(`/api/v2/records/recording/${id}`, {
      method: 'DELETE',
    })
  },

  // user (RESTful /me resource)
  regenerateToken() {
    return request<ApiResponse<TokenData>>('/api/v2/me/token', {
      method: 'POST',
    })
  },

  getUserInfo() {
    return request<ApiResponse<UserInfo>>('/api/v2/me')
  },

  updateUserInfo(nickname?: string, avatar?: string) {
    return request<ApiResponse<null>>('/api/v2/me', {
      method: 'PATCH',
      body: { nickname, avatar },
    })
  },

  updatePassword(oldPassword: string, newPassword: string) {
    return request<ApiResponse<null>>('/api/v2/me/password', {
      method: 'PUT',
      body: { old_password: oldPassword, new_password: newPassword },
    })
  },

  // API Token management (multi-token)
  listTokens() {
    return request<ApiResponse<ApiTokenItem[]>>('/api/v2/tokens')
  },

  createToken(name: string, permissions: number) {
    return request<ApiResponse<CreateTokenData>>('/api/v2/tokens', {
      method: 'POST',
      body: { name, permissions },
    })
  },

  updateToken(id: number, data: { name?: string; permissions?: number; is_active?: boolean }) {
    return request<ApiResponse<null>>(`/api/v2/tokens/${id}`, {
      method: 'PUT',
      body: data,
    })
  },

  deleteToken(id: number) {
    return request<ApiResponse<null>>(`/api/v2/tokens/${id}`, {
      method: 'DELETE',
    })
  },

  getPermissionLabels() {
    return request<ApiResponse<PermissionLabelsResponse>>('/api/v2/tokens/permissions')
  },

  // Episode metadata
  getBangumiEpisodes(id: number) {
    return request<ApiResponse<BangumiEpisodeMeta[]>>(`/api/v2/bangumi/${id}/episodes`)
  },

  // Per-user episode tracking (JWT)
  listEpisodes(bangumiId: number, force = false) {
    const params = force ? `?force=true` : ''
    return request<ApiResponse<EpisodeItem[]>>(`/api/v2/records/bangumi/${bangumiId}/episodes${params}`)
  },

  updateEpisode(bangumiId: number, ordinal: number, data: { watched?: boolean; progress_seconds?: number; duration_seconds?: number }) {
    return request<ApiResponse<EpisodeItem>>(`/api/v2/records/bangumi/${bangumiId}/episodes/${ordinal}`, {
      method: 'PATCH',
      body: data,
    })
  },

  // Sync
  syncRecords(records: SyncRequestRecord[]) {
    return request<ApiResponse<SyncResponseData>>('/api/v2/sync', {
      method: 'POST',
      body: { records },
    })
  },

  incrementalSync(since: string) {
    return request<ApiResponse<SyncResponseRecord[]>>(`/api/v2/sync/incremental?since=${encodeURIComponent(since)}`)
  },
}
