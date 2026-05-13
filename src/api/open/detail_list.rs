use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;
use super::api_token::check_api_token;


#[derive(Deserialize)]
pub struct DetailListQuery {
    pub token: Option<String>
}

#[derive(Serialize)]
pub struct DetailListResponse {
    pub status: i32,
    pub data: Option<Vec<DetailListItem>>
}

#[derive(Serialize)]
pub struct DetailListItem {
    pub id: u32,
    pub local_bangumi_id: u32,
    pub bangumi_id: Option<String>,
    pub title: Option<String>,
    pub r#type: Option<i8>,
    pub author: Option<String>,
    pub episodes: Option<i32>,
    pub cover_url: Option<String>,
    pub recorder: Option<String>,
    pub updated_at: NaiveDate,
    pub created_at: NaiveDate
}

pub async fn get_detail_list(
    State(pool): State<MySqlPool>,
    Query(params): Query<DetailListQuery>,
) -> Json<DetailListResponse> {

    if params.token.is_none() {
        return Json(DetailListResponse { 
            status: -1, 
            data: None 
        })
    }

    let token = params.token.as_ref().unwrap();
    
    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => {
            return Json(DetailListResponse {
                status: -2,
                data: None,
            });
        }
    };

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
            r.updated_at,
            r.created_at
        FROM recordings r
        LEFT JOIN bangumi_info_easy b
            ON r.bangumi_id = b.id
        LEFT JOIN bangumi_info_detailed d
            ON d.bangumi_id = b.id
        WHERE r.user_id = ?
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            for r in rows {
                results.push(DetailListItem {
                    id: r.id,
                    local_bangumi_id: r.local_bangumi_id,
                    bangumi_id: r.bangumi_id,
                    title: r.title,
                    r#type: r.r#type,
                    author: r.author,
                    episodes: r.episodes,
                    cover_url: r.cover_url,
                    recorder: r.recorder,
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

    Json(DetailListResponse {
        status: 0,
        data: Some(results),
    })
}