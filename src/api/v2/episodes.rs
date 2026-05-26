use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use sqlx::QueryBuilder;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use reqwest::Client;
use scraper::{Html, Selector};

use crate::auth_bearer::AuthUser;
use super::response::{success, not_found, internal_error, ApiResponse};

#[derive(Serialize, Debug)]
pub struct EpisodeItem {
    pub ordinal: i32,
    pub title: Option<String>,
    pub name_cn: Option<String>,
    pub airdate: Option<NaiveDate>,
    pub duration: Option<String>,
    pub watched: bool,
    pub progress_seconds: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub completed_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct UpdateEpisodeBody {
    pub watched: Option<bool>,
    pub progress_seconds: Option<i32>,
    pub duration_seconds: Option<i32>,
}

#[derive(Deserialize)]
pub struct ForceEpisodesQuery {
    pub force: Option<bool>,
}

pub async fn list_episodes(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(bangumi_id): Path<u32>,
    Query(query): Query<ForceEpisodesQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<EpisodeItem>>>) {
    let bangumi_id_str = bangumi_id.to_string();
    let force = query.force.unwrap_or(false);

    let recording = match sqlx::query!(
        r#"
        SELECT r.id
        FROM recordings r
        JOIN bangumi_info_easy b ON r.bangumi_id = b.id
        WHERE b.external_id = ? AND r.user_id = ?
        "#,
        bangumi_id_str,
        auth_user.user_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return not_found("Recording not found for this bangumi"),
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Database error");
        }
    };

    let recording_id = recording.id;

    let easy_id = match sqlx::query_scalar!(
        "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
        bangumi_id_str
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return not_found("Bangumi not found"),
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Database error");
        }
    };

    ensure_episode_metadata_cached(&pool, easy_id, bangumi_id_str.as_str(), force).await;

    let metadata = match sqlx::query!(
        r#"
        SELECT ordinal, title, name_cn, airdate, duration
        FROM bangumi_episodes
        WHERE bangumi_easy_id = ?
        ORDER BY ordinal ASC
        "#,
        easy_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Database error");
        }
    };

    let user_records = match sqlx::query!(
        r#"
        SELECT ordinal, watched, progress_seconds, duration_seconds, completed_at, updated_at
        FROM episode_records
        WHERE recording_id = ?
        "#,
        recording_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Database error");
        }
    };

    let user_map: std::collections::HashMap<i32, _> = user_records
        .iter()
        .map(|r| (r.ordinal, r))
        .collect();

    let metadata_map: std::collections::HashMap<i32, _> = metadata
        .into_iter()
        .map(|m| (m.ordinal, m))
        .collect();

    let mut seen_ordinals: std::collections::HashSet<i32> = std::collections::HashSet::new();
    let mut episodes: Vec<EpisodeItem> = Vec::new();

    // First pass: metadata entries merged with user records
    for (&ordinal, m) in &metadata_map {
        let user_state = user_map.get(&ordinal);
        seen_ordinals.insert(ordinal);
        episodes.push(EpisodeItem {
            ordinal,
            title: m.title.clone(),
            name_cn: m.name_cn.clone(),
            airdate: m.airdate,
            duration: m.duration.clone(),
            watched: user_state.map(|u| u.watched != 0).unwrap_or(false),
            progress_seconds: user_state.and_then(|u| u.progress_seconds),
            duration_seconds: user_state.and_then(|u| u.duration_seconds),
            completed_at: user_state.and_then(|u| u.completed_at),
            updated_at: user_state.map(|u| u.updated_at),
        });
    }

    // Second pass: user records without metadata (e.g. scraping hasn't run yet)
    for r in &user_records {
        if seen_ordinals.contains(&r.ordinal) {
            continue;
        }
        episodes.push(EpisodeItem {
            ordinal: r.ordinal,
            title: None,
            name_cn: None,
            airdate: None,
            duration: None,
            watched: r.watched != 0,
            progress_seconds: r.progress_seconds,
            duration_seconds: r.duration_seconds,
            completed_at: r.completed_at,
            updated_at: Some(r.updated_at),
        });
    }

    episodes.sort_by_key(|e| e.ordinal);

    success(episodes)
}

