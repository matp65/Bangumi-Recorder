use axum::{
    Json,
    extract::{Extension, Query, State},
    http::{HeaderMap, StatusCode},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{MySql, QueryBuilder, Row, mysql::MySqlPool};

use crate::api::v2::response::{
    ApiResponse, bad_request, forbidden, internal_error, success, success_empty,
};
use crate::auth_bearer::AuthUser;

#[derive(Clone, Copy)]
pub enum LogTarget {
    Bangumi(u32),
    Imdb(u32),
    Other(u32),
}

impl LogTarget {
    fn parts(self) -> (&'static str, Option<u32>) {
        match self {
            Self::Bangumi(id) => ("bangumi", Some(id)),
            Self::Imdb(id) => ("imdb", Some(id)),
            Self::Other(id) => ("other", Some(id)),
        }
    }
}

pub async fn write_recording_log(
    pool: &MySqlPool,
    recording_id: u32,
    user_id: Option<i64>,
    target: LogTarget,
    action: &str,
    field_name: Option<&str>,
    old_value: Option<Value>,
    new_value: Option<Value>,
    metadata: Option<Value>,
) {
    if old_value
        .as_ref()
        .zip(new_value.as_ref())
        .is_some_and(|(old, new)| old == new)
    {
        return;
    }

    let (target_type, target_id) = target.parts();
    let old_value = old_value.map(|v| v.to_string());
    let new_value = new_value.map(|v| v.to_string());
    let metadata = metadata.map(|v| v.to_string());

    if let Err(e) = sqlx::query(
        r#"INSERT INTO recording_logs
           (recording_id, user_id, target_type, target_id, action, field_name, old_value, new_value, metadata)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(recording_id)
    .bind(user_id)
    .bind(target_type)
    .bind(target_id)
    .bind(action)
    .bind(field_name)
    .bind(old_value)
    .bind(new_value)
    .bind(metadata)
    .execute(pool)
    .await
    {
        log::warn!("Failed to write recording log: {}", e);
    }
}

pub async fn write_system_log(
    pool: &MySqlPool,
    level: &str,
    category: &str,
    action: &str,
    message: &str,
    user_id: Option<i64>,
    metadata: Option<Value>,
) {
    let metadata = metadata.map(|v| v.to_string());
    if let Err(e) = sqlx::query(
        r#"INSERT INTO system_logs (level, category, action, message, user_id, metadata)
           VALUES (?, ?, ?, ?, ?, ?)"#,
    )
    .bind(level)
    .bind(category)
    .bind(action)
    .bind(message)
    .bind(user_id)
    .bind(metadata)
    .execute(pool)
    .await
    {
        log::warn!("Failed to write system log: {}", e);
    }
}

pub fn client_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
        })
        .map(ToString::to_string)
}

pub fn operation_metadata(headers: &HeaderMap, auth_type: &str, extra: Value) -> Value {
    json!({
        "ip": client_ip(headers),
        "auth_type": auth_type,
        "extra": extra,
    })
}

pub async fn is_admin(pool: &MySqlPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let row = sqlx::query("SELECT is_admin FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    Ok(row
        .and_then(|row| row.try_get::<i8, _>("is_admin").ok())
        .is_some_and(|value| value != 0))
}

#[derive(Deserialize)]
pub struct LogListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub target: Option<String>,
    pub action: Option<String>,
    pub category: Option<String>,
    pub username: Option<String>,
}

