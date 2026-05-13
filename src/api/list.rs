use axum::{
    Extension, Json, extract::State
};
use serde::Serialize;

use sqlx::mysql::MySqlPool;

use chrono::NaiveDate;

use crate::auth_bearer::AuthUser;

#[derive(Serialize)]
pub struct ListRecorderResponse {
    pub status: i32,
    pub data: Option<Vec<RecorderItem>>
}

#[derive(Serialize)]
pub struct RecorderItem {
    pub id: u32,
    pub local_bangumi_id: u32,
    pub bangumi_id: Option<String>,
    pub recorder: Option<String>,
    pub date: NaiveDate
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
            r.updated_at
        FROM recordings r
        LEFT JOIN bangumi_info_easy b
            ON r.bangumi_id = b.id
        WHERE r.user_id = ?
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
                    local_bangumi_id: r.local_bangumi_id,
                    bangumi_id: r.bangumi_id,
                    recorder: r.recorder,
                    date: r.updated_at.date(),
                });
            }

            Json(ListRecorderResponse {
                status: 0,
                data: Some(results),
            })
        }

        Err(_) => {
            Json(ListRecorderResponse {
                status: -1,
                data: None,
            })
        }
    }
}