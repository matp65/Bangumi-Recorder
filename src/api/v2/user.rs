use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use sqlx::mysql::MySqlPool;

use crate::auth_bearer::AuthUser;
use crate::api::user::{
    UserInfo, UpdateUserInfo, UpdatePasswordRequest,
    get_info as v1_get_info,
    update_info as v1_update_info,
    update_password as v1_update_password,
};
use super::response::{success, success_empty, bad_request, internal_error, ApiResponse};

#[derive(Serialize)]
pub struct TokenData {
    pub api_token: String,
}

pub async fn get_info(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<UserInfo>>) {
    let v1_resp = v1_get_info(State(pool.clone()), Extension(auth_user)).await;
    success(v1_resp.0)
}

pub async fn update_info(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<UpdateUserInfo>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let v1_resp = v1_update_info(State(pool.clone()), Extension(auth_user), Json(payload)).await;
    let inner = v1_resp.0;
    if inner.status == 0 {
        success_empty()
    } else {
        internal_error(inner.message.as_deref().unwrap_or("Update failed"))
    }
}

pub async fn update_password(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let v1_resp = v1_update_password(State(pool.clone()), Extension(auth_user), Json(payload)).await;
    let inner = v1_resp.0;
    if inner.status == 0 {
        success_empty()
    } else {
        bad_request(inner.message.as_deref().unwrap_or("Password update failed"))
    }
}

pub async fn regenerate_api_token(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<TokenData>>) {
    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    match sqlx::query!(
        "UPDATE users SET api_token_hash = ? WHERE id = ?",
        token_hash,
        auth_user.user_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => success(TokenData { api_token: raw_token }),
        Err(e) => {
            log::error!("Failed to regenerate token: {:?}", e);
            internal_error("Failed to regenerate token")
        }
    }
}
