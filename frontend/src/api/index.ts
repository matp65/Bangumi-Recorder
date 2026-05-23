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

export interface LoginResponse {
  status: number
  token?: string
  message?: string
}

export interface RegisterResponse {
  status: number
  token?: string
  api_token?: string
  message?: string
}

export interface ConfigResponse {
  allow_register: boolean
  register_need_token: boolean
}

export interface BangumiSearchItem {
  bangumi_id: string
  title: string
  alias: string
  cover: string
  info: string
  type: number
}

export interface SearchBangumiResponse {
  status: number
  data?: BangumiSearchItem[]
}

export interface BangumiItem {
  bangumi_id: string
  title: string
  cover_url: string
  type: number
  author: string
  release_date: string | null
  episodes: number
  description: string
}

export interface IDSearchResponse {
  status: number
  data?: BangumiItem
}

export interface DetailListItem {
  id: number
  local_bangumi_id: number | null
  other_id: number | null
  local_other_id: number | null
  bangumi_id: string | null
  title: string | null
  type: number | null
  author: string | null
  episodes: number | null
  cover_url: string | null
  recorder: string | null
  user_status?: number
  is_delete: boolean
  updated_at: string
  created_at: string
}

export interface DetailListResponse {
  status: number
  data?: DetailListItem[]
}

export interface AddRecordResponse {
  status: number
  local_bangumi_id?: number
  other_id?: number
  local_other_id?: number
  bangumi_id?: number
  recorder?: string
  date?: string
}

export interface GetRecorderResponse {
  status: number
  local_bangumi_id?: number
  other_id?: number
  local_other_id?: number
  bangumi_id?: number
  recorder?: string
  user_status?: number
  is_delete?: boolean
  date?: string
}

export interface UpdateRecorderResponse {
  status: number
  message?: string
}

export interface DeleteRecorderResponse {
  status: number
  message?: string
}

export interface AddRecordParams {
  bangumi_id?: number
  other_id?: number
  other_title?: string
  other_description?: string
  other_cover?: string
  other_max_number?: number
  other_status?: number
  user_status?: number
  recorder?: string
}

export interface GetRecordParams {
  bangumi_id?: number
  local_bangumi_id?: number
  other_id?: number
  local_other_id?: number
}

export interface DeleteRecordParams {
  bangumi_id?: number
  other_id?: number
  local_other_id?: number
}

export interface TokenRegenerateResponse {
  status: number
  api_token?: string
  message?: string
}

export interface UserInfo {
  id: number
  username: string
  nickname: string
  email: string
  avatar: string
  status: number
  reg_time: string
}

export interface UpdateUserInfo {
  nickname?: string
  avatar?: string
}

export interface UpdatePasswordRequest {
  old_password?: string
  new_password?: string
}

export interface UserResponse {
  status: number
  message?: string
}

export interface LocalSearchItem {
  bangumi_id: string | null
  other_id: number | null
  title: string
  cover: string | null
  info: string | null
  type: string | null
}

export interface LocalSearchResponse {
  status: number
  data?: LocalSearchItem[]
  total?: number
  page?: number
  page_size?: number
}

export const api = {
  login(username: string, password: string) {
    return request<LoginResponse>('/auth/login', {
      method: 'POST',
      body: { username, password },
    })
  },

  register(username: string, password: string, registerToken?: string) {
    return request<RegisterResponse>('/auth/register', {
      method: 'POST',
      body: { username, password, register_token: registerToken },
    })
  },

  getConfig() {
    return request<ConfigResponse>('/auth/config')
  },

  searchBangumi(title: string, page?: number) {
    return request<SearchBangumiResponse>('/api/v1/search/bangumi', {
      method: 'POST',
      body: { title, page },
    })
  },

  searchBangumiById(id: number) {
    return request<IDSearchResponse>('/api/v1/search/bangumi/id', {
      method: 'POST',
      body: { id },
    })
  },

  searchLocal(keyword?: string, id?: number, page?: number, pageSize?: number) {
    return request<LocalSearchResponse>('/api/v1/search/local', {
      method: 'POST',
      body: { keyword, id, page, page_size: pageSize },
    })
  },

  getDetailList() {
    return request<DetailListResponse>('/api/v1/record/detail_list')
  },

  addRecord(params: AddRecordParams) {
    return request<AddRecordResponse>('/api/v1/record/add', {
      method: 'POST',
      body: params,
    })
  },

  getRecord(params: GetRecordParams) {
    return request<GetRecorderResponse>('/api/v1/record/get', {
      method: 'POST',
      body: params,
    })
  },

  updateRecord(bangumi_id: number, recorder?: string, user_status?: number) {
    return request<UpdateRecorderResponse>('/api/v1/record/update', {
      method: 'POST',
      body: { bangumi_id, recorder, user_status },
    })
  },

  deleteRecord(params: DeleteRecordParams) {
    return request<DeleteRecorderResponse>('/api/v1/record/delete', {
      method: 'POST',
      body: params,
    })
  },

  regenerateToken() {
    return request<TokenRegenerateResponse>('/api/v1/auth/token/regenerate', {
      method: 'POST',
    })
  },

  getUserInfo() {
    return request<UserInfo>('/api/v1/user/info')
  },

  updateUserInfo(nickname?: string, avatar?: string) {
    return request<UserResponse>('/api/v1/user/update', {
      method: 'POST',
      body: { nickname, avatar },
    })
  },

  updatePassword(oldPassword: string, newPassword: string) {
    return request<UserResponse>('/api/v1/user/password', {
      method: 'POST',
      body: { old_password: oldPassword, new_password: newPassword },
    })
  },
}
