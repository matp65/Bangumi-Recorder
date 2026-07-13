use axum::{
    Json,
    extract::{Extension, Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Serialize;
use sqlx::mysql::MySqlPool;

use crate::api::logs::{
    LogListData, LogListQuery, RecordingLogItem, SystemLogItem, list_recording_logs,
    list_system_logs, operation_metadata, write_system_log,
};
use crate::api::open::api_token::{
    PERM_READ_LOGS, api_token_from_request, require_token_with_perm,
};
use crate::api::v2::response::{ApiResponse, forbidden, unauthorized};
use crate::auth_bearer::AuthUser;

#[derive(serde::Deserialize)]
pub struct OpenLogListQuery {
    pub token: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub target: Option<String>,
    pub action: Option<String>,
    pub category: Option<String>,
    pub username: Option<String>,
}

fn handle_auth_error<T: Serialize>(e: StatusCode) -> (StatusCode, Json<ApiResponse<T>>) {
    match e {
        StatusCode::UNAUTHORIZED => unauthorized("Invalid API token"),
        StatusCode::FORBIDDEN => forbidden("Insufficient permissions"),
        _ => unauthorized("Invalid API token"),
    }
}

pub async fn list_recording_logs_open(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Query(query): Query<OpenLogListQuery>,
) -> (StatusCode, Json<ApiResponse<LogListData<RecordingLogItem>>>) {
    let token = api_token_from_request(&headers, query.token.as_deref());
    let token_info = match require_token_with_perm(&pool, token, &[PERM_READ_LOGS]).await {
        Ok(info) => info,
        Err(e) => return handle_auth_error(e),
    };

    write_system_log(
        &pool,
        "info",
        "logs",
        "recording_logs_read",
        "OpenAPI token read recording logs",
        Some(token_info.user_id),
        Some(operation_metadata(
            &headers,
            "API Token",
            serde_json::json!({}),
        )),
    )
    .await;

    list_recording_logs(
        State(pool),
        Extension(AuthUser {
            user_id: token_info.user_id,
        }),
        Query(LogListQuery {
            page: query.page,
            page_size: query.page_size,
            start_time: query.start_time,
            end_time: query.end_time,
            target: query.target,
            action: query.action,
            category: None,
            username: None,
        }),
    )
    .await
}

pub async fn list_system_logs_open(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Query(query): Query<OpenLogListQuery>,
) -> (StatusCode, Json<ApiResponse<LogListData<SystemLogItem>>>) {
    let token = api_token_from_request(&headers, query.token.as_deref());
    let token_info = match require_token_with_perm(&pool, token, &[PERM_READ_LOGS]).await {
        Ok(info) => info,
        Err(e) => return handle_auth_error(e),
    };

    write_system_log(
        &pool,
        "info",
        "logs",
        "system_logs_read",
        "OpenAPI token read system logs",
        Some(token_info.user_id),
        Some(operation_metadata(
            &headers,
            "API Token",
            serde_json::json!({}),
        )),
    )
    .await;

    list_system_logs(
        State(pool),
        Extension(AuthUser {
            user_id: token_info.user_id,
        }),
        Query(LogListQuery {
            page: query.page,
            page_size: query.page_size,
            start_time: query.start_time,
            end_time: query.end_time,
            target: None,
            action: query.action,
            category: query.category,
            username: query.username,
        }),
    )
    .await
}
