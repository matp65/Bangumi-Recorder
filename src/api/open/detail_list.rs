use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::{require_token_with_perm, PERM_READ, PERM_WRITE};
use crate::auth_bearer::AuthUser;

pub use crate::api::detail_list::{DetailListResponse, DetailListItem};

#[derive(Deserialize)]
pub struct DetailListQuery {
    pub token: Option<String>,
}

pub async fn get_detail_list(
    State(pool): State<MySqlPool>,
    Query(params): Query<DetailListQuery>,
) -> Result<Json<DetailListResponse>, StatusCode> {
    let token_info = require_token_with_perm(&pool, params.token.as_deref(), &[PERM_READ, PERM_WRITE]).await?;
    let user_id = token_info.user_id;

    Ok(crate::api::detail_list::get_detail_list(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
    )
    .await)
}
