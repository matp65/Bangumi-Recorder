# Bangumi Recorder

自托管的媒体追踪记录器，基于 [Bangumi (bgm.tv)](https://bgm.tv) 和 IMDb 数据，支持搜索、添加条目、记录观看/阅读/游玩进度。

## 功能

- **Bangumi 搜索** — 按标题或 Bangumi ID 搜索动画 / 书籍 / 游戏 / 三次元等条目
- **IMDb 搜索** — 支持免 API 的 IMDb suggestion 检索，也可配置 OMDb API Key 使用 API 检索
- **追踪管理** — 添加 Bangumi、IMDb 或自定义条目到个人追踪列表，支持多种状态、软删除和硬删除
- **进度记录** — 以 `编号|分钟:秒` 格式记录进度（如 `5|2:12`），时间可为空（如 `5`）
- **JWT 认证** — 用户注册 / 登录，Bearer Token 鉴权
- **API Token 管理** — 支持多 Token、精细权限控制（只读/读写/查看信息/修改信息/添加记录/删除记录/修改记录/更改状态/全部允许），每个 Token 可独立设置权限组合
- **v2 API** — 统一的 `{status, data, message}` 响应格式 + HTTP 状态码，RESTful 路径设计
- **自托管** — 数据完全由自己掌控，MySQL 存储

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust (Edition 2024), Axum 0.7, Tokio, SQLx (MySQL) |
| 前端 | Vue 3, TypeScript, Arco Design Vue, Vite, Pinia |
| 数据库 | MySQL |
| 数据来源 | 抓取 [bgm.tv](https://bgm.tv)、IMDb suggestion / OMDb API |

## 前置条件

- [Rust](https://rustup.rs/) (Edition 2024)
- [Node.js](https://nodejs.org/) >= 18
- MySQL 数据库
- [sqlx-cli](https://crates.io/crates/sqlx-cli)（用于数据库迁移）

## 快速开始

### 1. 配置环境变量

```shell
cp .env.example .env
```

编辑 `.env`：

```env
LISTEN=127.0.0.1
LISTEN_PORT=8080
RUST_LOG=info
DATABASE_URL=mysql://用户名:密码@localhost/数据库名
JWT_SECRET=你的随机密钥

# 可选：IMDb API 检索。未配置时使用免 API 的 IMDb suggestion/页面抓取。
# OMDB_API_KEY=你的 OMDb API Key
# IMDB_SEARCH_MODE=no_api

# 可选：指定外部前端地址（开发时使用，会跳过嵌入的静态文件）
# --frontend-url http://localhost:5173
```

### 2. 数据库迁移

```shell
sqlx migrate run
```

### 3. 构建并运行后端

前端会在 `cargo build`/`cargo run` 时自动构建（需要 Node.js）。也可手动构建：

```shell
cd frontend && npm install && npm run build
```

```shell
# 开发模式（使用内置前端）
cargo run

# 开发模式（使用 Vite 开发服务器，请求代理到外部前端）
cargo run -- -f http://localhost:5173

# 开发模式（使用本地构建的前端目录）
cargo run -- -f ./frontend/dist

# 生产模式
cargo build --release
```

### 4. 构建并运行后端

```shell
# 开发模式（使用内置前端）
cargo run

# 开发模式（使用外部前端，如 Vite 开发服务器）
cargo run -- --frontend-url http://localhost:5173

# 生产模式
cargo build --release
./target/release/Bangumi-Recorder
```

服务启动后访问 `http://127.0.0.1:8080`。

## API 接口

### 认证

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/auth/register` | 注册，Body: `{"username":"...", "password":"..."}` |
| POST | `/auth/login` | 登录，返回 JWT Token |

## API

### v2（推荐）

所有 v2 接口统一返回格式：

```json
{ "status": 0, "data": ..., "message": "" }
```

- `status`: `0` 成功 / `-1` 报错
- `data`: 所有响应数据
- `message`: 错误描述（成功时为空）

错误时同时返回适当的 HTTP 状态码（400/404/409/401/500 等）。

#### 认证（无需 Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v2/auth/login` | `{"username","password"}` → `{ data: { token } }` |
| POST | `/api/v2/auth/register` | `{"username","password","register_token?"}` → `{ data: { token, api_token } }` |
| GET | `/api/v2/auth/config` | → `{ data: { allow_register, register_need_token } }` |

#### 用户（Bearer Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v2/me` | 获取用户信息 |
| PATCH | `/api/v2/me` | 更新资料 `{"nickname","avatar"}` |
| PUT | `/api/v2/me/password` | 修改密码 `{"old_password","new_password"}` |
| POST | `/api/v2/me/token` | 重新生成 API Token（创建新 Token + 全部权限）→ `{ data: { api_token } }` |

#### API Token 管理（Bearer Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v2/tokens` | 列出所有 API Token |
| POST | `/api/v2/tokens` | 创建 Token `{"name","permissions"}` → `{ data: { id, name, raw_token, permissions } }` |
| PUT | `/api/v2/tokens/:id` | 更新 Token `{"name","permissions","is_active"}` |
| DELETE | `/api/v2/tokens/:id` | 删除/吊销 Token |
| GET | `/api/v2/tokens/permissions` | 获取权限标签列表（用于前端展示） |

权限值（bitmask，可按位组合）：

| 权限 | 值 | 说明 |
|------|-----|------|
| Read-only | `1<<0` (1) | 查看记录列表和详情 |
| Read-Write | `1<<1` (2) | 添加、修改、删除记录及更改状态 |
| View Personal Info | `1<<2` (4) | 查看用户昵称、头像等信息 |
| Modify Personal Info | `1<<3` (8) | 修改用户昵称、头像等信息 |
| Add Record | `1<<4` (16) | 添加新的追番记录 |
| Delete Record | `1<<5` (32) | 删除追番记录 |
| Modify Record | `1<<6` (64) | 修改追番进度 |
| Change Status | `1<<7` (128) | 更改追番状态 |
| Allow All | `u64::MAX` | 允许所有操作 |

> 例：`permissions = 1 | 4 | 16 = 21` 表示只读 + 查看个人信息 + 添加记录

#### 搜索（Bearer Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v2/search?q=Re0&page=1` | 在线搜索 Bangumi（搜索页轻量，详情页 24h 缓存） |
| GET | `/api/v2/bangumi/:id?force=true` | 按 Bangumi ID 获取详情（24h 本地缓存，`?force=true` 跳过） |
| GET | `/api/v2/imdb/search?q=Interstellar&page=1&use_api=false` | 在线搜索 IMDb，支持免 API / OMDb API |
| GET | `/api/v2/imdb/:id?force=true&use_api=false` | 按 IMDb `tt...` ID 获取详情（24h 本地缓存） |
| GET | `/api/v2/search/local?q=...&page=1&page_size=20` | 搜索本地缓存 |

#### 追番记录（Bearer Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v2/records` | 列出所有追番（基本信息） |
| GET | `/api/v2/records/detail` | 列出所有追番（含详情） |
| POST | `/api/v2/records` | 添加追踪 `{"bangumi_id"}` / `{"source":"imdb","external_id":"tt..."}` / `{"other_id"}` |
| GET | `/api/v2/records/bangumi/:id` | 按 Bangumi ID 获取进度 |
| PATCH | `/api/v2/records/bangumi/:id` | 按 Bangumi ID 更新进度 `{"recorder","user_status"}` |
| DELETE | `/api/v2/records/bangumi/:id?hard_delete=false` | 按 Bangumi ID 删除记录，默认软删除，`hard_delete=true` 为硬删除 |
| GET | `/api/v2/records/imdb/:id` | 按 IMDb ID 获取进度 |
| PATCH | `/api/v2/records/imdb/:id` | 按 IMDb ID 更新进度 `{"recorder","user_status"}` |
| DELETE | `/api/v2/records/imdb/:id?hard_delete=false` | 按 IMDb ID 删除记录，默认软删除，`hard_delete=true` 为硬删除 |
| GET | `/api/v2/records/custom/:id` | 按自定义条目 ID 获取进度 |
| DELETE | `/api/v2/records/custom/:id?hard_delete=false` | 按自定义条目 ID 删除，默认软删除，`hard_delete=true` 为硬删除 |
| DELETE | `/api/v2/records/recording/:id` | 按记录 ID 删除 |

#### 单集追踪（Bearer Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v2/bangumi/:id/episodes` | 获取剧集元数据列表（爬取 bgm.tv 并缓存，24h TTL，`?force=true` 强制刷新） |
| GET | `/api/v2/records/bangumi/:id/episodes` | 获取单集追踪状态（含元数据合并，`?force=true` 强制刷新缓存） |
| PATCH | `/api/v2/records/bangumi/:id/episodes/:ordinal` | 更新单集进度 `{"watched","progress_seconds","duration_seconds"}` |

更新单集时会自动同步主表 `recorder` 字段：集数 = max(ordinal where watched=1)，时间 = max(progress_seconds) 格式化为 mm:ss。

#### 同步

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v2/sync` | 批量同步（一轮往返，更新仲裁按较新时间戳胜出） |
| GET | `/api/v2/sync/incremental?since=` | 增量同步（返回指定时间戳后变更的记录） |

#### 开放接口（API Token 鉴权，`?token=xxx`）

每个 Token 拥有独立的权限组合，不满足权限时返回 `403 Forbidden`。

| 方法 | 路径 | 所需权限 | 说明 |
|------|------|---------|------|
| GET | `/api/v2/open/me?token=` | View Personal Info | 用户信息 |
| GET | `/api/v2/open/records?token=` | Read-only / Read-Write | 列出追番 |
| GET | `/api/v2/open/records/detail?token=` | Read-only / Read-Write | 详细列表 |
| POST | `/api/v2/open/records?token=&bangumi_id=` | Add Record / Read-Write | 添加追番 |
| GET | `/api/v2/open/records/bangumi/:id?token=` | Read-only / Read-Write | 获取进度 |
| PATCH | `/api/v2/open/records/bangumi/:id?token=&recorder=` | Modify Record / Change Status / Read-Write | 更新进度 |
| DELETE | `/api/v2/open/records/bangumi/:id?token=&hard_delete=false` | Delete Record / Read-Write | 删除记录，支持硬删除 |
| GET | `/api/v2/open/records/custom/:id?token=` | Read-only / Read-Write | 获取自定义进度 |
| DELETE | `/api/v2/open/records/custom/:id?token=` | Delete Record / Read-Write | 删除自定义记录 |
| GET | `/api/v2/open/search?q=Re0&token=` | Read-only / Read-Write | 在线搜索 |
| GET | `/api/v2/open/bangumi/:id?force=true&token=` | Read-only / Read-Write | 获取详情（24h 缓存） |
| GET | `/api/v2/open/imdb/search?q=Interstellar&use_api=false&token=` | Read-only / Read-Write | IMDb 搜索 |
| GET | `/api/v2/open/imdb/:id?force=true&use_api=false&token=` | Read-only / Read-Write | IMDb 详情 |
| GET | `/api/v2/open/records/imdb/:id?token=` | Read-only / Read-Write | 获取 IMDb 进度 |
| PATCH | `/api/v2/open/records/imdb/:id?token=` | Modify Record / Change Status / Read-Write | 更新 IMDb 进度 |
| DELETE | `/api/v2/open/records/imdb/:id?token=&hard_delete=false` | Delete Record / Read-Write | 删除 IMDb 记录，支持硬删除 |
| GET | `/api/v2/open/bangumi/:id/episodes?force=true&token=` | Read-only / Read-Write | 剧集元数据 |
| GET | `/api/v2/open/episodes/:bangumi_id?force=true&token=` | Read-only / Read-Write | 单集追踪列表 |
| PATCH | `/api/v2/open/episodes/:bangumi_id/:ordinal?token=` | Modify Record / Read-Write | 更新单集进度 |
| POST | `/api/v2/open/sync?token=` | Read-Write | 批量同步 |
| GET | `/api/v2/open/sync/incremental?since=&token=` | Read-only / Read-Write | 增量同步 |
| GET | `/api/v2/open/search/local?q=...&token=` | Read-only / Read-Write | 本地搜索 |

### v1（向下兼容）

v1 接口保持 `/api/v1/*` 和 `/api/v1/open/*` 路径不变，响应格式为旧版自定义结构体，仅用于兼容已有客户端。新项目请使用 v2。

完整 API 文档见 [OpenAPI.yaml](./OpenAPI.yaml)。

## 项目结构

```
├── src/                     # Rust 后端
│   ├── main.rs              # 入口、路由注册
│   ├── auth_bearer.rs       # JWT 认证、登录/注册
│   └── api/                 # API 路由处理
│       ├── search.rs        # Bangumi 搜索（网页抓取）
│       ├── imdb.rs          # IMDb 搜索/详情缓存（免 API / OMDb API）
│       ├── new.rs           # 添加追番
│       ├── list.rs          # 追番列表
│       ├── get_recorder.rs  # 获取进度
│       ├── update_recorder.rs # 更新进度
│       ├── detail_list.rs   # 详细列表
│       ├── api_token.rs     # Token 鉴权 & 权限系统
│       ├── open/            # API Token 鉴权的开放接口（委托至 regular handler）
│       └── v2/              # v2 API（RESTful + 统一 {status,data,message} 格式）
│           └── token.rs     # API Token CRUD 管理
├── frontend/                # Vue 3 前端
│   └── src/
│       ├── views/           # 页面组件
│       │   ├── Dashboard.vue  # 追番面板
│       │   ├── Search.vue     # 搜索页面
│       │   ├── Detail.vue     # 详情页面
│       │   └── Login.vue      # 登录/注册
│       ├── stores/          # Pinia 状态管理
│       ├── router/          # Vue Router 配置
│       └── api/             # API 请求封装
├── migrations/              # 数据库迁移文件
└── OpenAPI.yaml             # API 规范文档
```

## 许可证

MIT
