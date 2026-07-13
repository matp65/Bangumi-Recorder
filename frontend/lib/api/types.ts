export interface ApiResponse<T> {
  status: number;
  data?: T;
  message?: string;
}
export interface LoginData {
  token: string;
}
export interface RegisterData {
  token: string;
  api_token: string;
}
export interface ConfigData {
  allow_register: boolean;
  register_need_token: boolean;
}
export interface BangumiSearchItem {
  source?: "bangumi";
  bangumi_id: string;
  title: string;
  alias: string;
  cover: string;
  info: string;
  type: number;
}
export interface ImdbSearchItem {
  source: "imdb";
  imdb_id: string;
  external_id: string;
  title: string;
  year: string | null;
  cover: string | null;
  info: string;
  type: number;
}
export interface BangumiItem {
  source?: "bangumi";
  bangumi_id: string;
  title: string;
  cover_url: string;
  type: number;
  author: string;
  release_date: string | null;
  episodes: number;
  description: string;
}
export interface ImdbItem {
  source: "imdb";
  imdb_id: string;
  external_id: string;
  title: string;
  cover_url: string;
  type: number;
  author: string;
  release_date: string | null;
  episodes: number;
  description: string;
}
export interface OtherItem {
  source: "custom";
  other_id: number;
  title: string;
  cover_url: string;
  type: number;
  author: string;
  release_date: string | null;
  episodes: number;
  description: string;
  status?: number | null;
}
export interface DetailListItem {
  id: number;
  source: string | null;
  external_id: string | null;
  local_external_media_id: number | null;
  local_bangumi_id: number | null;
  other_id: number | null;
  bangumi_id: string | null;
  imdb_id: string | null;
  title: string | null;
  type: number | null;
  author: string | null;
  episodes: number;
  cover_url: string | null;
  recorder: string | null;
  user_status?: number;
  is_delete: boolean;
  updated_at: string;
  created_at: string;
}
export interface EpisodeItem {
  ordinal: number;
  label: string | null;
  title: string | null;
  name_cn: string | null;
  airdate: string | null;
  duration: string | null;
  watched: boolean;
  progress_seconds: number | null;
  duration_seconds: number | null;
  completed_at: string | null;
  updated_at: string | null;
}
export interface AddRecordData {
  source?: string;
  external_id?: string;
  imdb_id?: string;
  local_external_media_id?: number;
  local_bangumi_id?: number;
  other_id?: number;
  bangumi_id?: number;
  recorder?: string;
  date?: string;
}
export interface GetRecordData {
  source?: string;
  external_id?: string;
  imdb_id?: string;
  local_external_media_id?: number;
  local_bangumi_id?: number;
  other_id?: number;
  bangumi_id?: number;
  recorder?: string;
  user_status?: number;
  is_delete?: boolean;
  date?: string;
}
export interface AddRecordParams {
  bangumi_id?: number;
  source?: "bangumi" | "imdb" | "custom";
  external_id?: string;
  imdb_id?: string;
  use_api?: boolean;
  other_id?: number;
  other_title?: string;
  other_description?: string;
  other_cover?: string;
  other_max_number?: number;
  other_status?: number;
  user_status?: number;
  recorder?: string;
}
export interface UserInfo {
  id: number;
  uuid: string;
  username: string;
  nickname: string;
  email: string;
  avatar: string;
  status: number;
  is_admin: boolean;
  reg_time: string;
}
export interface LocalSearchItem {
  source: "bangumi" | "imdb" | "custom";
  bangumi_id: string | null;
  imdb_id: string | null;
  other_id: number | null;
  title: string;
  cover: string | null;
  info: string | null;
  type: string | null;
}
export interface LocalSearchResult {
  items: LocalSearchItem[];
  total: number;
  page: number;
  page_size: number;
}
export interface PermissionLabel {
  label: string;
  value: number;
  description: string;
}
export interface PermissionLabelsResponse {
  labels: PermissionLabel[];
  all_value: number;
}
export interface ApiTokenItem {
  id: number;
  name: string;
  permissions: number;
  is_active: boolean;
  last_used_at: string | null;
  created_at: string;
  updated_at: string;
}
export interface CreateTokenData {
  id: number;
  name: string;
  raw_token: string;
  permissions: number;
}
export interface RecordingLogItem {
  id: number;
  recording_id: number | null;
  user_id: number | null;
  target_type: string;
  target_id: number | null;
  target_title: string | null;
  action: string;
  field_name: string | null;
  old_value: unknown;
  new_value: unknown;
  metadata: unknown;
  created_at: string;
}
export interface SystemLogItem {
  id: number;
  level: string;
  category: string;
  action: string;
  message: string;
  user_id: number | null;
  username: string | null;
  metadata: unknown;
  created_at: string;
}
export interface LogListData<T> {
  items: T[];
  page: number;
  page_size: number;
}
export interface RecordingLogFilters {
  start_time?: string;
  end_time?: string;
  target?: string;
  action?: string;
}
export interface SystemLogFilters {
  start_time?: string;
  end_time?: string;
  category?: string;
  action?: string;
  username?: string;
}
export interface AutoCleanupSetting {
  enabled: boolean;
}
export interface SyncRequestRecord {
  bangumi_id: string;
  recorder?: string | null;
  user_status?: number | null;
  updated_at?: string | null;
}
export interface SyncResponseRecord {
  bangumi_id: string;
  recorder: string | null;
  user_status: number | null;
  updated_at: string;
}
export interface SyncResponseData {
  records: SyncResponseRecord[];
  deleted: string[];
}
