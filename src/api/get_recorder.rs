use axum::{
    extract::{State, Extension},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;

use crate::auth_bearer::AuthUser;

#[derive(Deserialize)]
pub struct GetRecorderQuery {
    pub bangumi_id: Option<u32>
}

#[derive(Serialize)]
pub struct GetRecorderResponse {
    pub status: i32,
    pub local_bangumi_id: Option<u32>,
    pub bangumi_id: Option<u32>,
    pub recorder: Option<String>,
    pub user_status: Option<i8>,
    pub date: Option<NaiveDate>
}

pub async fn get_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<GetRecorderQuery>
) -> Json<GetRecorderResponse> {

    let bangumi_external_id = match params.bangumi_id {
        Some(id) => id,
        None => {
            return Json(GetRecorderResponse {
                status: -2,
                local_bangumi_id: None,
                bangumi_id: None,
                recorder: None,
                user_status: None,
                date: None,
            });
        }
    };

    let temp_local_bangumi_id = sqlx::query!(
        "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
        bangumi_external_id
    )
    .fetch_optional(&pool)
    .await;

    let local_bangumi_id = match temp_local_bangumi_id {
        Ok(Some(record)) => record.id,
        Ok(None) => {
            log::error!("Bangumi with external_id {} not found", bangumi_external_id);
            return Json(GetRecorderResponse { 
                status: -2,
                local_bangumi_id: None,
                bangumi_id: Some(bangumi_external_id),
                recorder: None,
                user_status: None,
                date: None
            });
        }
        Err(e) => {
            log::error!("Failed to query bangumi_info_easy: {}", e);
            return Json(GetRecorderResponse { 
                status: -2,
                local_bangumi_id: None,
                bangumi_id: Some(bangumi_external_id),
                recorder: None,
                user_status: None,
                date: None
            });
        }
    };

    match sqlx::query!(
        "SELECT recorder, status, updated_at FROM recordings WHERE user_id = ? AND bangumi_id = ?",
        auth_user.user_id,
        local_bangumi_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => {
            Json(GetRecorderResponse { 
                status: 0, 
                local_bangumi_id: Some(local_bangumi_id),
                bangumi_id: Some(bangumi_external_id),
                recorder: r.recorder,
                user_status: Some(r.status),
                date: Some(r.updated_at.date()),
            })
        }
        Ok(None) => {
            Json(GetRecorderResponse { 
                status: 0,
                local_bangumi_id: Some(local_bangumi_id),
                bangumi_id: Some(bangumi_external_id),
                recorder: None,
                user_status: None,
                date: None,
            })
        }
        Err(_) => {
            Json(GetRecorderResponse {
                status: -2,
                local_bangumi_id: Some(local_bangumi_id),
                bangumi_id: Some(bangumi_external_id),
                recorder: None,
                user_status: None,
                date: None,
            })
        }
    }
}