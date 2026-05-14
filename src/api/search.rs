use std::collections::HashMap;

use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};

use reqwest::Client;
use urlencoding::encode;
use scraper::{Html, Selector, ElementRef};
use sqlx::mysql::MySqlPool;

use chrono::{Duration, Utc, NaiveDate};

#[derive(Debug, Serialize, Deserialize)]
pub struct TitleSearchQuery {
    pub title: Option<String>,
    pub page: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IDSearchQuery {
    pub id: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct IDSearchResponse {
    pub status: i32,
    pub data: Option<BangumiItem>
}

#[derive(Debug, Serialize)]
pub struct BangumiItem {
    pub bangumi_id: String,
    pub title: String,
    pub cover_url: String,
    pub r#type: i8,
    pub author: String,
    pub release_date: Option<NaiveDate>,
    pub episodes: i32,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct BangumiSearchItem {
    pub bangumi_id: String,
    pub title: String,
    pub alias: String,
    pub cover: String,
    pub info: String,
    pub r#type: i32,
}

#[derive(Debug, Serialize)]
pub struct SearchBangumiResponse {
    pub status: i32,
    pub data: Option<Vec<BangumiSearchItem>>
}

fn parse_bangumi_type_easy(item: ElementRef) -> i32 {
    let type_sel = scraper::Selector::parse("span.ico_subject_type").unwrap();

    let span = match item.select(&type_sel).next() {
        Some(s) => s,
        None => return 8, // Other
    };

    let class = span.value().attr("class").unwrap_or("");

    match () {
        _ if class.contains("subject_type_2") => 1, // TV
        _ if class.contains("subject_type_4") => 2, // Game
        _ if class.contains("subject_type_1") => 7, // Book
        _ if class.contains("subject_type_3") => 6, // Music
        _ => 8, // Other
    }
}

pub async fn search_bangumi_by_title(
    State(pool): State<MySqlPool>,
    Json(params): Json<TitleSearchQuery>
) -> Json<SearchBangumiResponse> {

    let title = match &params.title {
        Some(t) if !t.trim().is_empty() => t.trim(),
        _ => {
            return Json(SearchBangumiResponse { status: -3, data: None });
        }
    };

    let page = params.page.unwrap_or(1);

    let encoded_title = encode(title);

    let url = if page == 1 {
        format!("https://bgm.tv/subject_search/{}?cat=all", encoded_title)
    } else {
        format!(
            "https://bgm.tv/subject_search/{}?cat=all&page={}",
            encoded_title,
            page
        )
    };

    let client = Client::new();

    let resp = match client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return Json(SearchBangumiResponse { status: -1, data: None });
        }
    };

    let html = match resp.text().await {
        Ok(t) => t,
        Err(_) => {
            return Json(SearchBangumiResponse { status: -2, data: None });
        }
    };

    let results = {
        let document = Html::parse_document(&html);

        let item_sel = Selector::parse("li.item").unwrap();
        let title_sel = Selector::parse("h3 a.l").unwrap();
        let alias_sel = Selector::parse("h3 small").unwrap();
        let cover_sel = Selector::parse("img.cover").unwrap();
        let info_sel = Selector::parse("p.info.tip").unwrap();

        let mut results: Vec<BangumiSearchItem> = Vec::new();

        for item in document.select(&item_sel) {

            let a = match item.select(&title_sel).next() {
                Some(a) => a,
                None => continue,
            };

            let title = a.text().collect::<String>().trim().to_string();

            let href = match a.value().attr("href") {
                Some(h) => h,
                None => continue,
            };

            let bangumi_id = match href.strip_prefix("/subject/") {
                Some(id) => id.to_string(),
                None => continue,
            };

            let alias = item
                .select(&alias_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let cover = item
                .select(&cover_sel)
                .next()
                .and_then(|img| img.value().attr("src"))
                .map(|src| {
                    if src.starts_with("//") {
                        format!("https:{}", src)
                    } else {
                        src.to_string()
                    }
                })
                .unwrap_or_default();

            let info = item
                .select(&info_sel)
                .next()
                .map(|e| {
                    e.text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();

            let r#type = parse_bangumi_type_easy(item);
            // ===== push =====
            results.push(BangumiSearchItem {
                bangumi_id,
                title,
                alias,
                cover,
                info,
                r#type,
            });
        }
        results
    };

    for result in &results {
        match sqlx::query_scalar!(
            "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
            result.bangumi_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(_)) => {
                let _ = sqlx::query!(
                    "UPDATE bangumi_info_easy SET title = ?, type = ?, info = ?, cover_url = ? WHERE external_id = ?",
                    result.title,
                    result.r#type,
                    result.info,
                    result.cover,
                    result.bangumi_id,
                )
                .execute(&pool)
                .await;
            }
            Ok(None) => {
                let _ = sqlx::query!(
                    "INSERT INTO bangumi_info_easy (external_id, title, type, info, cover_url) VALUES (?, ?, ?, ?, ?)",
                    result.bangumi_id,
                    result.title,
                    result.r#type,
                    result.info,
                    result.cover,
                )
                .execute(&pool)
                .await;
            }
            Err(_) => {}
        }
    }

    Json(SearchBangumiResponse {
        status: 0,
        data: Some(results),
    })
}

fn parse_date(input: &str) -> Option<NaiveDate> {
    let cleaned = input
        .replace("年", "-")
        .replace("月", "-")
        .replace("日", "");

    NaiveDate::parse_from_str(&cleaned, "%Y-%m-%d").ok()
}

pub async fn search_bangumi_by_id(
    State(pool): State<MySqlPool>,
    Json(params): Json<IDSearchQuery>
) -> Json<IDSearchResponse> {

    if params.id.is_none() {
        return Json(IDSearchResponse { 
            status: -1,
            data: None
        })
    }

    let id = params.id.unwrap();

    let _local_bangumi_id: Option<u32> = match sqlx::query_scalar!(
        "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
        id
    )
    .fetch_optional(&pool)
    .await    
    {
        Ok(Some(id)) => Some(id),
        Ok(None) => None,
        Err(_) => return Json(IDSearchResponse { 
            status: -2,
            data: None }),
    };

    if _local_bangumi_id.is_some() {
        match sqlx::query!(
            r#"
            SELECT 
                r.id,
                r.bangumi_id AS local_bangumi_id,
                b.external_id AS bangumi_id,
                b.title,
                b.type,
                d.author,
                d.release_date,
                d.episodes,
                d.description,
                b.cover_url,
                r.recorder,
                r.status,
                r.updated_at,
                r.created_at
            FROM recordings r
            LEFT JOIN bangumi_info_easy b
                ON r.bangumi_id = b.id
            LEFT JOIN bangumi_info_detailed d
                ON d.bangumi_id = b.id
            WHERE b.id = ?
            "#,
            _local_bangumi_id.unwrap()
        )
        .fetch_all(&pool)
        .await
            {
                Ok(rows) => {
                    if let Some(r) = rows.into_iter().next() {
                        if Utc::now().naive_utc() <= r.updated_at + Duration::hours(24) {
                            return Json(IDSearchResponse {
                                status: 0,
                                data: Some(BangumiItem {
                                    bangumi_id: r.bangumi_id.unwrap_or_default(),
                                    title: r.title.unwrap_or_default(),
                                    cover_url: r.cover_url.unwrap_or_default(),
                                    r#type: r.r#type.unwrap_or(8),
                                    author: r.author.unwrap_or_default(),
                                    release_date: Some(r.release_date.unwrap_or_default()),
                                    episodes: r.episodes.unwrap_or(0),
                                    description: r.description.unwrap_or_default(),
                                }),
                            });
                        }
                    }
                }
                Err(_) => {}
            }
    }

    let client = Client::new();
    let url = format!("https://bgm.tv/subject/{}", id);

    let resp = match client.get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Json(IDSearchResponse { status: -1, data: None }),
    };

    let html = match resp.text().await {
        Ok(t) => t,
        Err(_) => return Json(IDSearchResponse { status: -2, data: None }),
    };
    let mut result = {
        let document = Html::parse_document(&html);

        let type_selector = Selector::parse("h1.nameSingle small.grey").unwrap();

        let type_text = document
            .select(&type_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let r#type = match type_text.as_str() {
            "TV" => 1,
            "剧场版" => 2,
            "OVA" => 3,
            "ONA" => 4,
            "TV SP" => 5,
            "Music" => 6,
            "书籍" => 7,
            _ => 8,
        };

        let li_selector = Selector::parse("#infobox li").unwrap();
        let span_selector = Selector::parse("span.tip").unwrap();

        let mut info_map: HashMap<String, String> = HashMap::new();

        for li in document.select(&li_selector) {
            let key = li.select(&span_selector)
                .next()
                .map(|e| e.text().collect::<String>().replace(":", "").trim().to_string());

            let value = li.text()
                .collect::<String>()
                .split(':')
                .skip(1)
                .collect::<String>()
                .trim()
                .to_string();

            if let Some(k) = key {
                info_map.insert(k, value);
            }
        }

        let author = info_map.get("原作")
            .cloned()
            .unwrap_or_default();

        let episodes = info_map.get("话数")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0);

        let release_date = info_map.get("放送开始")
            .and_then(|v| parse_date(v));

        let desc_selector = Selector::parse("#subject_summary").unwrap();

        let description = document
            .select(&desc_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let title = document
            .select(&Selector::parse(".infobox a.thickbox").unwrap())
            .next()
            .and_then(|a| a.value().attr("title"))
            .map(|t| t.trim().to_string())
            .or_else(|| {
                let name_sel = Selector::parse("h1.nameSingle").unwrap();
                document
                    .select(&name_sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
            })
            .unwrap_or_default();

        let cover_sel = Selector::parse(".infobox img.cover").unwrap();
        let cover_url = document
            .select(&cover_sel)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(|src| {
                if src.starts_with("//") {
                    format!("https:{}", src)
                } else {
                    src.to_string()
                }
            })
            .unwrap_or_default();

        BangumiItem {
            bangumi_id: id.to_string(),
            title,
            cover_url,
            r#type,
            author,
            release_date,
            episodes,
            description,
        }
    };

    let bangumi_easy_id: u32 = match sqlx::query!(
        r#"SELECT id, title, cover_url FROM bangumi_info_easy WHERE external_id = ?"#,
        id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(row)) => {
            if !row.title.is_empty() {
                result.title = row.title;
            }
            if let Some(ref db_cover) = row.cover_url {
                if !db_cover.is_empty() {
                    result.cover_url = db_cover.clone();
                }
            }
            row.id
        }
        Ok(None) => {
            let insert_result = sqlx::query!(
                r#"
                INSERT INTO bangumi_info_easy
                (external_id, title, type, info, cover_url)
                VALUES (?, ?, ?, ?, ?)
                "#,
                id,
                result.title,
                result.r#type,
                "",
                result.cover_url
            )
            .execute(&pool)
            .await;

            match insert_result {
                Ok(res) => res.last_insert_id() as u32,
                Err(_) => return Json(IDSearchResponse { status: -4, data: None }),
            }
        }
        Err(_) => return Json(IDSearchResponse { status: -5, data: None }),
    };

    match sqlx::query_scalar!(
        "SELECT id FROM bangumi_info_detailed WHERE bangumi_id = ?",
        bangumi_easy_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(_)) => {
            let _ = sqlx::query!(
                "UPDATE bangumi_info_detailed SET type = ?, author = ?, release_date = ?, episodes = ?, description = ? WHERE bangumi_id = ?",
                result.r#type,
                result.author,
                result.release_date,
                result.episodes,
                result.description,
                bangumi_easy_id,
            )
            .execute(&pool)
            .await;
        }
        Ok(None) => {
            let _ = sqlx::query!(
                "INSERT INTO bangumi_info_detailed (bangumi_id, type, author, release_date, episodes, description) VALUES (?, ?, ?, ?, ?, ?)",
                bangumi_easy_id,
                result.r#type,
                result.author,
                result.release_date,
                result.episodes,
                result.description,
            )
            .execute(&pool)
            .await;
        }
        Err(_) => {}
    };

    Json(IDSearchResponse {
        status: 0,
        data: Some(result),
    })
}