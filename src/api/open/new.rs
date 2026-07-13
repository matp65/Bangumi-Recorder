use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::{
    PERM_ADD_RECORD, PERM_WRITE, api_token_from_request, require_token_with_perm,
};
use crate::auth_bearer::AuthUser;

pub use crate::api::new::AddRecordResponse;

#[derive(Debug, Deserialize)]
pub struct AddRecordQuery {
    pub bangumi_id: Option<u32>,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub imdb_id: Option<String>,
    pub use_api: Option<bool>,
    pub other_id: Option<u32>,
    pub other_title: Option<String>,
    pub other_description: Option<String>,
    pub other_cover: Option<String>,
    pub other_max_number: Option<i32>,
    pub other_status: Option<i32>,
    pub user_status: Option<i32>,
    pub recorder: Option<String>,
    pub token: Option<String>,
}

pub async fn add_record_open(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Query(params): Query<AddRecordQuery>,
) -> Result<Json<AddRecordResponse>, StatusCode> {
    let token = api_token_from_request(&headers, params.token.as_deref());
    let token_info = require_token_with_perm(&pool, token, &[PERM_ADD_RECORD, PERM_WRITE]).await?;
    let user_id = token_info.user_id;

    Ok(crate::api::new::add_record(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
        axum::Json(crate::api::new::AddRecordQuery {
            bangumi_id: params.bangumi_id,
            source: params.source,
            external_id: params.external_id,
            imdb_id: params.imdb_id,
            use_api: params.use_api,
            other_id: params.other_id,
            other_title: params.other_title,
            other_description: params.other_description,
            other_cover: params.other_cover,
            other_max_number: params.other_max_number,
            other_status: params.other_status,
            user_status: params.user_status,
            recorder: params.recorder,
        }),
    )
    .await)
}
