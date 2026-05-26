use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use sqlx::mysql::MySqlPool;

use super::api_token::{require_token_with_perm, PERM_READ, PERM_WRITE};
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
    let token_info = require_token_with_perm(&pool, params.token.as_deref(), &[PERM_READ, PERM_WRITE]).await?;
    let user_id = token_info.user_id;

    Ok(crate::api::list::list_recorder(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
    )
    .await)
}
