use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use sqlx::mysql::MySqlPool;

use super::api_token::require_api_token;
use crate::auth_bearer::AuthUser;

pub use crate::api::list::{ListRecorderResponse, RecorderItem};

#[derive(Deserialize)]
pub struct ListRecorderQuery {
    pub token: Option<String>,
}

pub async fn list_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<ListRecorderQuery>,
) -> Result<Json<ListRecorderResponse>, StatusCode> {
    let user_id = require_api_token(&pool, params.token.as_deref()).await?;

    Ok(crate::api::list::list_recorder(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
    )
    .await)
}
