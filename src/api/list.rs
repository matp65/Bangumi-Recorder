use axum::{Extension, Json, extract::State};
use serde::Serialize;

use sqlx::mysql::MySqlPool;

use chrono::NaiveDateTime;

use crate::auth_bearer::AuthUser;

#[derive(Serialize)]
pub struct ListRecorderResponse {
    pub status: i32,
    pub data: Option<Vec<RecorderItem>>,
}

#[derive(Serialize)]
pub struct RecorderItem {
    pub id: u32,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub local_external_media_id: Option<u32>,
    pub local_bangumi_id: Option<u32>,
    pub bangumi_id: Option<String>,
    pub imdb_id: Option<String>,
    pub recorder: Option<String>,
    pub user_status: Option<i8>,
    pub is_delete: bool,
    pub updated_at: NaiveDateTime,
    pub date: NaiveDateTime,
}

pub async fn list_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> Json<ListRecorderResponse> {
    let mut results: Vec<RecorderItem> = Vec::new();

    match sqlx::query!(
        r#"
        SELECT 
            r.id,
            r.bangumi_id AS local_bangumi_id,
            b.external_id AS bangumi_id,
            r.recorder,
            r.status,
            r.is_delete,
            r.updated_at
        FROM recordings r
        LEFT JOIN bangumi_info_easy b
            ON r.bangumi_id = b.id
        WHERE r.user_id = ? AND r.bangumi_id IS NOT NULL
        "#,
        auth_user.user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            for r in rows {
                results.push(RecorderItem {
                    id: r.id,
                    source: Some("bangumi".to_string()),
                    external_id: r.bangumi_id.clone(),
                    local_external_media_id: None,
                    local_bangumi_id: r.local_bangumi_id,
                    bangumi_id: r.bangumi_id,
                    imdb_id: None,
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    is_delete: r.is_delete != 0,
                    updated_at: r.updated_at,
                    date: r.updated_at,
                });
            }
        }

        Err(_) => {
            return Json(ListRecorderResponse {
                status: -1,
                data: None,
            });
        }
    }

    match sqlx::query!(
        r#"
        SELECT 
            r.id,
            r.external_media_id AS local_external_media_id,
            e.source,
            e.external_id,
            r.recorder,
            r.status,
            r.is_delete,
            r.updated_at
        FROM recordings r
        LEFT JOIN external_media e ON r.external_media_id = e.id
        WHERE r.user_id = ? AND r.external_media_id IS NOT NULL
        "#,
        auth_user.user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            for r in rows {
                results.push(RecorderItem {
                    id: r.id,
                    source: r.source.clone(),
                    external_id: r.external_id.clone(),
                    local_external_media_id: r.local_external_media_id,
                    local_bangumi_id: None,
                    bangumi_id: None,
                    imdb_id: if r.source.as_deref() == Some("imdb") {
                        r.external_id
                    } else {
                        None
                    },
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    is_delete: r.is_delete != 0,
                    updated_at: r.updated_at,
                    date: r.updated_at,
                });
            }
        }

        Err(e) => {
            log::error!("Failed to list external media records: {:?}", e);
        }
    }

    if let Ok(rows) = sqlx::query!(
        r#"
        SELECT 
            r.id,
            r.bangumi_id AS local_bangumi_id,
            r.external_media_id AS local_external_media_id,
            r.other_id,
            r.recorder,
            r.status,
            r.is_delete,
            r.updated_at
        FROM recordings r
        WHERE r.user_id = ? AND r.other_id IS NOT NULL
        "#,
        auth_user.user_id
    )
    .fetch_all(&pool)
    .await
    {
        for r in rows {
            results.push(RecorderItem {
                id: r.id,
                source: Some("custom".to_string()),
                external_id: None,
                local_external_media_id: r.local_external_media_id,
                local_bangumi_id: r.local_bangumi_id,
                bangumi_id: None,
                imdb_id: None,
                recorder: r.recorder,
                user_status: Some(r.status),
                is_delete: r.is_delete != 0,
                updated_at: r.updated_at,
                date: r.updated_at,
            });
        }
    }

    Json(ListRecorderResponse {
        status: 0,
        data: Some(results),
    })
}
