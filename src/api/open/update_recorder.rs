use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use sqlx::mysql::MySqlPool;

use super::api_token::require_api_token;
use crate::auth_bearer::AuthUser;

pub use crate::api::update_recorder::UpdateRecorderResponse;

#[derive(Deserialize)]
pub struct UpdateRecorderQuery {
    pub bangumi_id: Option<i32>,
    pub recorder: Option<String>,
    pub user_status: Option<i32>,
    pub token: Option<String>,
}

pub async fn update_user_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<UpdateRecorderQuery>,
) -> Result<Json<UpdateRecorderResponse>, StatusCode> {
    let user_id = require_api_token(&pool, params.token.as_deref()).await?;

    Ok(crate::api::update_recorder::update_user_recorder(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
        axum::Json(crate::api::update_recorder::UpdateRecorderQuery {
            bangumi_id: params.bangumi_id,
            recorder: params.recorder,
            user_status: params.user_status,
        }),
    )
    .await)
}
