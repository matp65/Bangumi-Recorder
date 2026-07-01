use axum::{Json, extract::State};
use chrono::{Duration, NaiveDate, Utc};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{QueryBuilder, mysql::MySqlPool};
use urlencoding::encode;

pub const IMDB_SOURCE: &str = "imdb";

#[derive(Debug, Serialize, Deserialize)]
pub struct ImdbSearchQuery {
    pub q: Option<String>,
    pub page: Option<i32>,
    pub use_api: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ImdbSearchItem {
    pub imdb_id: String,
    pub external_id: String,
    pub title: String,
    pub year: Option<String>,
    pub cover: Option<String>,
    pub info: String,
    pub r#type: i32,
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct SearchImdbResponse {
    pub status: i32,
    pub data: Option<Vec<ImdbSearchItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImdbIDSearchQuery {
    pub id: Option<String>,
    pub force: Option<bool>,
    pub use_api: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ImdbIDSearchResponse {
    pub status: i32,
    pub data: Option<ImdbItem>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ImdbItem {
    pub imdb_id: String,
    pub external_id: String,
    pub title: String,
    pub cover_url: String,
    pub r#type: i8,
    pub author: String,
    pub release_date: Option<NaiveDate>,
    pub episodes: i32,
    pub description: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
struct SuggestionResponse {
    d: Option<Vec<SuggestionItem>>,
}

#[derive(Debug, Deserialize)]
struct SuggestionItem {
    id: Option<String>,
    l: Option<String>,
    q: Option<String>,
    qid: Option<String>,
    s: Option<String>,
    y: Option<i32>,
    i: Option<SuggestionImage>,
}

#[derive(Debug, Deserialize)]
struct SuggestionImage {
    #[serde(rename = "imageUrl")]
    image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OmdbSearchResponse {
    #[serde(rename = "Search")]
    search: Option<Vec<OmdbSearchItem>>,
    #[serde(rename = "Response")]
    response: String,
}

#[derive(Debug, Deserialize)]
struct OmdbSearchItem {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Year")]
    year: Option<String>,
    #[serde(rename = "imdbID")]
    imdb_id: String,
    #[serde(rename = "Type")]
    title_type: Option<String>,
    #[serde(rename = "Poster")]
    poster: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OmdbDetail {
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "Year")]
    year: Option<String>,
    #[serde(rename = "Released")]
    released: Option<String>,
    #[serde(rename = "Runtime")]
    runtime: Option<String>,
    #[serde(rename = "Genre")]
    genre: Option<String>,
    #[serde(rename = "Director")]
    director: Option<String>,
    #[serde(rename = "Writer")]
    writer: Option<String>,
    #[serde(rename = "Plot")]
    plot: Option<String>,
    #[serde(rename = "Poster")]
    poster: Option<String>,
    #[serde(rename = "Type")]
    title_type: Option<String>,
    #[serde(rename = "totalSeasons")]
    total_seasons: Option<String>,
    #[serde(rename = "Response")]
    response: String,
}

pub fn normalize_imdb_id(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let candidate = if let Some(idx) = trimmed.find("/title/") {
        trimmed[idx + "/title/".len()..]
            .split('/')
            .next()
            .unwrap_or(trimmed)
    } else {
        trimmed
    };
    let candidate = candidate.trim();
    if candidate.len() >= 5
        && candidate.len() <= 14
        && candidate.starts_with("tt")
        && candidate[2..].chars().all(|c| c.is_ascii_digit())
    {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn should_use_api(requested: Option<bool>) -> bool {
    let mode = std::env::var("IMDB_SEARCH_MODE").unwrap_or_default();
    requested.unwrap_or_else(|| !mode.eq_ignore_ascii_case("no_api") && omdb_key().is_some())
        && omdb_key().is_some()
}

fn omdb_key() -> Option<String> {
    std::env::var("OMDB_API_KEY")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            std::env::var("IMDB_API_KEY")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
}

fn imdb_client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Bangumi-Recorder/1.0)")
        .build()
        .unwrap_or_else(|_| Client::new())
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("N/A") {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn type_from_imdb(value: Option<&str>) -> i8 {
    let value = value.unwrap_or_default().to_ascii_lowercase();
    if value.contains("game") {
        9
    } else if value.contains("series") || value.contains("tv") {
        1
    } else if value.contains("movie") || value.contains("episode") || value.contains("video") {
        2
    } else {
        8
    }
}

fn info_for_search(kind: Option<&str>, year: Option<&str>) -> String {
    match (kind, year) {
        (Some(kind), Some(year)) if !kind.is_empty() && !year.is_empty() => {
            format!("{} · {}", kind, year)
        }
        (Some(kind), _) if !kind.is_empty() => kind.to_string(),
        (_, Some(year)) if !year.is_empty() => year.to_string(),
        _ => String::new(),
    }
}

fn parse_release_date(value: Option<&str>, year: Option<&str>) -> Option<NaiveDate> {
    if let Some(value) = value {
        let value = value.trim();
        if !value.is_empty() && !value.eq_ignore_ascii_case("N/A") {
            if let Ok(date) = NaiveDate::parse_from_str(value, "%d %b %Y") {
                return Some(date);
            }
            if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                return Some(date);
            }
            if let Ok(date) = NaiveDate::parse_from_str(&format!("{}-01", value), "%Y-%m-%d") {
                return Some(date);
            }
        }
    }
    year.and_then(|y| y.get(0..4))
        .and_then(|y| y.parse::<i32>().ok())
        .and_then(|y| NaiveDate::from_ymd_opt(y, 1, 1))
}

fn suggestion_url(q: &str) -> String {
    let normalized = q
        .trim()
        .to_ascii_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
    let first = normalized.chars().next().unwrap_or('x');
    format!(
        "https://v3.sg.media-imdb.com/suggestion/{}/{}.json",
        first,
        encode(&normalized)
    )
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => clean_optional(Some(s.clone())),
        Value::Object(map) => map.get("name").and_then(value_to_string),
        Value::Array(items) => {
            let names: Vec<String> = items.iter().filter_map(value_to_string).collect();
            if names.is_empty() {
                None
            } else {
                Some(names.join(", "))
            }
        }
        _ => None,
    }
}

async fn upsert_imdb_search_results(pool: &MySqlPool, results: &[ImdbSearchItem]) {
    if results.is_empty() {
        return;
    }

    let mut qb = QueryBuilder::new(
        "INSERT INTO external_media (source, external_id, title, type, info, cover_url) ",
    );
    qb.push_values(results.iter(), |mut b, r| {
        b.push_bind(IMDB_SOURCE)
            .push_bind(r.imdb_id.as_str())
            .push_bind(r.title.as_str())
            .push_bind(r.r#type)
            .push_bind(r.info.as_str())
            .push_bind(r.cover.as_deref());
    });
    qb.push(
        " ON DUPLICATE KEY UPDATE title = VALUES(title), type = VALUES(type), info = VALUES(info), cover_url = VALUES(cover_url), updated_at = CURRENT_TIMESTAMP",
    );

    if let Err(e) = qb.build().execute(pool).await {
        log::error!(
            "batch upsert external_media IMDb search results error: {:?}",
            e
        );
    }
}

async fn upsert_imdb_item(pool: &MySqlPool, item: &ImdbItem) -> Result<u32, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO external_media (source, external_id, title, type, info, cover_url)
        VALUES (?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            title = VALUES(title),
            type = VALUES(type),
            info = VALUES(info),
            cover_url = VALUES(cover_url),
            updated_at = CURRENT_TIMESTAMP
        "#,
        IMDB_SOURCE,
        item.imdb_id.as_str(),
        item.title.as_str(),
        item.r#type,
        item.author.as_str(),
        item.cover_url.as_str()
    )
    .execute(pool)
    .await?;

    let easy_id = sqlx::query_scalar!(
        "SELECT id FROM external_media WHERE source = ? AND external_id = ?",
        IMDB_SOURCE,
        item.imdb_id.as_str()
    )
    .fetch_one(pool)
    .await?;

    match sqlx::query_scalar!(
        "SELECT id FROM external_media_detailed WHERE media_id = ? LIMIT 1",
        easy_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(_) => {
            sqlx::query!(
                r#"
                UPDATE external_media_detailed
                SET author = ?, release_date = ?, episodes = ?, description = ?, updated_at = CURRENT_TIMESTAMP
                WHERE media_id = ?
                "#,
                item.author.as_str(),
                item.release_date,
                item.episodes,
                item.description.as_str(),
                easy_id
            )
            .execute(pool)
            .await?;
        }
        None => {
            sqlx::query!(
                "INSERT INTO external_media_detailed (media_id, author, release_date, episodes, description) VALUES (?, ?, ?, ?, ?)",
                easy_id,
                item.author.as_str(),
                item.release_date,
                item.episodes,
                item.description.as_str(),
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(easy_id)
}

async fn cached_imdb_item(
    pool: &MySqlPool,
    imdb_id: &str,
) -> Result<Option<ImdbItem>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            m.external_id,
            m.title,
            m.cover_url,
            m.type AS "type!: i8",
            d.author,
            d.release_date,
            d.episodes,
            d.description,
            d.updated_at AS "updated_at?"
        FROM external_media m
        LEFT JOIN external_media_detailed d ON d.media_id = m.id
        WHERE m.source = ? AND m.external_id = ?
        "#,
        IMDB_SOURCE,
        imdb_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| ImdbItem {
        external_id: r.external_id.clone(),
        imdb_id: r.external_id,
        title: r.title,
        cover_url: r.cover_url.unwrap_or_default(),
        r#type: r.r#type,
        author: r.author.unwrap_or_default(),
        release_date: r.release_date,
        episodes: r.episodes.unwrap_or(0),
        description: r.description.unwrap_or_default(),
        source: IMDB_SOURCE.to_string(),
    }))
}

async fn search_imdb_no_api(q: &str) -> Result<Vec<ImdbSearchItem>, reqwest::Error> {
    let response: SuggestionResponse = imdb_client()
        .get(suggestion_url(q))
        .send()
        .await?
        .json()
        .await?;
    let results = response
        .d
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let imdb_id = normalize_imdb_id(item.id.as_deref()?)?;
            let title = clean_optional(item.l)?;
            let kind = clean_optional(item.q.or(item.qid));
            let year = item.y.map(|y| y.to_string());
            let cover = item.i.and_then(|i| clean_optional(i.image_url));
            let info = info_for_search(kind.as_deref(), year.as_deref());
            Some(ImdbSearchItem {
                external_id: imdb_id.clone(),
                imdb_id,
                title,
                year,
                cover,
                info,
                r#type: type_from_imdb(kind.as_deref()) as i32,
                source: IMDB_SOURCE.to_string(),
            })
        })
        .collect();

    Ok(results)
}

async fn fetch_imdb_detail_suggestion(imdb_id: &str) -> Result<Option<ImdbItem>, reqwest::Error> {
    let response: SuggestionResponse = imdb_client()
        .get(suggestion_url(imdb_id))
        .send()
        .await?
        .json()
        .await?;

    Ok(response.d.unwrap_or_default().into_iter().find_map(|item| {
        let item_id = normalize_imdb_id(item.id.as_deref()?)?;
        if item_id != imdb_id {
            return None;
        }

        let title = clean_optional(item.l)?;
        let kind = clean_optional(item.q.or(item.qid));
        let year = item.y.map(|y| y.to_string());
        let cover_url = item
            .i
            .and_then(|i| clean_optional(i.image_url))
            .unwrap_or_default();
        let author = clean_optional(item.s).unwrap_or_default();
        let description = info_for_search(kind.as_deref(), year.as_deref());

        Some(ImdbItem {
            imdb_id: imdb_id.to_string(),
            external_id: imdb_id.to_string(),
            title,
            cover_url,
            r#type: type_from_imdb(kind.as_deref()),
            author,
            release_date: parse_release_date(None, year.as_deref()),
            episodes: 0,
            description,
            source: IMDB_SOURCE.to_string(),
        })
    }))
}

async fn search_imdb_api(q: &str, page: i32) -> Result<Vec<ImdbSearchItem>, reqwest::Error> {
    let key = match omdb_key() {
        Some(key) => key,
        None => return Ok(Vec::new()),
    };
    let page = page.max(1);
    let url = format!(
        "https://www.omdbapi.com/?apikey={}&s={}&page={}",
        encode(&key),
        encode(q),
        page
    );
    let response: OmdbSearchResponse = imdb_client().get(url).send().await?.json().await?;
    if response.response.eq_ignore_ascii_case("False") {
        return Ok(Vec::new());
    }

    Ok(response
        .search
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let imdb_id = normalize_imdb_id(&item.imdb_id)?;
            let poster = clean_optional(item.poster).filter(|p| p != "N/A");
            let kind = clean_optional(item.title_type);
            let info = info_for_search(kind.as_deref(), item.year.as_deref());
            Some(ImdbSearchItem {
                external_id: imdb_id.clone(),
                imdb_id,
                title: item.title,
                year: item.year,
                cover: poster,
                info,
                r#type: type_from_imdb(kind.as_deref()) as i32,
                source: IMDB_SOURCE.to_string(),
            })
        })
        .collect())
}

async fn fetch_imdb_detail_api(imdb_id: &str) -> Result<Option<ImdbItem>, reqwest::Error> {
    let key = match omdb_key() {
        Some(key) => key,
        None => return Ok(None),
    };
    let url = format!(
        "https://www.omdbapi.com/?apikey={}&i={}&plot=full",
        encode(&key),
        imdb_id
    );
    let response: OmdbDetail = imdb_client().get(url).send().await?.json().await?;
    if response.response.eq_ignore_ascii_case("False") {
        return Ok(None);
    }

    let author = clean_optional(response.director)
        .or_else(|| clean_optional(response.writer))
        .or_else(|| clean_optional(response.genre))
        .unwrap_or_default();
    let episodes = response
        .total_seasons
        .as_deref()
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);
    let runtime = clean_optional(response.runtime).unwrap_or_default();
    let mut description = clean_optional(response.plot).unwrap_or_default();
    if !runtime.is_empty() {
        description = if description.is_empty() {
            runtime
        } else {
            format!("{} Runtime: {}", description, runtime)
        };
    }

    Ok(Some(ImdbItem {
        imdb_id: imdb_id.to_string(),
        external_id: imdb_id.to_string(),
        title: clean_optional(response.title).unwrap_or_else(|| imdb_id.to_string()),
        cover_url: clean_optional(response.poster).unwrap_or_default(),
        r#type: type_from_imdb(response.title_type.as_deref()),
        author,
        release_date: parse_release_date(response.released.as_deref(), response.year.as_deref()),
        episodes,
        description,
        source: IMDB_SOURCE.to_string(),
    }))
}

async fn fetch_imdb_detail_no_api(imdb_id: &str) -> Result<Option<ImdbItem>, reqwest::Error> {
    let url = format!("https://www.imdb.com/title/{}/", imdb_id);
    let html = imdb_client()
        .get(url)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await?
        .text()
        .await?;

    if html.contains("AwsWafIntegration") || html.contains("challenge-container") {
        return fetch_imdb_detail_suggestion(imdb_id).await;
    }

    let parsed = {
        let document = Html::parse_document(&html);
        let script_sel = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();

        let mut parsed = None;
        for script in document.select(&script_sel) {
            let raw = script.inner_html();
            let Ok(value) = serde_json::from_str::<Value>(&raw) else {
                continue;
            };
            let Some(obj) = value.as_object() else {
                continue;
            };

            let title = obj
                .get("name")
                .and_then(value_to_string)
                .unwrap_or_else(|| imdb_id.to_string());
            let cover_url = obj
                .get("image")
                .and_then(value_to_string)
                .unwrap_or_default();
            let description = obj
                .get("description")
                .and_then(value_to_string)
                .unwrap_or_default();
            let author = obj
                .get("director")
                .and_then(value_to_string)
                .or_else(|| obj.get("creator").and_then(value_to_string))
                .or_else(|| obj.get("genre").and_then(value_to_string))
                .unwrap_or_default();
            let kind = obj.get("@type").and_then(value_to_string);
            let release_date = obj
                .get("datePublished")
                .and_then(value_to_string)
                .and_then(|d| parse_release_date(Some(&d), None));
            let episodes = obj
                .get("numberOfEpisodes")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .unwrap_or(0);

            parsed = Some(ImdbItem {
                imdb_id: imdb_id.to_string(),
                external_id: imdb_id.to_string(),
                title,
                cover_url,
                r#type: type_from_imdb(kind.as_deref()),
                author,
                release_date,
                episodes,
                description,
                source: IMDB_SOURCE.to_string(),
            });
            break;
        }

        parsed.or_else(|| {
            Selector::parse("title")
                .ok()
                .and_then(|sel| document.select(&sel).next())
                .map(|e| {
                    e.text()
                        .collect::<String>()
                        .replace("- IMDb", "")
                        .trim()
                        .to_string()
                })
                .filter(|title| !title.is_empty())
                .map(|title| ImdbItem {
                    imdb_id: imdb_id.to_string(),
                    external_id: imdb_id.to_string(),
                    title,
                    cover_url: String::new(),
                    r#type: 8,
                    author: String::new(),
                    release_date: None,
                    episodes: 0,
                    description: String::new(),
                    source: IMDB_SOURCE.to_string(),
                })
        })
    };

    match parsed {
        Some(item) => Ok(Some(item)),
        None => fetch_imdb_detail_suggestion(imdb_id).await,
    }
}

pub async fn ensure_imdb_item_cached(
    pool: &MySqlPool,
    imdb_id: &str,
    force: bool,
    use_api: Option<bool>,
) -> Result<ImdbItem, String> {
    let imdb_id = normalize_imdb_id(imdb_id).ok_or_else(|| "Invalid IMDb id".to_string())?;

    if !force {
        match cached_imdb_item(pool, &imdb_id).await {
            Ok(Some(item)) => {
                let fresh = sqlx::query_scalar!(
                    r#"
                    SELECT d.updated_at
                    FROM external_media m
                    JOIN external_media_detailed d ON d.media_id = m.id
                    WHERE m.source = ? AND m.external_id = ?
                    "#,
                    IMDB_SOURCE,
                    imdb_id
                )
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .map(|ts| Utc::now().naive_utc() <= ts + Duration::hours(24))
                .unwrap_or(false);

                if fresh && !item.title.is_empty() {
                    return Ok(item);
                }
            }
            Ok(None) => {}
            Err(e) => log::error!("Failed to read cached IMDb item {}: {:?}", imdb_id, e),
        }
    }

    let fetched = if should_use_api(use_api) {
        fetch_imdb_detail_api(&imdb_id)
            .await
            .map_err(|e| format!("IMDb API fetch failed: {}", e))?
    } else {
        None
    };

    let item = match fetched {
        Some(item) => item,
        None => fetch_imdb_detail_no_api(&imdb_id)
            .await
            .map_err(|e| format!("IMDb fetch failed: {}", e))?
            .ok_or_else(|| "IMDb title not found".to_string())?,
    };

    upsert_imdb_item(pool, &item)
        .await
        .map_err(|e| format!("IMDb cache write failed: {}", e))?;

    Ok(item)
}

pub async fn search_imdb(
    State(pool): State<MySqlPool>,
    Json(params): Json<ImdbSearchQuery>,
) -> Json<SearchImdbResponse> {
    let q = match params.q.as_deref().map(str::trim) {
        Some(q) if !q.is_empty() => q,
        _ => {
            return Json(SearchImdbResponse {
                status: -3,
                data: None,
            });
        }
    };

    let page = params.page.unwrap_or(1).max(1);
    let results = if should_use_api(params.use_api) {
        match search_imdb_api(q, page).await {
            Ok(results) => results,
            Err(e) => {
                log::warn!(
                    "IMDb API search failed, falling back to no-api search: {:?}",
                    e
                );
                match search_imdb_no_api(q).await {
                    Ok(results) => results,
                    Err(_) => {
                        return Json(SearchImdbResponse {
                            status: -1,
                            data: None,
                        });
                    }
                }
            }
        }
    } else {
        match search_imdb_no_api(q).await {
            Ok(results) => results,
            Err(_) => {
                return Json(SearchImdbResponse {
                    status: -1,
                    data: None,
                });
            }
        }
    };

    upsert_imdb_search_results(&pool, &results).await;

    Json(SearchImdbResponse {
        status: 0,
        data: Some(results),
    })
}

pub async fn search_imdb_by_id(
    State(pool): State<MySqlPool>,
    Json(params): Json<ImdbIDSearchQuery>,
) -> Json<ImdbIDSearchResponse> {
    let id = match params.id.as_deref().and_then(normalize_imdb_id) {
        Some(id) => id,
        None => {
            return Json(ImdbIDSearchResponse {
                status: -1,
                data: None,
            });
        }
    };

    match ensure_imdb_item_cached(&pool, &id, params.force.unwrap_or(false), params.use_api).await {
        Ok(item) => Json(ImdbIDSearchResponse {
            status: 0,
            data: Some(item),
        }),
        Err(e) => {
            log::error!("IMDb detail lookup failed for {}: {}", id, e);
            Json(ImdbIDSearchResponse {
                status: -2,
                data: None,
            })
        }
    }
}
