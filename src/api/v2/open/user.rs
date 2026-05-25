use axum::{extract::{Query, State}, http::StatusCode, Json};
use sqlx::mysql::MySqlPool;

use crate::api::open::api_token::check_api_token;
use crate::api::open::user::GetTokenQuery;
use crate::api::user::UserInfo;
use crate::api::v2::response::{success, unauthorized, ApiResponse};

pub async fn get_info(
    State(pool): State<MySqlPool>,
    Query(params): Query<GetTokenQuery>,
) -> (StatusCode, Json<ApiResponse<UserInfo>>) {
    let token = match params.token.as_ref() {
        Some(token) => token,
        None => return unauthorized("Missing API token"),
    };

    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => return unauthorized("Invalid API token"),
    };

    let user_info = sqlx::query_as!(
        UserInfo,
        "SELECT id, uuid, username, nickname, email, avatar, status, DATE(created_at) AS reg_time FROM users WHERE id = ?",
        user_id
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