#[derive(Serialize)]
pub struct RecordingLogItem {
    pub id: u64,
    pub recording_id: Option<u32>,
    pub user_id: Option<i64>,
    pub target_type: String,
    pub target_id: Option<u32>,
    pub target_title: Option<String>,
    pub action: String,
    pub field_name: Option<String>,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub metadata: Option<Value>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct SystemLogItem {
    pub id: u64,
    pub level: String,
    pub category: String,
    pub action: String,
    pub message: String,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct LogListData<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
}

fn page(query: &LogListQuery) -> (u32, u32, u32) {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * page_size;
    (page, page_size, offset)
}

fn parse_json(raw: Option<String>) -> Option<Value> {
    raw.and_then(|v| serde_json::from_str(&v).ok())
}

pub async fn list_recording_logs(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<LogListQuery>,
) -> (StatusCode, Json<ApiResponse<LogListData<RecordingLogItem>>>) {
    let (page, page_size, offset) = page(&query);
    let mut qb = QueryBuilder::<MySql>::new(
        r#"SELECT l.id, l.recording_id, CAST(l.user_id AS SIGNED) AS user_id,
                  l.target_type, l.target_id,
                  COALESCE(b.title, e.title, o.name) AS target_title,
                  l.action, l.field_name,
                  CAST(old_value AS CHAR) AS old_value,
                  CAST(new_value AS CHAR) AS new_value,
                  CAST(metadata AS CHAR) AS metadata,
                  l.created_at
           FROM recording_logs l
           LEFT JOIN bangumi_info_easy b ON l.target_type = 'bangumi' AND b.id = l.target_id
           LEFT JOIN external_media e ON l.target_type = 'imdb' AND e.id = l.target_id
           LEFT JOIN other_recorders o ON l.target_type = 'other' AND o.id = l.target_id
           WHERE l.user_id = "#,
    );
    qb.push_bind(auth_user.user_id);
    if let Some(start_time) = query.start_time.as_deref().filter(|v| !v.is_empty()) {
        qb.push(" AND l.created_at >= ").push_bind(start_time);
    }
    if let Some(end_time) = query.end_time.as_deref().filter(|v| !v.is_empty()) {
        qb.push(" AND l.created_at <= ").push_bind(end_time);
    }
    if let Some(action) = query.action.as_deref().filter(|v| !v.is_empty()) {
        qb.push(" AND l.action = ").push_bind(action);
    }
    if let Some(target) = query.target.as_deref().filter(|v| !v.is_empty()) {
        let target_like = format!("%{}%", target);
        qb.push(" AND (COALESCE(b.title, e.title, o.name) LIKE ")
            .push_bind(target_like)
            .push(" OR l.target_type = ")
            .push_bind(target)
            .push(" OR CAST(l.target_id AS CHAR) = ")
            .push_bind(target)
            .push(" OR CAST(l.recording_id AS CHAR) = ")
            .push_bind(target)
            .push(")");
    }
    qb.push(" ORDER BY l.created_at DESC, l.id DESC LIMIT ")
        .push_bind(page_size)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows = qb.build().fetch_all(&pool).await;

    match rows {
        Ok(rows) => success(LogListData {
            items: rows
                .into_iter()
                .map(|row| RecordingLogItem {
                    id: row.get("id"),
                    recording_id: row.get("recording_id"),
                    user_id: row.get("user_id"),
                    target_type: row.get("target_type"),
                    target_id: row.get("target_id"),
                    target_title: row.get("target_title"),
                    action: row.get("action"),
                    field_name: row.get("field_name"),
                    old_value: parse_json(row.get("old_value")),
                    new_value: parse_json(row.get("new_value")),
                    metadata: parse_json(row.get("metadata")),
                    created_at: row.get("created_at"),
                })
                .collect(),
            page,
            page_size,
        }),
        Err(e) => {
            log::error!("Failed to list recording logs: {}", e);
            internal_error("Failed to list recording logs")
        }
    }
}

pub async fn list_system_logs(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<LogListQuery>,
) -> (StatusCode, Json<ApiResponse<LogListData<SystemLogItem>>>) {
    match is_admin(&pool, auth_user.user_id).await {
        Ok(true) => {}
        Ok(false) => return forbidden("System logs require administrator privileges"),
        Err(e) => {
            log::error!("Failed to check admin status: {}", e);
            return internal_error("Failed to check admin status");
        }
    }

    let (page, page_size, offset) = page(&query);
    let mut qb = QueryBuilder::<MySql>::new(
        r#"SELECT s.id, s.level, s.category, s.action, s.message, CAST(s.user_id AS SIGNED) AS user_id,
                  u.username,
                  CAST(s.metadata AS CHAR) AS metadata,
                  s.created_at
           FROM system_logs s
           LEFT JOIN users u ON u.id = s.user_id
           WHERE 1 = 1"#,
    );
    if let Some(start_time) = query.start_time.as_deref().filter(|v| !v.is_empty()) {
        qb.push(" AND s.created_at >= ").push_bind(start_time);
    }
    if let Some(end_time) = query.end_time.as_deref().filter(|v| !v.is_empty()) {
        qb.push(" AND s.created_at <= ").push_bind(end_time);
    }
    if let Some(category) = query.category.as_deref().filter(|v| !v.is_empty()) {
        qb.push(" AND s.category = ").push_bind(category);
    }
    if let Some(action) = query.action.as_deref().filter(|v| !v.is_empty()) {
        qb.push(" AND s.action = ").push_bind(action);
    }
    if let Some(username) = query.username.as_deref().filter(|v| !v.is_empty()) {
        let username_like = format!("%{}%", username);
        qb.push(" AND (u.username LIKE ")
            .push_bind(username_like)
            .push(" OR CAST(s.user_id AS CHAR) = ")
            .push_bind(username)
            .push(")");
    }
    qb.push(" ORDER BY s.created_at DESC, s.id DESC LIMIT ")
        .push_bind(page_size)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows = qb.build().fetch_all(&pool).await;

    match rows {
        Ok(rows) => success(LogListData {
            items: rows
                .into_iter()
                .map(|row| SystemLogItem {
                    id: row.get("id"),
                    level: row.get("level"),
                    category: row.get("category"),
                    action: row.get("action"),
                    message: row.get("message"),
                    user_id: row.get("user_id"),
                    username: row.get("username"),
                    metadata: parse_json(row.get("metadata")),
                    created_at: row.get("created_at"),
                })
                .collect(),
            page,
            page_size,
        }),
        Err(e) => {
            log::error!("Failed to list system logs: {}", e);
            internal_error("Failed to list system logs")
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateAutoCleanupRequest {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct AutoCleanupSetting {
    pub enabled: bool,
}

pub async fn get_auto_cleanup_setting(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<AutoCleanupSetting>>) {
    match auto_cleanup_enabled(&pool, auth_user.user_id).await {
        Ok(enabled) => success(AutoCleanupSetting { enabled }),
        Err(e) => {
            log::error!("Failed to get auto cleanup setting: {}", e);
            internal_error("Failed to get auto cleanup setting")
        }
    }
}

pub async fn update_auto_cleanup_setting(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<UpdateAutoCleanupRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let value = json!({ "enabled": body.enabled }).to_string();
    let result = sqlx::query(
        r#"INSERT INTO user_settings (user_id, setting_key, setting_value)
           VALUES (?, 'auto_delete_soft_deleted_after_30d', ?)
           ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value)"#,
    )
    .bind(auth_user.user_id)
    .bind(value)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            write_system_log(
                &pool,
                "info",
                "settings",
                "auto_cleanup_updated",
                if body.enabled {
                    "Enabled automatic cleanup for soft-deleted records"
                } else {
                    "Disabled automatic cleanup for soft-deleted records"
                },
                Some(auth_user.user_id),
                Some(json!({ "enabled": body.enabled, "retention_days": 30 })),
            )
            .await;
            if body.enabled {
                let _ = cleanup_soft_deleted_records(&pool, Some(auth_user.user_id)).await;
            }
            success_empty()
        }
        Err(e) => {
            log::error!("Failed to update auto cleanup setting: {}", e);
            bad_request("Failed to update auto cleanup setting")
        }
    }
}

pub async fn auto_cleanup_enabled(pool: &MySqlPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "SELECT CAST(setting_value AS CHAR) AS setting_value FROM user_settings WHERE user_id = ? AND setting_key = 'auto_delete_soft_deleted_after_30d'",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row
        .and_then(|row| row.get::<Option<String>, _>("setting_value"))
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|value| value.get("enabled").and_then(Value::as_bool))
        .unwrap_or(false))
}

