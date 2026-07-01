use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use super::api_token::{PERM_READ, PERM_WRITE, require_token_with_perm};
use crate::auth_bearer::AuthUser;

pub use crate::api::get_recorder::GetRecorderResponse;

#[derive(Deserialize)]
pub struct GetRecorderQuery {
    pub bangumi_id: Option<u32>,
    pub imdb_id: Option<String>,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub local_bangumi_id: Option<u32>,
    pub local_external_media_id: Option<u32>,
    pub other_id: Option<u32>,
    pub token: Option<String>,
}

pub async fn get_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<GetRecorderQuery>,
) -> Result<Json<GetRecorderResponse>, StatusCode> {
    let token_info =
        require_token_with_perm(&pool, params.token.as_deref(), &[PERM_READ, PERM_WRITE]).await?;
    let user_id = token_info.user_id;

    Ok(crate::api::get_recorder::get_recorder(
        State(pool),
        axum::extract::Extension(AuthUser { user_id }),
        axum::Json(crate::api::get_recorder::GetRecorderQuery {
            bangumi_id: params.bangumi_id,
            imdb_id: params.imdb_id,
            source: params.source,
            external_id: params.external_id,
            local_bangumi_id: params.local_bangumi_id,
            local_external_media_id: params.local_external_media_id,
            other_id: params.other_id,
        }),
    )
    .await)
}