pub async fn update_episode(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path((bangumi_id, ordinal)): Path<(u32, i32)>,
    Json(body): Json<UpdateEpisodeBody>,
) -> (StatusCode, Json<ApiResponse<EpisodeItem>>) {
    let bangumi_id_str = bangumi_id.to_string();

    let recording_id = match sqlx::query_scalar!(
        r#"
        SELECT r.id
        FROM recordings r
        JOIN bangumi_info_easy b ON r.bangumi_id = b.id
        WHERE b.external_id = ? AND r.user_id = ?
        "#,
        bangumi_id_str,
        auth_user.user_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return not_found("Recording not found for this bangumi"),
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Database error");
        }
    };

    let watched = body.watched.unwrap_or(false);
    let progress_seconds = body.progress_seconds;
    let duration_seconds = body.duration_seconds;
    let now = chrono::Utc::now().naive_utc();
    let completed_at = if watched { Some(now) } else { None };

    match sqlx::query!(
        r#"
        INSERT INTO episode_records (recording_id, ordinal, watched, progress_seconds, duration_seconds, completed_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            watched = VALUES(watched),
            progress_seconds = VALUES(progress_seconds),
            duration_seconds = VALUES(duration_seconds),
            completed_at = VALUES(completed_at),
            updated_at = VALUES(updated_at)
        "#,
        recording_id,
        ordinal,
        watched as i8,
        progress_seconds,
        duration_seconds,
        completed_at,
        now
    )
    .execute(&pool)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Failed to update episode");
        }
    };

    let updated = match sqlx::query!(
        r#"
        SELECT ordinal, watched, progress_seconds, duration_seconds, completed_at, updated_at
        FROM episode_records
        WHERE recording_id = ? AND ordinal = ?
        "#,
        recording_id,
        ordinal
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => EpisodeItem {
            ordinal: r.ordinal,
            title: None,
            name_cn: None,
            airdate: None,
            duration: None,
            watched: r.watched != 0,
            progress_seconds: r.progress_seconds,
            duration_seconds: r.duration_seconds,
            completed_at: r.completed_at,
            updated_at: Some(r.updated_at),
        },
        Ok(None) => {
            return internal_error("Episode record not found after insert");
        }
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Database error");
        }
    };

    // Sync main recordings.recorder field: "{max_watched_ordinal}|{mm:ss}"
    let count = sqlx::query_scalar!(
        r#"
        SELECT MAX(ordinal) FROM episode_records
        WHERE recording_id = ? AND watched = 1
        "#,
        recording_id
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(None)
    .unwrap_or(0);

    let max_progress = sqlx::query_scalar!(
        r#"
        SELECT MAX(progress_seconds) FROM episode_records
        WHERE recording_id = ?
        "#,
        recording_id
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(None);

    let time_str = match max_progress {
        Some(s) if s > 0 => format_progress_seconds(s),
        _ => "0:00".to_string(),
    };
    let new_recorder = format!("{}|{}", count, time_str);

    if let Err(e) = sqlx::query!(
        "UPDATE recordings SET recorder = ?, updated_at = ? WHERE id = ?",
        new_recorder,
        now,
        recording_id
    )
    .execute(&pool)
    .await
    {
        log::error!("Failed to sync recorder after episode update: {:?}", e);
    }

    success(updated)
}

pub fn format_progress_seconds(sec: i32) -> String {
    let m = sec / 60;
    let s = sec % 60;
    format!("{}:{:02}", m, s)
}

/// Parsed episode data from bangumi subject page.
#[derive(Debug, PartialEq)]
pub struct ParsedEpisode {
    pub ordinal: i32,
    pub ep_id: String,
    pub title: Option<String>,
    pub name_cn: Option<String>,
    pub airdate: Option<NaiveDate>,
    pub duration: Option<String>,
}

/// Extract the first `HH:MM:SS` pattern from a string.
fn extract_time_hms(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    for i in 0..bytes.len().saturating_sub(7) {
        if bytes[i + 2] == b':'
            && bytes[i + 5] == b':'
            && bytes[i..i + 2].iter().all(|b| b.is_ascii_digit())
            && bytes[i + 3..i + 5].iter().all(|b| b.is_ascii_digit())
            && bytes[i + 6..i + 8].iter().all(|b| b.is_ascii_digit())
        {
            return Some(s[i..i + 8].to_string());
        }
    }
    None
}

/// Parse bangumi subject page HTML to extract episode metadata from `ul.prg_list` and `div.prg_popup`.
/// Skips `li.subtitle` entries (e.g. SP, OP, ED).
pub fn parse_prg_list_episodes(html: &str) -> Vec<ParsedEpisode> {
    let document = Html::parse_document(html);

    let li_sel = Selector::parse("ul.prg_list > li").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let popup_sel = Selector::parse("div.prg_popup").unwrap();
    let tip_sel = Selector::parse("span.tip").unwrap();

    let mut from_list: Vec<ParsedEpisode> = Vec::new();

    for li in document.select(&li_sel) {
        if li.value().classes().any(|c| c == "subtitle") {
            continue;
        }

        let a = match li.select(&a_sel).next() {
            Some(a) => a,
            None => continue,
        };

        let href = a.value().attr("href").unwrap_or("");
        let ep_id = href.strip_prefix("/ep/").unwrap_or("").to_string();
        let title_attr = a.value().attr("title").unwrap_or("");
        let text_content = a.text().collect::<String>().trim().to_string();

        let ordinal = text_content.parse::<i32>().ok().or_else(|| {
            let trimmed = title_attr.trim();
            if let Some(rest) = trimmed.strip_prefix("ep.").or_else(|| trimmed.strip_prefix("EP.")) {
                rest.split_whitespace().next().and_then(|n| n.parse::<i32>().ok())
            } else {
                None
            }
        });

        let ordinal = match ordinal {
            Some(o) => o,
            None => continue,
        };

        let title = if title_attr.is_empty() {
            None
        } else {
            let trimmed = title_attr.trim();
            if let Some(rest) = trimmed.strip_prefix("ep.").or_else(|| trimmed.strip_prefix("EP.")) {
                let rest = rest.trim();
                if let Some(pos) = rest.find(|c: char| c == ' ' || c == '\u{3000}') {
                    Some(rest[pos..].trim().to_string())
                } else {
                    Some(rest.to_string())
                }
            } else {
                Some(trimmed.to_string())
            }
        };

        from_list.push(ParsedEpisode {
            ordinal,
            ep_id,
            title,
            name_cn: None,
            airdate: None,
            duration: None,
        });
    }

    let mut popup_map: std::collections::HashMap<String, (Option<String>, Option<NaiveDate>, Option<String>)> = std::collections::HashMap::new();

    for popup in document.select(&popup_sel) {
        let id = popup.value().id().unwrap_or("");
        let ep_id = id.strip_prefix("prginfo_").unwrap_or("").to_string();
        if ep_id.is_empty() {
            continue;
        }

        let tip_text = popup
            .select(&tip_sel)
            .next()
            .map(|s| s.text().collect::<String>())
            .unwrap_or_default();

        let mut name_cn: Option<String> = None;
        let mut airdate: Option<NaiveDate> = None;
        let mut duration: Option<String> = None;

        // tip_text is concatenated text without <br /> separators, e.g.:
        // "中文标题: 初始的终结与结束的开始首播: 2020-01-01时长: 00:52:40讨论 (+20)"
        // Extract fields by finding each prefix and slicing to the next prefix or end.
        let prefixes = ["中文标题:", "首播:", "时长:"];
        for (i, prefix) in prefixes.iter().enumerate() {
            if let Some(start) = tip_text.find(prefix) {
                let val_start = start + prefix.len();
                let val_end = prefixes[i + 1..]
                    .iter()
                    .filter_map(|next| tip_text[val_start..].find(next))
                    .map(|pos| val_start + pos)
                    .min()
                    .unwrap_or(tip_text.len());
                let val = tip_text[val_start..val_end].trim();
                if val.is_empty() {
                    continue;
                }
                match *prefix {
                    "中文标题:" => name_cn = Some(val.to_string()),
                    "首播:" => airdate = NaiveDate::parse_from_str(val, "%Y-%m-%d").ok(),
                    "时长:" => {
                        duration = extract_time_hms(val);
                    }
                    _ => {}
                }
            }
        }

        popup_map.insert(ep_id, (name_cn, airdate, duration));
    }

    let episodes: Vec<ParsedEpisode> = from_list
        .into_iter()
        .map(|mut ep| {
            if let Some((name_cn, airdate, duration)) = popup_map.remove(&ep.ep_id) {
                ep.name_cn = name_cn;
                ep.airdate = airdate;
                ep.duration = duration;
            }
            ep
        })
        .collect();

    episodes
}

pub(crate) async fn ensure_episode_metadata_cached(pool: &MySqlPool, easy_id: u32, bangumi_id: &str, force: bool) {
    if !force {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM bangumi_episodes WHERE bangumi_easy_id = ?",
            easy_id
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if count > 0 {
            let incomplete = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM bangumi_episodes WHERE bangumi_easy_id = ? AND (title IS NULL OR name_cn IS NULL)",
                easy_id
            )
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            if incomplete == 0 {
                return;
            }

            let max_updated = sqlx::query_scalar!(
                "SELECT MAX(updated_at) FROM bangumi_episodes WHERE bangumi_easy_id = ?",
                easy_id
            )
            .fetch_one(pool)
            .await
            .unwrap_or(None);

            let ttl = chrono::Duration::hours(24);
            match max_updated {
                Some(ts) if Utc::now().naive_utc() - ts <= ttl => return,
                _ => {}
            }
        }
    }

    let url = format!("https://bgm.tv/subject/{}", bangumi_id);
    let client = Client::new();
    let resp = match client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to fetch episode metadata page for {}: {:?}", bangumi_id, e);
            return;
        }
    };

    let html = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to decode episode metadata page for {}: {:?}", bangumi_id, e);
            return;
        }
    };

    let episodes = parse_prg_list_episodes(&html);

    if !episodes.is_empty() {
        let mut qb = QueryBuilder::new(
            "INSERT INTO bangumi_episodes (bangumi_easy_id, ordinal, title, name_cn, airdate, duration) ",
        );
        qb.push_values(episodes.iter(), |mut b, ep| {
            b.push_bind(easy_id)
             .push_bind(ep.ordinal)
             .push_bind(&ep.title)
             .push_bind(&ep.name_cn)
             .push_bind(ep.airdate)
             .push_bind(&ep.duration);
        });
        qb.push(
            " ON DUPLICATE KEY UPDATE title = VALUES(title), name_cn = VALUES(name_cn), airdate = VALUES(airdate), duration = VALUES(duration)",
        );
        if let Err(e) = qb.build().execute(pool).await {
            log::error!("batch insert bangumi_episodes error: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_progress_seconds() {
        assert_eq!(format_progress_seconds(0), "0:00");
        assert_eq!(format_progress_seconds(1), "0:01");
        assert_eq!(format_progress_seconds(59), "0:59");
        assert_eq!(format_progress_seconds(60), "1:00");
        assert_eq!(format_progress_seconds(61), "1:01");
        assert_eq!(format_progress_seconds(120), "2:00");
        assert_eq!(format_progress_seconds(3661), "61:01");
    }

    #[test]
    fn test_extract_time_hms() {
        assert_eq!(extract_time_hms("00:52:40").as_deref(), Some("00:52:40"));
        assert_eq!(extract_time_hms("00:52:40讨论 (+20)").as_deref(), Some("00:52:40"));
        assert_eq!(extract_time_hms("01:30:30讨论 (+214)").as_deref(), Some("01:30:30"));
        assert_eq!(extract_time_hms("00:52:40讨论").as_deref(), Some("00:52:40"));
        assert_eq!(extract_time_hms("no time here"), None);
        assert_eq!(extract_time_hms("12:34"), None);
        assert_eq!(extract_time_hms(""), None);
    }

    #[test]
    fn test_parse_prg_list_skips_subtitle() {
        let html = r#"
        <ul class="prg_list">
            <li><a href="/ep/925279" title="ep.1 First Episode">01</a></li>
            <li class="subtitle"><span>SP</span></li>
            <li><a href="/ep/925280" title="ep.2 Second Episode">02</a></li>
            <li><a href="/ep/925281" title="ep.3 Third Episode">03</a></li>
        </ul>
        "#;
        let result = parse_prg_list_episodes(html);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].ordinal, 1);
        assert_eq!(result[0].title.as_deref(), Some("First Episode"));
        assert_eq!(result[0].ep_id, "925279");
        assert_eq!(result[1].ordinal, 2);
        assert_eq!(result[2].ordinal, 3);
    }

    #[test]
    fn test_parse_prg_list_extracts_ordinal_from_text() {
        let html = r#"
        <ul class="prg_list">
            <li><a href="/ep/925279" title="ep.1 始まりの終わりと 終わりの始まり">01</a></li>
            <li><a href="/ep/925280" title="ep.2 再会の魔女">02</a></li>
        </ul>
        "#;
        let result = parse_prg_list_episodes(html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].ordinal, 1);
        assert_eq!(result[1].ordinal, 2);
    }

    #[test]
    fn test_parse_prg_list_decimal_ordinal() {
        let html = r#"
        <ul class="prg_list">
            <li><a href="/ep/925285" title="ep.11.5 Memory Snow">11.5</a></li>
        </ul>
        "#;
        let result = parse_prg_list_episodes(html);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_prg_popup_extracts_name_cn_airdate_duration() {
        let html = r#"
        <ul class="prg_list">
            <li><a href="/ep/925279" title="ep.1 始まりの終わりと 終わりの始まり">01</a></li>
            <li><a href="/ep/925280" title="ep.2 再会の魔女">02</a></li>
        </ul>
        <div id="prginfo_925279" class="prg_popup"><span class="tip">中文标题: 初始的终结与结束的开始<br />首播: 2020-01-01<br />时长: 00:52:40<br /></span></div>
        <div id="prginfo_925280" class="prg_popup"><span class="tip">中文标题: 再遇魔女<br />首播: 2020-01-08<br />时长: 00:24:35<br /></span></div>
        "#;
        let result = parse_prg_list_episodes(html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name_cn.as_deref(), Some("初始的终结与结束的开始"));
        assert_eq!(result[0].airdate, Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()));
        assert_eq!(result[0].duration.as_deref(), Some("00:52:40"));
        assert_eq!(result[1].name_cn.as_deref(), Some("再遇魔女"));
        assert_eq!(result[1].airdate, Some(NaiveDate::from_ymd_opt(2020, 1, 8).unwrap()));
        assert_eq!(result[1].duration.as_deref(), Some("00:24:35"));
    }

    #[test]
    fn test_parse_prg_popup_missing_cn_title() {
        let html = r#"
        <ul class="prg_list">
            <li><a href="/ep/925285" title="ep.11.5 Memory Snow">11.5</a></li>
        </ul>
        <div id="prginfo_925285" class="prg_popup"><span class="tip">首播: 2020-02-12<br />时长: 01:00:00<br /></span></div>
        "#;
        let result = parse_prg_list_episodes(html);
        // 11.5 is not a valid i32, should be excluded
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_prg_popup_filters_duration_trailing_text() {
        let html = r#"
        <ul class="prg_list">
            <li><a href="/ep/925279" title="ep.1 First">01</a></li>
            <li><a href="/ep/925280" title="ep.2 Second">02</a></li>
        </ul>
        <div id="prginfo_925279" class="prg_popup"><span class="tip">中文标题: 初始<br />首播: 2020-01-01<br />时长: 00:52:40<br /><hr class="board" /><span class="cmt clearit"><a href="/subject/ep/925279">讨论</a> <small class="na">(+20)</small></span></span></div>
        <div id="prginfo_925280" class="prg_popup"><span class="tip">中文标题: 再遇<br />首播: 2020-01-08<br />时长: 01:30:30讨论 (+214)<br /></span></div>
        "#;
        let result = parse_prg_list_episodes(html);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].duration.as_deref(), Some("00:52:40"));
        assert_eq!(result[1].duration.as_deref(), Some("01:30:30"));
    }
}
