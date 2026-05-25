# Bangumi Recorder API 文档

> 适用于第三方平台/自动化工具对接

## 目录

- [概述](#概述)
- [通用约定](#通用约定)
- [状态码说明](#状态码说明)
- [鉴权方式](#鉴权方式)
  - [方式一：JWT Bearer Token（Web 前端用）](#方式一jwt-bearer-tokenweb-前端用)
  - [方式二：API Token（推荐第三方对接用）](#方式二api-token推荐第三方对接用)
- [API 接口](#api-接口)
  - [鉴权相关](#鉴权相关)
  - [番组搜索](#番组搜索)
  - [用户信息（JWT 鉴权）](#用户信息jwt-鉴权)
  - [追番记录（JWT 鉴权）](#追番记录jwt-鉴权)
  - [追番记录（API Token 鉴权）](#追番记录api-token-鉴权)
  - [用户信息（API Token 鉴权）](#用户信息api-token-鉴权)
- [数据字典](#数据字典)
- [对接示例](#对接示例)

---

## 概述

Bangumi Recorder 是一个自托管的追番进度记录服务。通过 API 可以搜索番组信息、添加/更新/查询/删除追番记录。

- **Base URL**: `http://<host>:<port>`（默认 `http://127.0.0.1:8080`）
- **数据格式**: 响应均为 `application/json`；JWT 接口使用 POST/JSON Body 传参，Open API 接口统一使用 URL Query 参数传参

---

## 通用约定

| 项目 | 说明 |
|------|------|
| 编码 | UTF-8 |
| 请求格式 | JWT 接口: `Content-Type: application/json`（POST body）；Open API 接口: URL Query 参数（无 body） |
| 响应格式 | `Content-Type: application/json` |
| HttpOnly POST | 搜索及记录增删改查均使用 **POST**（GET 仅用于 `list` / `detail_list` / `user/info`） |
| 空字符串字段 | 可能返回空字符串，视为无值处理 |
| Open API 鉴权错误 | Token 缺失或无效时返回 **HTTP 401**（无 JSON body），应用层错误仍返回 JSON 含 status |

---

## 状态码说明

所有接口响应均包含 `status` 字段（`number` 类型）：

| status | 含义 |
|--------|------|
| `0` | 成功 |
| `1` | 参数缺失或无效 |
| `-1` | 服务端错误 |
| `-2` | 资源不存在 |
| `-3` | 记录已存在（add）/ 记录不存在（update） |
| `-4` | 缺少 `Authorization` 请求头 |
| `-5` | JWT Token 无效或已过期 |
| `-6` | API Token 无效 |
| `-10` | 用户已存在（注册） |
| `-11` | 不允许注册 |
| `-12` | 用户名或密码错误（登录） |

---

## 鉴权方式

### 方式一：JWT Bearer Token（Web 前端用）

请求头携带：

```
Authorization: Bearer <jwt_token>
```

Token 通过 `/auth/login` 或 `/auth/register` 获取，有效期 **7 天**。

### 方式二：API Token（推荐第三方对接用）

URL Query 参数携带：

```
?token=<api_token>
```

API Token 在注册时返回一次，也可通过 `/api/v1/auth/token/regenerate` 重新生成（旧 Token 立即失效）。

> **建议**: 对于自动化脚本、浏览器插件、第三方平台等外部对接场景，优先使用 API Token 方式，避免频繁处理 JWT 过期问题。

---

## API 接口

---
### 鉴权相关

#### `GET /auth/config`

查询是否允许注册。

**鉴权**: 无

**请求**: 无参数

**响应**:

```json
{
  "allow_register": true
}
```

---

#### `POST /auth/register`

注册新用户。

**鉴权**: 无（受 `ALLOW_REGISTER` 环境变量控制）

**请求**:

```json
{
  "username": "string",
  "password": "string"
}
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| username | string | 是 | 用户名 |
| password | string | 是 | 密码 |

**响应**:

```json
{
  "status": 0,
  "token": "eyJhbGciOi...",
  "api_token": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Register success"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| token | string | JWT Token（登录态） |
| api_token | string | API Token（**仅返回一次，请妥善保存**） |

> API Token 格式为 UUID v4，存储时哈希处理，丢失后只能通过 regenerate 换新。

---

#### `POST /auth/login`

登录。

**鉴权**: 无

**请求**:

```json
{
  "username": "string",
  "password": "string"
}
```

**响应**:

```json
{
  "status": 0,
  "token": "eyJhbGciOi...",
  "message": "Login success"
}
```

| status | 说明 |
|--------|------|
| 0 | 登录成功 |
| -12 | 用户名或密码错误 |

---

#### `POST /api/v1/auth/token/regenerate`

重新生成 API Token（旧 Token 立即失效）。

**鉴权**: JWT Bearer Token

**请求**: 无参数

**响应**:

```json
{
  "status": 0,
  "api_token": "660e8400-e29b-41d4-a716-446655440001",
  "message": "Token regenerated"
}
```

---

### 番组搜索

> 搜索基于对 bgm.tv 的实时抓取，可能有一定延迟。

#### `POST /api/v1/search/bangumi`

按标题搜索番组。

**鉴权**: JWT Bearer Token

**请求**:

```json
{
  "title": "string",
  "page": 1
}
```

| 参数 | 类型 | 必填 | 默认 | 说明 |
|------|------|------|------|------|
| title | string | 是 | - | 搜索关键词 |
| page | number | 否 | 1 | 页码 |

**响应**:

```json
{
  "status": 0,
  "data": [
    {
      "bangumi_id": 425998,
      "title": "Re:从零开始的异世界生活",
      "alias": "Re:Zero",
      "cover": "https://...",
      "info": "TV动画, 2016年4月...",
      "type": 1
    }
  ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| bangumi_id | number | bgm.tv 条目 ID |
| title | string | 标题 |
| alias | string | 别名 |
| cover | string | 封面图 URL |
| info | string | 简介摘要 |
| type | number | 类型，见[数据字典](#type-番组类型) |

---

#### `POST /api/v1/search/bangumi/id`

按 bgm.tv ID 获取番组详细信息，并入库到本地数据库。

**鉴权**: JWT Bearer Token

**请求**:

```json
{
  "id": 425998
}
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | number | 是 | bgm.tv 条目 ID |

**响应**:

```json
{
  "status": 0,
  "data": {
    "bangumi_id": 425998,
    "title": "Re:从零开始的异世界生活",
    "cover_url": "https://...",
    "type": 1,
    "author": "长月达平",
    "release_date": "2016-04-04",
    "episodes": 25,
    "description": "在从便利商店回家的路上..."
  }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| type | number | 类型，见[数据字典](#type-番组类型) |
| episodes | number \| null | 总集数，可能为 null |
| release_date | string \| null | 上映日期 (YYYY-MM-DD)，可能为 null |

---

### 用户信息（JWT 鉴权）

> 以下接口前缀均为 `/api/v1/user`，鉴权方式为 **JWT Bearer Token**。

#### `GET /api/v1/user/info`

获取当前用户的基本信息。

**鉴权**: JWT Bearer Token

**请求**: 无参数

**响应**:

```json
{
  "id": 1,
  "uuid": "7047965d-036b-4b1d-a877-7f5ced0147e4",
  "username": "user",
  "nickname": "小明",
  "email": "user@example.com",
  "avatar": "",
  "status": 0,
  "reg_time": "2026-05-19"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| id | number | 用户 ID |
| uuid | string | 用户 UUID（v7） |
| username | string | 用户名 |
| nickname | string | 昵称（可编辑） |
| email | string | 邮箱（只读，注册时填入） |
| avatar | string | 头像 URL（可编辑） |
| status | number | 用户状态，0=正常 |
| reg_time | string | 注册日期 (YYYY-MM-DD) |

---

#### `POST /api/v1/user/update`

更新用户信息（昵称、头像 URL）。传哪个字段更新哪个。

**鉴权**: JWT Bearer Token

**请求**:

```json
{
  "nickname": "小明",
  "avatar": "https://example.com/avatar.jpg"
}
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| nickname | string | 否 | 新昵称 |
| avatar | string | 否 | 新头像 URL |

> `nickname` 和 `avatar` 至少提供一个。

**响应**:

```json
{
  "status": 0
}
```

| status | 说明 |
|--------|------|
| 0 | 更新成功 |

---

#### `POST /api/v1/user/password`

修改密码。需要提供原密码验证。

**鉴权**: JWT Bearer Token

**请求**:

```json
{
  "old_password": "old_pass",
  "new_password": "new_pass"
}
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| old_password | string | 是 | 原密码 |
| new_password | string | 是 | 新密码（至少 6 位） |

**响应**:

```json
{
  "status": 0,
  "message": "Password updated successfully"
}
```

| status | 说明 |
|--------|------|
| 0 | 修改成功 |
| 1 | 参数缺失 |
| 2 | 原密码错误 |
| 3 | 密码哈希失败 |
| 4 | 用户未找到 |
| 5 | 数据库错误 |

> 密码修改后 JWT Token 不会立即失效。建议前端在修改成功后引导用户重新登录。

---

### 追番记录（JWT 鉴权）

> 以下接口前缀均为 `/api/v1/record`，鉴权方式为 **JWT Bearer Token**。

#### `POST /api/v1/record/add`

添加追番记录。若番组未入库会先自动抓取入库。若记录曾软删除，会自动恢复。

**请求**:

```json
{
  "bangumi_id": 425998,
  "user_status": 1,
  "recorder": "1|0:00"
}
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| bangumi_id | number | 是 | bgm.tv 条目 ID |
| user_status | number | 否 | 观看状态，见[数据字典](#user_status-观看状态)，默认 0 |
| recorder | string | 否 | 初始进度（与 user_status 至少提供一个） |

**响应** (成功):

```json
{
  "status": 0,
  "local_bangumi_id": 1,
  "bangumi_id": 425998,
  "recorder": "1|0:00",
  "date": "2026-05-19T12:00:00"
}
```

**响应** (记录已存在):

```json
{
  "status": -3,
  "local_bangumi_id": 1,
  "bangumi_id": 425998,
  "recorder": "5|2:12",
  "date": null
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| local_bangumi_id | number | 本地库中的番组内部 ID |
| recorder | string | 当前进度 |
| date | string | 记录创建时间，已存在时为 null |

| status | 说明 |
|--------|------|
| 0 | 添加成功（或软删除记录已恢复） |
| -3 | 记录已存在且为激活状态，返回已有进度 |

---

#### `POST /api/v1/record/update`

更新追番进度和/或观看状态。更新进度时会自动写入 `recording_logs` 历史记录。

**请求**:

```json
{
  "bangumi_id": 425998,
  "recorder": "5|2:12",
  "user_status": 2
}
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| bangumi_id | number | 是 | bgm.tv 条目 ID |
| recorder | string | 否 | 进度记录字符串 |
| user_status | number | 否 | 观看状态 |

> `recorder` 和 `user_status` 至少提供一个，传哪个更新哪个。

**recorder 格式**：`<集数>|<时:分:秒>` 或 `<集数>|<时:分>`

| 示例 | 含义 |
|------|------|
| `5\|2:12` | 第 5 集 2 分 12 秒 |
| `12\|0:00` | 第 12 集开头 |
| `1\|1:30:00` | 第 1 集 1 小时 30 分（剧场版） |
| `3` | 第 3 集（不记录时间） |

**响应**:

```json
{
  "status": 0,
  "message": "Updated successfully"
}
```

| status | 说明 |
|--------|------|
| 0 | 更新成功 |
| -1 | 参数缺失 |
| -2 | 数据库错误 |
| -3 | 追番记录不存在 |
```

---

#### `POST /api/v1/record/get`

查询单个追番记录。

**请求**:

```json
{
  "bangumi_id": 425998
}
```

**响应**:

```json
{
  "status": 0,
  "local_bangumi_id": 1,
  "bangumi_id": 425998,
  "recorder": "5|2:12",
  "user_status": 1,
  "is_delete": 0,
  "date": "2026-05-19T12:00:00"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| is_delete | number | 0=正常, 1=已软删除 |

> **注意**: `is_delete=1` 的记录也会返回。对接方应检查此字段，避免视为正常记录。

---

#### `POST /api/v1/record/delete`

软删除追番记录。

**请求**:

```json
{
  "bangumi_id": 425998
}
```

**响应**:

```json
{
  "status": 0,
  "message": "Delete success"
}
```

> 软删除仅标记 `is_delete=1`，不会从数据库物理删除。

---

#### `GET /api/v1/record/list`

获取所有追番记录（简版）。

**请求**: 无参数

**响应**:

```json
{
  "status": 0,
  "data": [
    {
      "id": 1,
      "local_bangumi_id": 1,
      "bangumi_id": 425998,
      "recorder": "5|2:12",
      "user_status": 1,
      "is_delete": 0,
      "date": "2026-05-19T12:00:00"
    }
  ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| id | number | 记录内部 ID |
| local_bangumi_id | number | 番组本地库 ID |

---

#### `GET /api/v1/record/detail_list`

获取所有追番记录（详版，含番组标题、封面等）。

**请求**: 无参数

**响应**:

```json
{
  "status": 0,
  "data": [
    {
      "id": 1,
      "local_bangumi_id": 1,
      "bangumi_id": 425998,
      "title": "Re:从零开始的异世界生活",
      "type": 1,
      "author": "长月达平",
      "episodes": 25,
      "cover_url": "https://...",
      "recorder": "5|2:12",
      "user_status": 1,
      "is_delete": 0,
      "updated_at": "2026-05-19T12:00:00",
      "created_at": "2026-05-19T10:00:00"
    }
  ]
}
```

| 新增字段 | 类型 | 说明 |
|----------|------|------|
| title | string | 番组标题 |
| type | number | 番组类型 |
| author | string | 作者/原作 |
| episodes | number \| null | 总集数 |
| cover_url | string | 封面 URL |
| created_at | string | 记录创建时间 |
| updated_at | string | 记录最后更新时间 |

---

### 追番记录（API Token 鉴权）

> 以下接口前缀均为 `/api/v1/open`，鉴权方式为 **API Token（Query 参数）**。
>
> 功能与 JWT 鉴权版本一一对应，区别仅在于鉴权方式和传参方式：**所有参数均通过 URL Query 传递，不使用 JSON Body**。

#### `POST /api/v1/open/new`

添加追番记录（支持同时设置初始进度）。若记录曾软删除，会自动恢复。

```
POST /api/v1/open/new?token=<api_token>&bangumi_id=425998&user_status=1&recorder=1|0:00
```

**请求**: 所有参数通过 URL Query 传递（无 JSON Body）

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| token | string | 是 | API Token |
| bangumi_id | number | 是 | bgm.tv 条目 ID |
| user_status | number | 否 | 观看状态，见[数据字典](#user_status-观看状态)，默认 0 |
| recorder | string | 否 | 初始进度（与 user_status 至少提供一个） |

**响应** (成功):

```json
{
  "status": 0,
  "local_bangumi_id": 1,
  "bangumi_id": 425998,
  "recorder": "1|0:00",
  "date": "2026-05-19T12:00:00"
}
```

**响应** (记录已存在):

```json
{
  "status": -3,
  "local_bangumi_id": 1,
  "bangumi_id": 425998,
  "recorder": "5|2:12",
  "date": null
}
```

| status | 说明 |
|--------|------|
| 0 | 添加成功（或软删除记录已恢复） |
| -1 | 参数缺失 |
| -2 | 番组未找到 |
| -3 | 记录已存在且为激活状态，返回已有进度 |

> Token 缺失或无效时返回 **HTTP 401**（无 JSON body）。

---

#### `POST /api/v1/open/update`

更新进度。更新进度时会自动写入 `recording_logs` 历史记录。

```
POST /api/v1/open/update?token=<api_token>&bangumi_id=425998&recorder=5|2:12&user_status=1
```

**请求**: 所有参数通过 URL Query 传递（无 JSON Body）

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| token | string | 是 | API Token |
| bangumi_id | number | 是 | bgm.tv 条目 ID |
| recorder | string | 否 | 进度字符串 |
| user_status | number | 否 | 观看状态 |

**响应**:

```json
{
  "status": 0,
  "message": "Updated successfully"
}
```

| status | 说明 |
|--------|------|
| 0 | 更新成功 |
| -1 | 参数缺失 |
| -2 | 番组未找到 / 数据库错误 |
| -3 | 追番记录不存在 |

> Token 缺失或无效时返回 **HTTP 401**（无 JSON body）。

---

#### `POST /api/v1/open/get`

查询单条记录。

```
POST /api/v1/open/get?token=<api_token>&bangumi_id=425998
```

**请求**: 所有参数通过 URL Query 传递（无 JSON Body）

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| token | string | 是 | API Token |
| bangumi_id | number | 是 | bgm.tv 条目 ID |

**响应**:

```json
{
  "status": 0,
  "local_bangumi_id": 1,
  "bangumi_id": 425998,
  "recorder": "5|2:12",
  "user_status": 1,
  "is_delete": 0,
  "date": "2026-05-19T12:00:00"
}
```

> Token 缺失或无效时返回 **HTTP 401**。

---

#### `POST /api/v1/open/delete`

软删除记录。

```
POST /api/v1/open/delete?token=<api_token>&bangumi_id=425998
```

**请求**: 所有参数通过 URL Query 传递（无 JSON Body）

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| token | string | 是 | API Token |
| bangumi_id | number | 是 | bgm.tv 条目 ID |

**响应**:

```json
{
  "status": 0,
  "message": "Delete success"
}
```

| status | 说明 |
|--------|------|
| 0 | 删除成功 |
| -1 | 参数缺失 |
| -2 | 番组未找到 / 数据库错误 |
| -3 | 追番记录不存在 |

> Token 缺失或无效时返回 **HTTP 401**（无 JSON body）。

---

#### `GET /api/v1/open/list`

获取追番记录列表（简版）。

```
GET /api/v1/open/list?token=<api_token>
```

响应同 [`/api/v1/record/list`](#get-apiv1recordlist)。

> Token 缺失或无效时返回 **HTTP 401**。

---

#### `GET /api/v1/open/detail_list`

获取追番记录列表（详版，含番组信息）。

```
GET /api/v1/open/detail_list?token=<api_token>
```

响应同 [`/api/v1/record/detail_list`](#get-apiv1recorddetail_list)。

> Token 缺失或无效时返回 **HTTP 401**。

---

### 用户信息（API Token 鉴权）

> 以下接口前缀均为 `/api/v1/open`，鉴权方式为 **API Token（Query 参数）**。

#### `GET /api/v1/open/user/info`

获取当前用户的基本信息。

```
GET /api/v1/open/user/info?token=<api_token>
```

**请求**: 无参数（Token 通过 Query 传递）

**响应**:

```json
{
  "id": 1,
  "uuid": "7047965d-036b-4b1d-a877-7f5ced0147e4",
  "username": "user",
  "nickname": "小明",
  "email": "user@example.com",
  "avatar": "",
  "status": 0,
  "reg_time": "2026-05-19"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| id | number | 用户 ID |
| uuid | string | 用户 UUID（v7） |
| username | string | 用户名 |
| nickname | string | 昵称 |
| email | string | 邮箱 |
| avatar | string | 头像 URL |
| status | number | 用户状态 |
| reg_time | string | 注册日期 |

> Token 缺失或无效时返回 **HTTP 401**。用户不存在时返回各字段空值 / 0。

---

## 数据字典

### `user_status` 观看状态

| 值 | 含义 |
|----|------|
| 0 | 想看 |
| 1 | 在看 |
| 2 | 看过 |
| 3 | 搁置 |
| 4 | 抛弃 |

### `type` 番组类型

| 值 | 含义 |
|----|------|
| 1 | TV |
| 2 | 剧场版 / 电影 |
| 3 | OVA |
| 4 | ONA / WEB |
| 5 | TV Short |
| 6 | 音乐 |
| 7 | 书籍 |
| 8 | 其他 |

### `recorder` 进度格式

```
<episode>|<hours:minutes:seconds>
```

- `episode`: 当前集数（number）
- `hours` / `minutes` / `seconds`: 当前集内的播放进度
- 分隔符为竖线 `|`

| 示例 | 含义 |
|------|------|
| `5` | 第 5 集 |
| `5\|2:12` | 第 5 集 2 分 12 秒 |
| `12\|0:00` | 第 12 集 0 分 0 秒 |
| `1\|1:30:45` | 第 1 集 1 小时 30 分 45 秒 |

---

## 对接示例

### 场景一：自动化追番工具（如播放器插件）

使用 API Token 方式，避免频繁登录：

```bash
# 1. 获取令牌（注册时返回，或登录后 regenerate）
# 假设 token = "550e8400-e29b-41d4-a716-446655440000"

# 2. 播放时更新进度（参数通过 Query 传递）
curl -X POST "http://127.0.0.1:8080/api/v1/open/update?token=550e8400-e29b-41d4-a716-446655440000&bangumi_id=425998&recorder=5%7C2%3A12"

# 3. 查看当前所有追番
curl "http://127.0.0.1:8080/api/v1/open/detail_list?token=550e8400-e29b-41d4-a716-446655440000"

# 4. 开始追一部新番
curl -X POST "http://127.0.0.1:8080/api/v1/open/new?token=550e8400-e29b-41d4-a716-446655440000&bangumi_id=425998&user_status=1&recorder=1%7C0%3A00"
```

### 场景二：第三方平台同步

```python
import requests

BASE = "http://127.0.0.1:8080"
TOKEN = "550e8400-e29b-41d4-a716-446655440000"

# 获取追番列表
resp = requests.get(f"{BASE}/api/v1/open/detail_list", params={"token": TOKEN})
data = resp.json()
if data["status"] == 0:
    for item in data["data"]:
        if item["is_delete"] == 0:
            print(f"{item['title']} - 进度: {item['recorder']} - 状态: {item['user_status']}")

# 更新进度（参数通过 Query 传递）
resp = requests.post(
    f"{BASE}/api/v1/open/update",
    params={"token": TOKEN, "bangumi_id": 425998, "recorder": "6|15:30"}
)
```

### 场景三：Web 前端（JWT 方式）

```javascript
// 登录
const loginResp = await fetch('/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ username: 'user', password: '123456' })
});
const { token } = await loginResp.json();

// 搜索番组
const searchResp = await fetch('/api/v1/search/bangumi', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`
  },
  body: JSON.stringify({ title: 'Re:Zero', page: 1 })
});
const { data } = await searchResp.json();

// 添加追番
await fetch('/api/v1/record/add', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`
  },
  body: JSON.stringify({ bangumi_id: data[0].bangumi_id, user_status: 1 })
});
```
