use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::{Duration, Utc, NaiveDate};

use crate::api::search::{
    TitleSearchQuery, IDSearchQuery,
    BangumiSearchItem, LocalSearchItem,
    search_bangumi_by_title as v1_search_title,
    search_bangumi_by_id as v1_search_by_id,
    search_local as v1_search_local,
};
use super::response::{success, not_found, internal_error, bad_request, ApiResponse};
use super::episodes::parse_prg_list_episodes;

pub use crate::api::search::BangumiItem;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub page: Option<i32>,
}

#[derive(Deserialize)]
pub struct BangumiQuery {
    pub force: Option<bool>,
}

#[derive(Deserialize)]
pub struct LocalSearchParams {
    pub q: Option<String>,
    pub id: Option<u32>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Serialize)]
pub struct LocalSearchResult {
    pub items: Vec<LocalSearchItem>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// GET /api/v2/search?q=keyword&page=1
///
/// Forwards to v1 which scrapes bgm.tv search results and saves them to the
/// cache (bangumi_info_easy).  The search page scrape is lightweight; the
/// heavy detail scrape is cached via [`get_bangumi`].
pub async fn search_bangumi(
    State(pool): State<MySqlPool>,
    Query(params): Query<SearchQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<BangumiSearchItem>>>) {
    let title = match &params.q {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => return bad_request("Missing search query"),
    };

    let v1_resp = v1_search_title(
        State(pool.clone()),
        Json(TitleSearchQuery {
            title: Some(title),
            page: params.page,
        }),
    )
    .await;

    let inner = v1_resp.0;
    if inner.status == 0 {
        success(inner.data.unwrap_or_default())
    } else {
        internal_error("Search failed")
    }
}

/// GET /api/v2/bangumi/:id?force=true
///
/// Returns cached result from bangumi_info_detailed if the data was last
/// updated within 24 hours, avoiding a scrape of bgm.tv.  Pass `?force=true`
/// to skip the cache and force a fresh scrape.
pub async fn get_bangumi(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Query(query): Query<BangumiQuery>,
) -> (StatusCode, Json<ApiResponse<BangumiItem>>) {
    let force = query.force.unwrap_or(false);
    let id_str = id.to_string();

    if !force {
        if let Ok(Some(row)) = sqlx::query!(
            r#"
            SELECT b.external_id, b.title, b.cover_url, b.type              AS "type?: i8",
                   d.author, d.release_date, d.episodes, d.description,
                   d.updated_at                                             AS "updated_at?"
            FROM bangumi_info_easy b
            LEFT JOIN bangumi_info_detailed d ON d.bangumi_id = b.id
            WHERE b.external_id = ?
            "#,
            id_str
        )
        .fetch_optional(&pool)
        .await
        {
            if let Some(ts) = row.updated_at {
                if Utc::now().naive_utc() <= ts + Duration::hours(24) {
                    return success(BangumiItem {
                        bangumi_id: row.external_id,
                        title: row.title,
                        cover_url: row.cover_url.unwrap_or_default(),
                        r#type: row.r#type.unwrap_or(8),
                        author: row.author.unwrap_or_default(),
                        release_date: row.release_date,
                        episodes: row.episodes.unwrap_or(0),
                        description: row.description.unwrap_or_default(),
                    });
                }
            }
        }
    }

    let v1_resp = v1_search_by_id(
        State(pool.clone()),
        Json(IDSearchQuery { id: Some(id) }),
    )
    .await;

    let inner = v1_resp.0;
    match inner.status {
        0 => match inner.data {
            Some(item) => success(item),
            None => not_found("Bangumi not found"),
        },
        _ => internal_error("Search failed"),
    }
}

#[derive(Serialize)]
pub struct BangumiEpisodeMeta {
    pub ordinal: i32,
    pub title: Option<String>,
    pub name_cn: Option<String>,
    pub airdate: Option<NaiveDate>,
    pub duration: Option<String>,
}

/// GET /api/v2/bangumi/:id/episodes
///
/// Returns the episode list for a bangumi subject, scraped from bgm.tv and
/// cached in the bangumi_episodes table.
pub async fn get_bangumi_episodes(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
) -> (StatusCode, Json<ApiResponse<Vec<BangumiEpisodeMeta>>>) {
    let id_str = id.to_string();

    let easy_id = match sqlx::query_scalar!(
        "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
        id_str
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(eid)) => eid,
        Ok(None) => return not_found("Bangumi not found in cache. Search first."),
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Database error");
        }
    };

    let cached = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM bangumi_episodes WHERE bangumi_easy_id = ?",
        easy_id
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    if cached == 0 {
        if let Err(_) = scrape_and_cache_episodes(&pool, easy_id, &id_str).await {
            return internal_error("Failed to scrape episode data");
        }
    }

    let rows = sqlx::query!(
        r#"
        SELECT ordinal, title, name_cn, airdate, duration
        FROM bangumi_episodes
        WHERE bangumi_easy_id = ?
        ORDER BY ordinal ASC
        "#,
        easy_id
    )
    .fetch_all(&pool)
    .await;

    match rows {
        Ok(rows) => {
            let episodes: Vec<BangumiEpisodeMeta> = rows
                .into_iter()
                .map(|r| BangumiEpisodeMeta {
                    ordinal: r.ordinal,
                    title: r.title,
                    name_cn: r.name_cn,
                    airdate: r.airdate,
                    duration: r.duration,
                })
                .collect();
            success(episodes)
        }
        Err(e) => {
            log::error!("DB error: {:?}", e);
            internal_error("Database error")
        }
    }
}

/// Scrape bgm.tv episode list and cache into bangumi_episodes.
/// Used by both the JWT and open endpoints.
pub async fn scrape_and_cache_episodes(
    pool: &MySqlPool,
    easy_id: u32,
    bangumi_id: &str,
) -> Result<(), ()> {
    let url = format!("https://bgm.tv/subject/{}", bangumi_id);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|_| ())?;

    let html = resp.text().await.map_err(|_| ())?;

    let episodes = parse_prg_list_episodes(&html);

    for ep in episodes {
        let _ = sqlx::query!(
            r#"
            INSERT INTO bangumi_episodes (bangumi_easy_id, ordinal, title, name_cn, airdate, duration)
            VALUES (?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                title = VALUES(title),
                name_cn = VALUES(name_cn),
                airdate = VALUES(airdate),
                duration = VALUES(duration)
            "#,
            easy_id,
            ep.ordinal,
            ep.title,
            ep.name_cn,
            ep.airdate,
            ep.duration
        )
        .execute(pool)
        .await;
    }

    Ok(())
}

/// GET /api/v2/search/local?q=keyword&page=1&page_size=20
pub async fn search_local(
    State(pool): State<MySqlPool>,
    Query(params): Query<LocalSearchParams>,
) -> (StatusCode, Json<ApiResponse<LocalSearchResult>>) {
    let v1_resp = v1_search_local(
        State(pool.clone()),
        Json(crate::api::search::LocalSearchQuery {
            keyword: params.q,
            id: params.id,
            page: params.page,
            page_size: params.page_size,
        }),
    )
    .await;

    let inner = v1_resp.0;
    if inner.status == 0 {
        success(LocalSearchResult {
            items: inner.data.unwrap_or_default(),
            total: inner.total.unwrap_or(0),
            page: inner.page.unwrap_or(1),
            page_size: inner.page_size.unwrap_or(20),
        })
    } else {
        internal_error("Search failed")
    }
}
