use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::{PERM_VIEW_INFO, api_token_from_request, require_token_with_perm};
use crate::auth_bearer::AuthUser;

pub use crate::api::user::UserInfo;

#[derive(Debug, Deserialize)]
pub struct GetTokenQuery {
    pub token: Option<String>,
}

pub async fn get_info(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Query(params): Query<GetTokenQuery>,
) -> Result<Json<UserInfo>, StatusCode> {
    let token = api_token_from_request(&headers, params.token.as_deref());
    let token_info = require_token_with_perm(&pool, token, &[PERM_VIEW_INFO]).await?;

    Ok(crate::api::user::get_info(
        State(pool),
        axum::extract::Extension(AuthUser {
            user_id: token_info.user_id,
        }),
    )
    .await)
}
