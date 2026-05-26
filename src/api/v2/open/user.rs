use axum::{extract::{Query, State}, http::StatusCode, Json};
use sqlx::mysql::MySqlPool;

use crate::api::open::api_token::{require_token_with_perm, PERM_VIEW_INFO};
use crate::api::open::user::GetTokenQuery;
use crate::api::user::UserInfo;
use crate::api::v2::response::{success, unauthorized, forbidden, ApiResponse};

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

    let user_info = sqlx::query_as!(
        UserInfo,
        "SELECT id, uuid, username, nickname, email, avatar, status, DATE(created_at) AS reg_time FROM users WHERE id = ?",
        token_info.user_id
    )
    .fetch_one(&pool)
    .await;

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
            reg_time: None,
        }),
    }
}
