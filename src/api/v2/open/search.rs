use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;

use crate::api::open::api_token::require_api_token;
use crate::api::v2::search::{
    search_bangumi as v2_search_bangumi,
    get_bangumi as v2_get_bangumi,
    search_local as v2_search_local,
};
use crate::api::v2::response::{unauthorized as v2_unauthorized, ApiResponse};

pub use crate::api::search::BangumiItem;
pub use crate::api::v2::search::LocalSearchResult;

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

/// GET /api/v2/open/search?q=keyword&page=1&force=true&token=xxx
pub async fn search_bangumi(
    State(pool): State<MySqlPool>,
    Query(params): Query<SearchQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<crate::api::search::BangumiSearchItem>>>) {
    let _uid = match require_api_token(&pool, params.token.as_deref()).await {
        Ok(uid) => uid,
        Err(_) => return v2_unauthorized("Invalid API token"),
    };

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
    let _uid = match require_api_token(&pool, params.token.as_deref()).await {
        Ok(uid) => uid,
        Err(_) => return v2_unauthorized("Invalid API token"),
    };

    v2_get_bangumi(
        State(pool),
        Path(id),
        Query(crate::api::v2::search::BangumiQuery {
            force: params.force,
        }),
    )
    .await
}

/// GET /api/v2/open/search/local?q=keyword&token=xxx
pub async fn search_local(
    State(pool): State<MySqlPool>,
    Query(params): Query<LocalSearchParams>,
) -> (StatusCode, Json<ApiResponse<LocalSearchResult>>) {
    let _uid = match require_api_token(&pool, params.token.as_deref()).await {
        Ok(uid) => uid,
        Err(_) => return v2_unauthorized("Invalid API token"),
    };

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
