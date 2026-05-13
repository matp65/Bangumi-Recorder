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
  local_bangumi_id: number
  bangumi_id: string | null
  title: string | null
  type: number | null
  author: string | null
  episodes: number | null
  cover_url: string | null
  recorder: string | null
  user_status?: number
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
  bangumi_id?: number
  date?: string
}

export interface GetRecorderResponse {
  status: number
  local_bangumi_id?: number
  bangumi_id?: number
  recorder?: string
  user_status?: number
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

export interface TokenRegenerateResponse {
  status: number
  api_token?: string
  message?: string
}

export const api = {
  login(username: string, password: string) {
    return request<LoginResponse>('/auth/login', {
      method: 'POST',
      body: { username, password },
    })
  },

  register(username: string, password: string) {
    return request<RegisterResponse>('/auth/register', {
      method: 'POST',
      body: { username, password },
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

  getDetailList() {
    return request<DetailListResponse>('/api/v1/record/detail_list')
  },

  addRecord(bangumi_id: number, user_status: number) {
    return request<AddRecordResponse>('/api/v1/record/add', {
      method: 'POST',
      body: { bangumi_id, user_status },
    })
  },

  getRecord(bangumi_id: number) {
    return request<GetRecorderResponse>('/api/v1/record/get', {
      method: 'POST',
      body: { bangumi_id },
    })
  },

  updateRecord(bangumi_id: number, recorder?: string, user_status?: number) {
    return request<UpdateRecorderResponse>('/api/v1/record/update', {
      method: 'POST',
      body: { bangumi_id, recorder, user_status },
    })
  },

  deleteRecord(bangumi_id: number) {
    return request<DeleteRecorderResponse>('/api/v1/record/delete', {
      method: 'POST',
      body: { bangumi_id },
    })
  },

  regenerateToken() {
    return request<TokenRegenerateResponse>('/api/v1/auth/token/regenerate', {
      method: 'POST',
    })
  },
}
