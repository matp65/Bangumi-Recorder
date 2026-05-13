# Bangumi Recorder

自托管的番剧追番记录器，基于 [Bangumi (bgm.tv)](https://bgm.tv) 数据，支持搜索、添加番剧、记录观看进度。

## 功能

- **番剧搜索** — 按标题或 Bangumi ID 搜索番剧 / 漫画 / 游戏条目
- **追番管理** — 添加条目到个人追番列表，支持多种观看状态
- **进度记录** — 以 `集数|分钟:秒` 格式精确记录观看进度（如 `5|2:12`）
- **JWT 认证** — 用户注册 / 登录，Bearer Token 鉴权
- **API Token** — 支持 API Token 鉴权，方便外部工具集成（`/api/v1/open/*` 路由）
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
```

### 2. 数据库迁移

```shell
sqlx migrate run
```

### 3. 构建前端

```shell
cd frontend
npm install
npm run build
```

### 4. 构建并运行后端

```shell
# 开发模式
cargo run

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

### 受保护接口（需 Bearer Token）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/search/bangumi` | 按标题搜索，Body: `{"title":"Re0"}` |
| POST | `/api/v1/search/bangumi/id` | 按 ID 搜索，Body: `{"id":425998}` |
| POST | `/api/v1/record/add` | 添加追番，Body: `{"bangumi_id":425998,"user_status":1}` |
| POST | `/api/v1/record/update` | 更新进度，Body: `{"bangumi_id":425998,"recorder":"5|2:12"}` |
| POST | `/api/v1/record/get` | 获取进度，Body: `{"bangumi_id":425998}` |
| GET | `/api/v1/record/list` | 列出所有追番（基本信息） |
| GET | `/api/v1/record/detail_list` | 列出所有追番（含详情） |

### 开放接口（API Token 鉴权）

以上 record 接口对应 `/api/v1/open/*` 路由，使用 `?token=你的API-Token` 查询参数鉴权。

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
│       └── open/            # API Token 鉴权的开放接口
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
