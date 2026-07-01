use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use sqlx::Row;
use sqlx::mysql::MySqlPool;

use crate::api::open::api_token::{PERM_VIEW_INFO, require_token_with_perm};
use crate::api::open::user::GetTokenQuery;
use crate::api::user::UserInfo;
use crate::api::v2::response::{ApiResponse, forbidden, success, unauthorized};

pub async fn get_info(
    State(pool): State<MySqlPool>,
    Query(params): Query<GetTokenQuery>,
) -> (StatusCode, Json<ApiResponse<UserInfo>>) {
    let token = match params.token.as_ref() {
        Some(token) => token,
        None => return unauthorized("Missing API token"),
    };

    let token_info = match require_token_with_perm(&pool, Some(token), &[PERM_VIEW_INFO]).await {
        Ok(info) => info,
        Err(StatusCode::UNAUTHORIZED) => return unauthorized("Invalid API token"),
        Err(StatusCode::FORBIDDEN) => return forbidden("Insufficient permissions"),
        Err(_) => return unauthorized("Invalid API token"),
    };

    let user_info = sqlx::query(
        "SELECT id, uuid, username, nickname, email, avatar, status, is_admin, DATE(created_at) AS reg_time FROM users WHERE id = ?",
    )
    .bind(token_info.user_id)
    .fetch_one(&pool)
    .await
    .map(|row| UserInfo {
        id: row.try_get::<u32, _>("id").map(|id| id as i64).unwrap_or(0),
        uuid: row.try_get("uuid").unwrap_or_default(),
        username: row.try_get("username").unwrap_or_default(),
        nickname: row.try_get("nickname").unwrap_or_default(),
        email: row.try_get("email").unwrap_or_default(),
        avatar: row.try_get("avatar").unwrap_or_default(),
        status: row.try_get("status").unwrap_or(0),
        is_admin: row
            .try_get::<i8, _>("is_admin")
            .is_ok_and(|value| value != 0),
        reg_time: row.try_get("reg_time").ok(),
    });

    match user_info {
        Ok(info) => (StatusCode::OK, success(info).1),
        Err(_) => success(UserInfo {
            id: 0,
            uuid: String::new(),
            username: String::new(),
            nickname: String::new(),
            email: String::new(),
            avatar: String::new(),
            status: 0,
            is_admin: false,
            reg_time: None,
        }),
    }
}
