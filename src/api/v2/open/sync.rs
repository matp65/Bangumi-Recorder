use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use sqlx::mysql::MySqlPool;
use chrono::NaiveDateTime;

use crate::api::open::api_token::{require_token_with_perm, PERM_READ, PERM_WRITE};
use crate::api::v2::sync::{SyncRequestBody, SyncResponseData, SyncResponseRecord};
use crate::api::v2::response::{unauthorized, ApiResponse};

#[derive(serde::Deserialize)]
pub struct OpenSyncQuery {
    pub token: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct OpenSyncSinceQuery {
    pub token: Option<String>,
    pub since: Option<NaiveDateTime>,
}

fn forbidden<T: Serialize>(msg: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::FORBIDDEN, Json(ApiResponse {
        status: -1,
        data: None,
        message: Some(msg.to_string()),
    }))
}

fn bad_request<T: Serialize>(msg: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::BAD_REQUEST, Json(ApiResponse {
        status: -1,
        data: None,
        message: Some(msg.to_string()),
    }))
}

pub async fn sync_records(
    State(pool): State<MySqlPool>,
    Query(query): Query<OpenSyncQuery>,
    Json(body): Json<SyncRequestBody>,
) -> (StatusCode, Json<ApiResponse<SyncResponseData>>) {
    let token = match query.token.as_deref() {
        Some(t) => t,
        None => return unauthorized("Missing API token"),
    };

    let token_info = match require_token_with_perm(&pool, Some(token), &[PERM_READ, PERM_WRITE]).await {
        Ok(info) => info,
        Err(StatusCode::UNAUTHORIZED) => return unauthorized("Invalid API token"),
        Err(_) => return forbidden("Insufficient permissions"),
    };

    crate::api::v2::sync::do_sync_records(&pool, token_info.user_id, body).await
}

pub async fn incremental_sync(
    State(pool): State<MySqlPool>,
    Query(query): Query<OpenSyncSinceQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<SyncResponseRecord>>>) {
    let token = match query.token.as_deref() {
        Some(t) => t,
        None => return unauthorized("Missing API token"),
    };

    let token_info = match require_token_with_perm(&pool, Some(token), &[PERM_READ]).await {
        Ok(info) => info,
        Err(StatusCode::UNAUTHORIZED) => return unauthorized("Invalid API token"),
        Err(_) => return forbidden("Insufficient permissions"),
    };

    let since = match query.since {
        Some(s) => s,
        None => return bad_request("Missing 'since' query parameter"),
    };

    crate::api::v2::sync::do_incremental_sync_records(&pool, token_info.user_id, since).await
}
