use axum::{
    extract::{State, Query},
    Json, http::StatusCode
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;

use super::api_token::check_api_token;

#[derive(Debug, Deserialize)]
pub struct GetTokenQuery {
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub email: String,
    pub avatar: String,
    pub status: i8,
    pub reg_time: NaiveDate,
}

pub async fn get_info(
    State(pool): State<MySqlPool>,
    Query(params): Query<GetTokenQuery>
) -> Result<Json<UserInfo>, StatusCode> {

    let token = match params.token.as_ref() {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    let user_info = sqlx::query_as!(
        UserInfo,
        "SELECT id, username, nickname, email, avatar, status, created_at AS reg_time FROM users WHERE id = ?",
        user_id
    )
    .fetch_one(&pool)
    .await;

    match user_info {
        Ok(info) => Ok(Json(UserInfo {
            id: info.id,
            username: info.username,
            nickname: info.nickname,
            email: info.email,
            avatar: info.avatar,
            status: info.status,
            reg_time: info.reg_time,
        })),
        Err(_) => Ok(Json(UserInfo {
            id: 0,
            username: String::new(),
            nickname: String::new(),
            email: String::new(),
            avatar: String::new(),
            status: 0,
            reg_time: NaiveDate::from_ymd_opt(1970, 1, 1).expect("Failed to create date"),
        })),
    }
}