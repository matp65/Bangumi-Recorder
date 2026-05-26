use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use sqlx::mysql::MySqlPool;

use super::api_token::{require_token_with_perm, PERM_MODIFY_RECORD, PERM_CHANGE_STATUS, PERM_WRITE};
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
    let token_info = require_token_with_perm(&pool, params.token.as_deref(), &[PERM_MODIFY_RECORD, PERM_WRITE, PERM_CHANGE_STATUS]).await?;
    let user_id = token_info.user_id;

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
