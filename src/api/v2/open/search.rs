use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

use crate::api::open::api_token::{require_token_with_perm, PERM_READ, PERM_WRITE};
use crate::api::v2::search::{
    search_bangumi as v2_search_bangumi,
    get_bangumi as v2_get_bangumi,
    search_local as v2_search_local,
};
use crate::api::v2::response::{unauthorized as v2_unauthorized, forbidden as v2_forbidden, ApiResponse};

pub use crate::api::search::BangumiItem;
pub use crate::api::v2::search::LocalSearchResult;
pub use crate::api::v2::search::BangumiEpisodeMeta;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub page: Option<i32>,
    pub token: Option<String>,
}

#[derive(Deserialize)]
pub struct BangumiQuery {
    pub force: Option<bool>,
    pub token: Option<String>,
}

#[derive(Deserialize)]
pub struct LocalSearchParams {
    pub q: Option<String>,
    pub id: Option<u32>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub token: Option<String>,
}

async fn verify_token<T: Serialize>(pool: &MySqlPool, token: Option<&str>) -> Result<(), (StatusCode, Json<ApiResponse<T>>)> {
    match require_token_with_perm(pool, token, &[PERM_READ, PERM_WRITE]).await {
        Ok(_) => Ok(()),
        Err(StatusCode::UNAUTHORIZED) => Err(v2_unauthorized("Invalid API token")),
        Err(StatusCode::FORBIDDEN) => Err(v2_forbidden("Insufficient permissions")),
        Err(_) => Err(v2_unauthorized("Invalid API token")),
    }
}

/// GET /api/v2/open/search?q=keyword&page=1&force=true&token=xxx
pub async fn search_bangumi(
    State(pool): State<MySqlPool>,
    Query(params): Query<SearchQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<crate::api::search::BangumiSearchItem>>>) {
    if let Err(e) = verify_token(&pool, params.token.as_deref()).await {
        return e;
    }

    v2_search_bangumi(
        State(pool),
        Query(crate::api::v2::search::SearchQuery {
            q: params.q,
            page: params.page,
        }),
    )
    .await
}

/// GET /api/v2/open/bangumi/:id?force=true&token=xxx
pub async fn get_bangumi(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Query(params): Query<BangumiQuery>,
) -> (StatusCode, Json<ApiResponse<BangumiItem>>) {
    if let Err(e) = verify_token(&pool, params.token.as_deref()).await {
        return e;
    }

    v2_get_bangumi(
        State(pool),
        Path(id),
        Query(crate::api::v2::search::BangumiQuery {
            force: params.force,
        }),
    )
    .await
}

#[derive(Deserialize)]
pub struct EpisodeListQuery {
    pub token: Option<String>,
}

/// GET /api/v2/open/bangumi/:id/episodes?token=xxx
pub async fn get_bangumi_episodes(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Query(params): Query<EpisodeListQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<BangumiEpisodeMeta>>>) {
    if let Err(e) = verify_token(&pool, params.token.as_deref()).await {
        return e;
    }

    crate::api::v2::search::get_bangumi_episodes(
        State(pool),
        Path(id),
    )
    .await
}

/// GET /api/v2/open/search/local?q=keyword&token=xxx
pub async fn search_local(
    State(pool): State<MySqlPool>,
    Query(params): Query<LocalSearchParams>,
) -> (StatusCode, Json<ApiResponse<LocalSearchResult>>) {
    if let Err(e) = verify_token(&pool, params.token.as_deref()).await {
        return e;
    }

    v2_search_local(
        State(pool),
        Query(crate::api::v2::search::LocalSearchParams {
            q: params.q,
            id: params.id,
            page: params.page,
            page_size: params.page_size,
        }),
    )
    .await
}
