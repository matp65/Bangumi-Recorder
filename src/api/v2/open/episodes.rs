use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use sqlx::mysql::MySqlPool;

use crate::api::open::api_token::{require_token_with_perm, PERM_READ, PERM_WRITE};
use crate::auth_bearer::AuthUser;
use crate::api::v2::episodes::{EpisodeItem, UpdateEpisodeBody, ForceEpisodesQuery};
use crate::api::v2::response::{unauthorized, ApiResponse};

#[derive(serde::Deserialize)]
pub struct OpenTokenQuery {
    pub token: Option<String>,
    pub force: Option<bool>,
}

fn forbidden<T: Serialize>(msg: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::FORBIDDEN, Json(ApiResponse {
        status: -1,
        data: None,
        message: Some(msg.to_string()),
    }))
}

pub async fn list_episodes(
    State(pool): State<MySqlPool>,
    Path(bangumi_id): Path<u32>,
    Query(query): Query<OpenTokenQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<EpisodeItem>>>) {
    let token = match query.token.as_deref() {
        Some(t) => t,
        None => return unauthorized("Missing API token"),
    };

    let token_info = match require_token_with_perm(&pool, Some(token), &[PERM_READ, PERM_WRITE]).await {
        Ok(info) => info,
        Err(StatusCode::UNAUTHORIZED) => return unauthorized("Invalid API token"),
        Err(_) => return forbidden("Insufficient permissions"),
    };

    crate::api::v2::episodes::list_episodes(
        State(pool),
        Extension(AuthUser { user_id: token_info.user_id }),
        Path(bangumi_id),
        Query(ForceEpisodesQuery { force: query.force }),
    )
    .await
}

pub async fn update_episode(
    State(pool): State<MySqlPool>,
    Path((bangumi_id, ordinal)): Path<(u32, i32)>,
    Query(query): Query<OpenTokenQuery>,
    Json(body): Json<UpdateEpisodeBody>,
) -> (StatusCode, Json<ApiResponse<EpisodeItem>>) {
    let token = match query.token.as_deref() {
        Some(t) => t,
        None => return unauthorized("Missing API token"),
    };

    let token_info = match require_token_with_perm(&pool, Some(token), &[PERM_WRITE]).await {
        Ok(info) => info,
        Err(StatusCode::UNAUTHORIZED) => return unauthorized("Invalid API token"),
        Err(_) => return forbidden("Insufficient permissions"),
    };

    crate::api::v2::episodes::update_episode(
        State(pool),
        Extension(AuthUser { user_id: token_info.user_id }),
        Path((bangumi_id, ordinal)),
        Json(body),
    )
    .await
}
