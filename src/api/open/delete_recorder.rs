use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::{PERM_DELETE_RECORD, PERM_WRITE, require_token_with_perm};
use crate::auth_bearer::AuthUser;

pub use crate::api::delete_recorder::DeleteRecorderResponse;

#[derive(Deserialize)]
pub struct DeleteRecorderQuery {
    pub bangumi_id: Option<u32>,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub imdb_id: Option<String>,
    pub other_id: Option<u32>,
    pub local_other_id: Option<u32>,
    pub hard_delete: Option<bool>,
    pub token: Option<String>,
}

pub async fn delete_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<DeleteRecorderQuery>,
) -> Result<Json<DeleteRecorderResponse>, StatusCode> {
    let token_info = require_token_with_perm(
        &pool,
        params.token.as_deref(),
        &[PERM_DELETE_RECORD, PERM_WRITE],
    )
    .await?;
    let user_id = token_info.user_id;

    Ok(crate::api::delete_recorder::delete_recorder(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
        axum::Json(crate::api::delete_recorder::DeleteRecorderQuery {
            bangumi_id: params.bangumi_id,
            source: params.source,
            external_id: params.external_id,
            imdb_id: params.imdb_id,
            other_id: params.other_id,
            local_other_id: params.local_other_id,
            hard_delete: params.hard_delete,
        }),
    )
    .await)
}
