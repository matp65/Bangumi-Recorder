use axum::{
    Json,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::mysql::MySqlPool;

use super::response::{
    ApiResponse, bad_request, internal_error, not_found, success, success_empty,
};
use crate::api::api_token::{ALL_COMBINED, PERM_LABELS, hash_token};
use crate::api::logs::{operation_metadata, write_system_log};
use crate::auth_bearer::AuthUser;

#[derive(Serialize)]
pub struct TokenListItem {
    pub id: i64,
    pub name: String,
    pub permissions: u64,
    pub is_active: bool,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct CreateTokenData {
    pub id: i64,
    pub name: String,
    pub raw_token: String,
    pub permissions: u64,
}

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub name: Option<String>,
    pub permissions: Option<u64>,
}

#[derive(Deserialize)]
pub struct UpdateTokenRequest {
    pub name: Option<String>,
    pub permissions: Option<u64>,
    pub is_active: Option<bool>,
}

#[derive(Serialize)]
pub struct PermissionLabel {
    pub label: &'static str,
    pub value: u64,
    pub description: &'static str,
}

#[derive(Serialize)]
pub struct PermissionLabelsResponse {
    pub labels: Vec<PermissionLabel>,
    pub all_value: u64,
}

pub async fn permission_labels() -> (StatusCode, Json<ApiResponse<PermissionLabelsResponse>>) {
    success(PermissionLabelsResponse {
        labels: PERM_LABELS
            .iter()
            .map(|&(label, value, description)| PermissionLabel {
                label,
                value,
                description,
            })
            .collect(),
        all_value: ALL_COMBINED,
    })
}

async fn username_for_log(pool: &MySqlPool, user_id: i64) -> String {
    sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| format!("user#{}", user_id))
}

pub async fn list_tokens(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<TokenListItem>>>) {
    let rows = sqlx::query(
        "SELECT id, name, permissions, is_active, last_used_at, created_at, updated_at \
         FROM api_tokens WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(auth_user.user_id)
    .fetch_all(&pool)
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to list tokens: {:?}", e);
            return internal_error("Failed to list tokens");
        }
    };

    let items: Vec<TokenListItem> = rows
        .iter()
        .map(|r| {
            let last_used: Option<String> =
                r.try_get("last_used_at")
                    .ok()
                    .and_then(|v: Option<chrono::NaiveDateTime>| {
                        v.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    });
            let created: String = r
                .try_get::<chrono::NaiveDateTime, _>("created_at")
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();
            let updated: String = r
                .try_get::<chrono::NaiveDateTime, _>("updated_at")
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();
            let is_active: i8 = r.try_get("is_active").unwrap_or(0);
            TokenListItem {
                id: r.try_get::<u64, _>("id").map(|v| v as i64).unwrap_or(0),
                name: r.try_get("name").unwrap_or_default(),
                permissions: r.try_get::<u64, _>("permissions").unwrap_or(0),
                is_active: is_active != 0,
                last_used_at: last_used,
                created_at: created,
                updated_at: updated,
            }
        })
        .collect();

    success(items)
}

pub async fn create_token(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    headers: HeaderMap,
    Json(payload): Json<CreateTokenRequest>,
) -> (StatusCode, Json<ApiResponse<CreateTokenData>>) {
    let name = payload.name.unwrap_or_default();
    if name.trim().is_empty() {
        return bad_request("Token name is required");
    }
    let name = name.trim().to_string();

    let permissions = payload.permissions.unwrap_or(0);
    if permissions == 0 {
        return bad_request("At least one permission must be selected");
    }

    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = hash_token(&raw_token);

    let result = sqlx::query(
        "INSERT INTO api_tokens (user_id, name, token_hash, permissions) VALUES (?, ?, ?, ?)",
    )
    .bind(auth_user.user_id)
    .bind(&name)
    .bind(&token_hash)
    .bind(permissions as i64)
    .execute(&pool)
    .await;

    match result {
        Ok(res) => {
            let id = res.last_insert_id() as i64;
            let username = username_for_log(&pool, auth_user.user_id).await;
            write_system_log(
                &pool,
                "info",
                "api_token",
                "api_token_created",
                &format!("{} created API Token '{}'", username, name),
                Some(auth_user.user_id),
                Some(operation_metadata(
                    &headers,
                    "JWT",
                    serde_json::json!({ "token_id": id, "name": name, "permissions": permissions, "username": username }),
                )),
            )
            .await;
            success(CreateTokenData {
                id,
                name,
                raw_token,
                permissions,
            })
        }
        Err(e) => {
            log::error!("Failed to create token: {:?}", e);
            internal_error("Failed to create token")
        }
    }
}

