use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use sqlx::mysql::MySqlPool;

use super::api_token::{
    PERM_CHANGE_STATUS, PERM_MODIFY_RECORD, PERM_WRITE, require_token_with_perm,
};
use crate::auth_bearer::AuthUser;

pub use crate::api::update_recorder::UpdateRecorderResponse;

#[derive(Deserialize)]
pub struct UpdateRecorderQuery {
    pub bangumi_id: Option<i32>,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub imdb_id: Option<String>,
    pub recorder: Option<String>,
    pub user_status: Option<i32>,
    pub token: Option<String>,
}

pub async fn update_user_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<UpdateRecorderQuery>,
) -> Result<Json<UpdateRecorderResponse>, StatusCode> {
    let token_info = require_token_with_perm(
        &pool,
        params.token.as_deref(),
        &[PERM_MODIFY_RECORD, PERM_WRITE, PERM_CHANGE_STATUS],
    )
    .await?;
    let user_id = token_info.user_id;

    Ok(crate::api::update_recorder::update_user_recorder(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
        axum::Json(crate::api::update_recorder::UpdateRecorderQuery {
            bangumi_id: params.bangumi_id,
            source: params.source,
            external_id: params.external_id,
            imdb_id: params.imdb_id,
            recorder: params.recorder,
            user_status: params.user_status,
        }),
    )
    .await)
}
