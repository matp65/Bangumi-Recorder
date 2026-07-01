use axum::{
    Json,
    extract::{Extension, State},
};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

use crate::api::imdb::{IMDB_SOURCE, normalize_imdb_id};
use crate::auth_bearer::AuthUser;

#[derive(Deserialize)]
pub struct GetRecorderQuery {
    pub bangumi_id: Option<u32>,
    pub imdb_id: Option<String>,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub local_bangumi_id: Option<u32>,
    pub local_external_media_id: Option<u32>,
    pub other_id: Option<u32>,
}

#[derive(Serialize)]
pub struct GetRecorderResponse {
    pub status: i32,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub imdb_id: Option<String>,
    pub local_external_media_id: Option<u32>,
    pub local_bangumi_id: Option<u32>,
    pub other_id: Option<u32>,
    pub bangumi_id: Option<u32>,
    pub recorder: Option<String>,
    pub user_status: Option<i8>,
    pub is_delete: Option<bool>,
    pub date: Option<NaiveDate>,
}

fn empty_response(status: i32) -> GetRecorderResponse {
    GetRecorderResponse {
        status,
        source: None,
        external_id: None,
        imdb_id: None,
        local_external_media_id: None,
        local_bangumi_id: None,
        other_id: None,
        bangumi_id: None,
        recorder: None,
        user_status: None,
        is_delete: None,
        date: None,
    }
}

struct RecordingRow {
    bangumi_id: Option<u32>,
    external_media_id: Option<u32>,
    other_id: Option<u32>,
    recorder: Option<String>,
    status: i8,
    is_delete: i8,
    updated_at: NaiveDateTime,
}

async fn respond_other(pool: &MySqlPool, user_id: i64, other_id: u32) -> Json<GetRecorderResponse> {
    let row = sqlx::query_as!(
        RecordingRow,
        "SELECT bangumi_id, external_media_id, other_id, recorder, status, is_delete, updated_at FROM recordings WHERE user_id = ? AND other_id = ? AND is_delete = 0",
        user_id,
        other_id
    )
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some(r)) => Json(GetRecorderResponse {
            status: 0,
            source: Some("custom".to_string()),
            external_id: None,
            imdb_id: None,
            local_external_media_id: r.external_media_id,
            local_bangumi_id: r.bangumi_id,
            other_id: r.other_id,
            bangumi_id: None,
            recorder: r.recorder,
            user_status: Some(r.status),
            is_delete: Some(r.is_delete != 0),
            date: Some(r.updated_at.date()),
        }),
        Ok(None) => Json(GetRecorderResponse {
            status: 0,
            source: Some("custom".to_string()),
            external_id: None,
            imdb_id: None,
            local_external_media_id: None,
            local_bangumi_id: None,
            other_id: Some(other_id),
            bangumi_id: None,
            recorder: None,
            user_status: None,
            is_delete: None,
            date: None,
        }),
        Err(e) => {
            log::error!("Failed to query custom recording: {}", e);
            Json(empty_response(-2))
        }
    }
}

async fn resolve_bangumi(
    pool: &MySqlPool,
    params: &GetRecorderQuery,
) -> Result<Option<(u32, String)>, sqlx::Error> {
    if let Some(local_id) = params.local_bangumi_id {
        return Ok(sqlx::query!(
            "SELECT id, external_id FROM bangumi_info_easy WHERE id = ?",
            local_id
        )
        .fetch_optional(pool)
        .await?
        .map(|r| (r.id, r.external_id)));
    }

    if let Some(id) = params.bangumi_id {
        return Ok(sqlx::query!(
            "SELECT id, external_id FROM bangumi_info_easy WHERE external_id = ?",
            id
        )
        .fetch_optional(pool)
        .await?
        .map(|r| (r.id, r.external_id)));
    }

    if params
        .source
        .as_deref()
        .unwrap_or_default()
        .eq_ignore_ascii_case("bangumi")
        && let Some(external_id) = params.external_id.as_deref()
    {
        return Ok(sqlx::query!(
            "SELECT id, external_id FROM bangumi_info_easy WHERE external_id = ?",
            external_id
        )
        .fetch_optional(pool)
        .await?
        .map(|r| (r.id, r.external_id)));
    }

    Ok(None)
}

