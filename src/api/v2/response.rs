use axum::{Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: i32,
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// --- success: HTTP 200 ---

pub fn success<T: Serialize>(data: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::OK,
        Json(ApiResponse {
            status: 0,
            data: Some(data),
            message: None,
        }),
    )
}

pub fn success_with_message<T: Serialize>(
    data: T,
    message: impl Into<String>,
) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::OK,
        Json(ApiResponse {
            status: 0,
            data: Some(data),
            message: Some(message.into()),
        }),
    )
}

pub fn success_empty() -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::OK,
        Json(ApiResponse {
            status: 0,
            data: None,
            message: None,
        }),
    )
}

// --- error: custom HTTP status + -1 body ---

pub fn err_with_status<T: Serialize>(
    status: StatusCode,
    message: &str,
) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        status,
        Json(ApiResponse {
            status: -1,
            data: None,
            message: Some(message.to_string()),
        }),
    )
}

pub fn bad_request<T: Serialize>(message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    err_with_status(StatusCode::BAD_REQUEST, message)
}

pub fn not_found<T: Serialize>(message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    err_with_status(StatusCode::NOT_FOUND, message)
}

pub fn conflict<T: Serialize>(message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    err_with_status(StatusCode::CONFLICT, message)
}

pub fn internal_error<T: Serialize>(message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    err_with_status(StatusCode::INTERNAL_SERVER_ERROR, message)
}

pub fn unauthorized<T: Serialize>(message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    err_with_status(StatusCode::UNAUTHORIZED, message)
}

pub fn forbidden<T: Serialize>(message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    err_with_status(StatusCode::FORBIDDEN, message)
}
