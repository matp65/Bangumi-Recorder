use axum::{
    extract::{State, Extension},
    Json,
};
use serde::Serialize;
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;

use crate::auth_bearer::AuthUser;

#[derive(Serialize)]
pub struct DetailListResponse {
    pub status: i32,
    pub data: Option<Vec<DetailListItem>>
}

#[derive(Serialize)]
pub struct DetailListItem {
    pub id: u32,
    pub local_bangumi_id: Option<u32>,
    pub other_id: Option<u32>,
    pub local_other_id: Option<u32>,
    pub bangumi_id: Option<String>,
    pub title: Option<String>,
    pub r#type: Option<i8>,
    pub author: Option<String>,
    pub episodes: Option<i32>,
    pub cover_url: Option<String>,
    pub recorder: Option<String>,
    pub user_status: Option<i8>,
    pub is_delete: bool,
    pub updated_at: NaiveDate,
    pub created_at: NaiveDate
}

pub async fn get_detail_list(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> Json<DetailListResponse> {
    let mut results: Vec<DetailListItem> = Vec::new();

    match sqlx::query!(
        r#"
        SELECT 
            r.id,
            r.bangumi_id AS local_bangumi_id,
            b.external_id AS bangumi_id,
            b.title,
            b.type,
            d.author,
            d.episodes,
            b.cover_url,
            r.recorder,
            r.status,
            r.is_delete,
            r.updated_at,
            r.created_at
        FROM recordings r
        LEFT JOIN bangumi_info_easy b
            ON r.bangumi_id = b.id
        LEFT JOIN bangumi_info_detailed d
            ON d.bangumi_id = b.id
        WHERE r.user_id = ? AND r.bangumi_id IS NOT NULL
        "#,
        auth_user.user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            for r in rows {
                results.push(DetailListItem {
                    id: r.id,
                    local_bangumi_id: r.local_bangumi_id,
                    other_id: None,
                    local_other_id: None,
                    bangumi_id: r.bangumi_id,
                    title: r.title,
                    r#type: r.r#type,
                    author: r.author,
                    episodes: r.episodes,
                    cover_url: r.cover_url,
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    is_delete: r.is_delete != 0,
                    updated_at: r.updated_at.date(),
                    created_at: r.created_at.date(),
                });
            }
        }
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return Json(DetailListResponse {
                status: -1,
                data: None
            })
        }
    }

    match sqlx::query!(
        r#"
        SELECT 
            r.id,
            r.bangumi_id AS local_bangumi_id,
            r.other_id,
            o.name AS title,
            o.cover_url,
            o.max_number AS episodes,
            r.recorder,
            r.status,
            r.is_delete,
            r.updated_at,
            r.created_at
        FROM recordings r
        LEFT JOIN other_recorders o ON r.other_id = o.id
        WHERE r.user_id = ? AND r.other_id IS NOT NULL
        "#,
        auth_user.user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            for r in rows {
                results.push(DetailListItem {
                    id: r.id,
                    local_bangumi_id: r.local_bangumi_id,
                    other_id: r.other_id,
                    local_other_id: Some(r.id),
                    bangumi_id: None,
                    title: r.title,
                    r#type: None,
                    author: None,
                    episodes: r.episodes,
                    cover_url: r.cover_url,
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    is_delete: r.is_delete != 0,
                    updated_at: r.updated_at.date(),
                    created_at: r.created_at.date(),
                });
            }
        }
        Err(e) => {
            log::error!("DB error: {:?}", e);
        }
    }

    Json(DetailListResponse {
        status: 0,
        data: Some(results),
    })
}
