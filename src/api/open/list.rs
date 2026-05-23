use axum::{
    Json, extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use sqlx::mysql::MySqlPool;

use chrono::NaiveDate;

use super::api_token::check_api_token;

#[derive(Serialize)]
pub struct ListRecorderResponse {
    pub status: i32,
    pub data: Option<Vec<RecorderItem>>
}

#[derive(Deserialize)]
pub struct ListRecorderQuery {
    pub token: Option<String>
}
#[derive(Serialize)]
pub struct RecorderItem {
    pub id: u32,
    pub local_bangumi_id: Option<u32>,
    pub bangumi_id: Option<String>,
    pub recorder: Option<String>,
    pub user_status: Option<i8>,
    pub is_delete: bool,
    pub date: NaiveDate
}

pub async fn list_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<ListRecorderQuery>
) -> Result<Json<ListRecorderResponse>, StatusCode> {

    if params.token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let mut results: Vec<RecorderItem> = Vec::new();

    let token = params.token.as_ref().unwrap();
    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

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
        user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            for r in rows {
                results.push(RecorderItem {
                    id: r.id,
                    local_bangumi_id: r.local_bangumi_id,
                    bangumi_id: r.bangumi_id,
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    is_delete: r.is_delete != 0,
                    date: r.updated_at.date(),
                });
            }
        }

        Err(_) => {
            return Ok(Json(ListRecorderResponse {
                status: -1,
                data: None,
            }))
        }
    }

    match sqlx::query!(
        r#"
        SELECT 
            r.id,
            r.bangumi_id AS local_bangumi_id,
            r.recorder,
            r.status,
            r.is_delete,
            r.updated_at
        FROM recordings r
        WHERE r.user_id = ? AND r.other_id IS NOT NULL
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            for r in rows {
                results.push(RecorderItem {
                    id: r.id,
                    local_bangumi_id: r.local_bangumi_id,
                    bangumi_id: None,
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    is_delete: r.is_delete != 0,
                    date: r.updated_at.date(),
                });
            }
        }

        Err(_) => {}
    }

    Ok(Json(ListRecorderResponse {
        status: 0,
        data: Some(results),
    }))
}