async fn resolve_external_media(
    pool: &MySqlPool,
    params: &GetRecorderQuery,
) -> Result<Option<(u32, String)>, sqlx::Error> {
    if let Some(local_id) = params.local_external_media_id {
        return Ok(sqlx::query!(
            "SELECT id, external_id FROM external_media WHERE id = ? AND source = ?",
            local_id,
            IMDB_SOURCE
        )
        .fetch_optional(pool)
        .await?
        .map(|r| (r.id, r.external_id)));
    }

    let imdb_id = params
        .imdb_id
        .as_deref()
        .or_else(|| {
            if params
                .source
                .as_deref()
                .unwrap_or_default()
                .eq_ignore_ascii_case(IMDB_SOURCE)
            {
                params.external_id.as_deref()
            } else {
                None
            }
        })
        .and_then(normalize_imdb_id);

    if let Some(imdb_id) = imdb_id {
        return Ok(sqlx::query!(
            "SELECT id, external_id FROM external_media WHERE source = ? AND external_id = ?",
            IMDB_SOURCE,
            imdb_id
        )
        .fetch_optional(pool)
        .await?
        .map(|r| (r.id, r.external_id)));
    }

    Ok(None)
}

pub async fn get_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<GetRecorderQuery>,
) -> Json<GetRecorderResponse> {
    if params.bangumi_id.is_none()
        && params.imdb_id.is_none()
        && params.external_id.is_none()
        && params.local_bangumi_id.is_none()
        && params.local_external_media_id.is_none()
        && params.other_id.is_none()
    {
        return Json(empty_response(-2));
    }

    if let Some(other_id) = params.other_id {
        return respond_other(&pool, auth_user.user_id, other_id).await;
    }

    if let Ok(Some((local_id, external_id))) = resolve_external_media(&pool, &params).await {
        return match sqlx::query!(
            "SELECT recorder, status, is_delete, updated_at FROM recordings WHERE user_id = ? AND external_media_id = ? AND is_delete = 0",
            auth_user.user_id,
            local_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(r)) => Json(GetRecorderResponse {
                status: 0,
                source: Some(IMDB_SOURCE.to_string()),
                external_id: Some(external_id.clone()),
                imdb_id: Some(external_id),
                local_external_media_id: Some(local_id),
                local_bangumi_id: None,
                other_id: None,
                bangumi_id: None,
                recorder: r.recorder,
                user_status: Some(r.status),
                is_delete: Some(r.is_delete != 0),
                date: Some(r.updated_at.date()),
            }),
            Ok(None) => Json(GetRecorderResponse {
                status: 0,
                source: Some(IMDB_SOURCE.to_string()),
                external_id: Some(external_id.clone()),
                imdb_id: Some(external_id),
                local_external_media_id: Some(local_id),
                local_bangumi_id: None,
                other_id: None,
                bangumi_id: None,
                recorder: None,
                user_status: None,
                is_delete: None,
                date: None,
            }),
            Err(e) => {
                log::error!("Failed to query IMDb recording: {}", e);
                Json(empty_response(-2))
            }
        };
    }

    let (local_id, external_id) = match resolve_bangumi(&pool, &params).await {
        Ok(Some(record)) => record,
        Ok(None) => return Json(empty_response(-2)),
        Err(e) => {
            log::error!("Failed to resolve bangumi record: {}", e);
            return Json(empty_response(-2));
        }
    };

    match sqlx::query!(
        "SELECT recorder, status, is_delete, updated_at FROM recordings WHERE user_id = ? AND bangumi_id = ? AND is_delete = 0",
        auth_user.user_id,
        local_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => Json(GetRecorderResponse {
            status: 0,
            source: Some("bangumi".to_string()),
            external_id: Some(external_id.clone()),
            imdb_id: None,
                local_external_media_id: None,
                local_bangumi_id: Some(local_id),
                other_id: None,
                bangumi_id: external_id.parse::<u32>().ok(),
            recorder: r.recorder,
            user_status: Some(r.status),
            is_delete: Some(r.is_delete != 0),
            date: Some(r.updated_at.date()),
        }),
        Ok(None) => Json(GetRecorderResponse {
            status: 0,
            source: Some("bangumi".to_string()),
            external_id: Some(external_id.clone()),
            imdb_id: None,
                local_external_media_id: None,
                local_bangumi_id: Some(local_id),
                other_id: None,
                bangumi_id: external_id.parse::<u32>().ok(),
            recorder: None,
            user_status: None,
            is_delete: None,
            date: None,
        }),
        Err(e) => {
            log::error!("Failed to query bangumi recording: {}", e);
            Json(empty_response(-2))
        }
    }
}
