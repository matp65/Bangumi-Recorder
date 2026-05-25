use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::require_api_token;
use crate::auth_bearer::AuthUser;

pub use crate::api::new::AddRecordResponse;

#[derive(Debug, Deserialize)]
pub struct AddRecordQuery {
    pub bangumi_id: Option<u32>,
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
    Query(params): Query<AddRecordQuery>,
) -> Result<Json<AddRecordResponse>, StatusCode> {
    let user_id = require_api_token(&pool, params.token.as_deref()).await?;

    Ok(crate::api::new::add_record(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
        axum::Json(crate::api::new::AddRecordQuery {
            bangumi_id: params.bangumi_id,
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
