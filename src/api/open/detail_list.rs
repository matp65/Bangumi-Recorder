use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::require_api_token;
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
    let user_id = require_api_token(&pool, params.token.as_deref()).await?;

    Ok(crate::api::detail_list::get_detail_list(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
    )
    .await)
}
