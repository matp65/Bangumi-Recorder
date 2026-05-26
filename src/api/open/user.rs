use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::{require_token_with_perm, PERM_VIEW_INFO};
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
    let token_info = require_token_with_perm(&pool, params.token.as_deref(), &[PERM_VIEW_INFO]).await?;

    Ok(crate::api::user::get_info(
        State(pool),
        axum::extract::Extension(AuthUser { user_id: token_info.user_id }),
    )
    .await)
}
