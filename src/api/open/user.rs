use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::require_api_token;
use crate::auth_bearer::AuthUser;

pub use crate::api::user::UserInfo;

#[derive(Debug, Deserialize)]
pub struct GetTokenQuery {
    pub token: Option<String>,
}

pub async fn get_info(
    State(pool): State<MySqlPool>,
    Query(params): Query<GetTokenQuery>,
) -> Result<Json<UserInfo>, StatusCode> {
    let _user_id = require_api_token(&pool, params.token.as_deref()).await?;

    Ok(crate::api::user::get_info(
        State(pool),
        axum::extract::Extension(AuthUser { user_id: _user_id }),
    )
    .await)
}