pub async fn update_token(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<u64>,
    headers: HeaderMap,
    Json(payload): Json<UpdateTokenRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    // Verify the token belongs to this user
    let existing = sqlx::query(
        "SELECT id, name, permissions, is_active FROM api_tokens WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_optional(&pool)
    .await;

    let existing = match existing {
        Ok(Some(row)) => row,
        Ok(None) => return not_found("Token not found"),
        Err(e) => {
            log::error!("Failed to find token: {:?}", e);
            return internal_error("Database error");
        }
    };

    let old_name: String = existing.try_get("name").unwrap_or_default();
    let old_permissions: u64 = existing.try_get("permissions").unwrap_or(0);
    let old_is_active: i8 = existing.try_get("is_active").unwrap_or(0);

    if let Some(name) = &payload.name {
        let name = name.trim();
        if name.is_empty() {
            return bad_request("Token name cannot be empty");
        }
        if let Err(e) = sqlx::query("UPDATE api_tokens SET name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(&pool)
            .await
        {
            log::error!("Failed to update token name: {:?}", e);
            return internal_error("Failed to update token");
        }
    }

    if let Some(permissions) = payload.permissions
        && let Err(e) = sqlx::query("UPDATE api_tokens SET permissions = ? WHERE id = ?")
            .bind(permissions as i64)
            .bind(id)
            .execute(&pool)
            .await
    {
        log::error!("Failed to update token permissions: {:?}", e);
        return internal_error("Failed to update token");
    }

    if let Some(is_active) = payload.is_active
        && let Err(e) = sqlx::query("UPDATE api_tokens SET is_active = ? WHERE id = ?")
            .bind(is_active as i8)
            .bind(id)
            .execute(&pool)
            .await
    {
        log::error!("Failed to update token active status: {:?}", e);
        return internal_error("Failed to update token");
    }

    let username = username_for_log(&pool, auth_user.user_id).await;
    write_system_log(
        &pool,
        "info",
        "api_token",
        "api_token_updated",
        &format!("{} updated API Token '{}'", username, old_name),
        Some(auth_user.user_id),
        Some(operation_metadata(
            &headers,
            "JWT",
            serde_json::json!({
                "token_id": id,
                "old": {
                    "name": old_name,
                    "permissions": old_permissions,
                    "is_active": old_is_active != 0,
                },
                "new": {
                    "name": payload.name,
                    "permissions": payload.permissions,
                    "is_active": payload.is_active,
                },
                "username": username,
            }),
        )),
    )
    .await;

    success_empty()
}

pub async fn delete_token(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<u64>,
    headers: HeaderMap,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let existing =
        sqlx::query("SELECT name, permissions FROM api_tokens WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(auth_user.user_id)
            .fetch_optional(&pool)
            .await;

    let existing = match existing {
        Ok(row) => row,
        Err(e) => {
            log::error!("Failed to find token before delete: {:?}", e);
            return internal_error("Database error");
        }
    };

    let result = sqlx::query("DELETE FROM api_tokens WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(auth_user.user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                if let Some(row) = existing {
                    let username = username_for_log(&pool, auth_user.user_id).await;
                    let name = row.try_get::<String, _>("name").unwrap_or_default();
                    let permissions = row.try_get::<u64, _>("permissions").unwrap_or(0);
                    write_system_log(
                        &pool,
                        "info",
                        "api_token",
                        "api_token_deleted",
                        &format!("{} deleted API Token '{}'", username, name),
                        Some(auth_user.user_id),
                        Some(operation_metadata(
                            &headers,
                            "JWT",
                            serde_json::json!({
                                "token_id": id,
                                "name": name,
                                "permissions": permissions,
                                "username": username,
                            }),
                        )),
                    )
                    .await;
                }
                success_empty()
            } else {
                not_found("Token not found")
            }
        }
        Err(e) => {
            log::error!("Failed to delete token: {:?}", e);
            internal_error("Failed to delete token")
        }
    }
}