pub async fn cleanup_soft_deleted_records(
    pool: &MySqlPool,
    user_id: Option<i64>,
) -> Result<u64, sqlx::Error> {
    let result = if let Some(user_id) = user_id {
        sqlx::query(
            "DELETE FROM recordings WHERE user_id = ? AND is_delete = 1 AND updated_at < DATE_SUB(NOW(), INTERVAL 30 DAY)",
        )
        .bind(user_id)
        .execute(pool)
        .await?
    } else {
        sqlx::query(
            r#"DELETE r FROM recordings r
               JOIN user_settings s ON s.user_id = r.user_id
               WHERE s.setting_key = 'auto_delete_soft_deleted_after_30d'
                 AND JSON_UNQUOTE(JSON_EXTRACT(s.setting_value, '$.enabled')) = 'true'
                 AND r.is_delete = 1
                 AND r.updated_at < DATE_SUB(NOW(), INTERVAL 30 DAY)"#,
        )
        .execute(pool)
        .await?
    };
    let deleted = result.rows_affected();
    if deleted > 0 {
        write_system_log(
            pool,
            "info",
            "cleanup",
            "soft_deleted_records_removed",
            "Removed soft-deleted records older than 30 days",
            None,
            Some(json!({ "deleted": deleted, "retention_days": 30, "user_id": user_id })),
        )
        .await;
    }
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn query(page_value: Option<u32>, page_size: Option<u32>) -> LogListQuery {
        LogListQuery {
            page: page_value,
            page_size,
            start_time: None,
            end_time: None,
            target: None,
            action: None,
            category: None,
            username: None,
        }
    }

    #[test]
    fn page_defaults_and_clamps_page_size() {
        assert_eq!(page(&query(None, None)), (1, 50, 0));
        assert_eq!(page(&query(Some(0), Some(0))), (1, 1, 0));
        assert_eq!(page(&query(Some(3), Some(500))), (3, 100, 200));
    }

    #[test]
    fn parse_json_ignores_invalid_values() {
        assert_eq!(
            parse_json(Some(r#"{"a":1}"#.to_string())),
            Some(json!({ "a": 1 }))
        );
        assert_eq!(parse_json(Some("not json".to_string())), None);
        assert_eq!(parse_json(None), None);
    }

    #[test]
    fn client_ip_prefers_first_forwarded_ip() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.1, 10.0.0.1"),
        );
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.2"));

        assert_eq!(client_ip(&headers), Some("203.0.113.1".to_string()));
    }

    #[test]
    fn client_ip_falls_back_to_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.2"));

        assert_eq!(client_ip(&headers), Some("198.51.100.2".to_string()));
    }

    #[test]
    fn operation_metadata_includes_auth_type_ip_and_extra() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.2"));

        let metadata = operation_metadata(&headers, "JWT", json!({ "username": "alice" }));

        assert_eq!(metadata["ip"], "198.51.100.2");
        assert_eq!(metadata["auth_type"], "JWT");
        assert_eq!(metadata["extra"]["username"], "alice");
    }
}
