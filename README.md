# Bangumi Recorder

自托管的番剧追番记录器，基于 [Bangumi (bgm.tv)](https://bgm.tv) 数据，支持搜索、添加番剧、记录观看进度。

## 功能

- **番剧搜索** — 按标题或 Bangumi ID 搜索番剧 / 漫画 / 游戏条目
- **追番管理** — 添加条目到个人追番列表，支持多种观看状态
- **进度记录** — 以 `集数|分钟:秒` 格式精确记录观看进度（如 `5|2:12`）
- **JWT 认证** — 用户注册 / 登录，Bearer Token 鉴权
- **API Token** — 支持 API Token 鉴权，方便外部工具集成（`/api/v2/open/*` 路由，`?token=xxx`）
- **v2 API** — 统一的 `{status, data, message}` 响应格式 + HTTP 状态码，RESTful 路径设计
- **自托管** — 数据完全由自己掌控，MySQL 存储

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust (Edition 2024), Axum 0.7, Tokio, SQLx (MySQL) |
| 前端 | Vue 3, TypeScript, Arco Design Vue, Vite, Pinia |
| 数据库 | MySQL |
| 数据来源 | 抓取 [bgm.tv](https://bgm.tv) |

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
| POST | `/api/v2/me/token` | 重新生成 API Token → `{ data: { api_token } }` |

#### 搜索（Bearer Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v2/search?q=Re0&page=1` | 在线搜索 Bangumi（搜索页轻量，详情页 24h 缓存） |
| GET | `/api/v2/bangumi/:id?force=true` | 按 Bangumi ID 获取详情（24h 本地缓存，`?force=true` 跳过） |
| GET | `/api/v2/search/local?q=...&page=1&page_size=20` | 搜索本地缓存 |

#### 追番记录（Bearer Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v2/records` | 列出所有追番（基本信息） |
| GET | `/api/v2/records/detail` | 列出所有追番（含详情） |
| POST | `/api/v2/records` | 添加追番 `{"bangumi_id","other_id","recorder",...}` |
| GET | `/api/v2/records/bangumi/:id` | 按 Bangumi ID 获取进度 |
| PATCH | `/api/v2/records/bangumi/:id` | 按 Bangumi ID 更新进度 `{"recorder","user_status"}` |
| DELETE | `/api/v2/records/bangumi/:id` | 按 Bangumi ID 删除记录 |
| GET | `/api/v2/records/custom/:id` | 按自定义条目 ID 获取进度 |
| DELETE | `/api/v2/records/custom/:id` | 按自定义条目 ID 删除 |
| DELETE | `/api/v2/records/recording/:id` | 按记录 ID 删除 |

#### 开放接口（API Token 鉴权，`?token=xxx`）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v2/open/me?token=` | 用户信息 |
| GET | `/api/v2/open/records?token=` | 列出追番 |
| GET | `/api/v2/open/records/detail?token=` | 详细列表 |
| POST | `/api/v2/open/records?token=&bangumi_id=` | 添加追番 |
| GET | `/api/v2/open/records/bangumi/:id?token=` | 获取进度 |
| PATCH | `/api/v2/open/records/bangumi/:id?token=&recorder=` | 更新进度 |
| DELETE | `/api/v2/open/records/bangumi/:id?token=` | 删除记录 |
| GET | `/api/v2/open/records/custom/:id?token=` | 获取自定义进度 |
| DELETE | `/api/v2/open/records/custom/:id?token=` | 删除自定义记录 |
| GET | `/api/v2/open/search?q=Re0&token=` | 在线搜索 |
| GET | `/api/v2/open/bangumi/:id?force=true&token=` | 获取详情（24h 缓存） |
| GET | `/api/v2/open/search/local?q=...&token=` | 本地搜索 |

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
│       ├── new.rs           # 添加追番
│       ├── list.rs          # 追番列表
│       ├── get_recorder.rs  # 获取进度
│       ├── update_recorder.rs # 更新进度
│       ├── detail_list.rs   # 详细列表
│       ├── open/            # API Token 鉴权的开放接口（委托至 regular handler）
│       └── v2/              # v2 API（RESTful + 统一 {status,data,message} 格式）
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
