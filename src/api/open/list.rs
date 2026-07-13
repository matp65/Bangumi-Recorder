use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;

use sqlx::mysql::MySqlPool;

use super::api_token::{PERM_READ, PERM_WRITE, api_token_from_request, require_token_with_perm};
use crate::auth_bearer::AuthUser;

pub use crate::api::list::{ListRecorderResponse, RecorderItem};

#[derive(Deserialize)]
pub struct ListRecorderQuery {
    pub token: Option<String>,
}

pub async fn list_recorder(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Query(params): Query<ListRecorderQuery>,
) -> Result<Json<ListRecorderResponse>, StatusCode> {
    let token = api_token_from_request(&headers, params.token.as_deref());
    let token_info = require_token_with_perm(&pool, token, &[PERM_READ, PERM_WRITE]).await?;
    let user_id = token_info.user_id;

    Ok(
        crate::api::list::list_recorder(
            State(pool),
            axum::extract::Extension(AuthUser { user_id }),
        )
        .await,
    )
}
