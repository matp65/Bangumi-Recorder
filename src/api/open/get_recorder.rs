use axum::{
    Json, extract::{Query, State}
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;

use super::api_token::check_api_token;

#[derive(Deserialize)]
pub struct GetRecorderQuery {
    pub bangumi_id: Option<u32>,
    pub token: Option<String>
}

#[derive(Serialize)]
pub struct GetRecorderResponse {
    pub status: i32,
    pub local_bangumi_id: Option<u32>,
    pub bangumi_id: Option<u32>,
    pub recorder: Option<String>,
    pub date: Option<NaiveDate>
}

pub async fn get_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<GetRecorderQuery>
) -> Json<GetRecorderResponse> {

    if params.bangumi_id.is_none() || params.token.is_none() {
        return Json(GetRecorderResponse { 
            status: -2,
            local_bangumi_id: None,
            bangumi_id: None,
            recorder: None,
            date: None 
        })
    }

    let temp_local_bangumi_id = sqlx::query!(
        "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
        params.bangumi_id.unwrap()
    )
    .fetch_optional(&pool)
    .await;

    let local_bangumi_id = match temp_local_bangumi_id {
        Ok(Some(record)) => record.id,
        Ok(None) => {
            log::error!("Bangumi with external_id {} not found", params.bangumi_id.unwrap());
            return Json(GetRecorderResponse { 
                status: -2,
                local_bangumi_id: None,
                bangumi_id: Some(params.bangumi_id.unwrap()),
                recorder: None,
                date: None
            });
        }
        Err(e) => {
            log::error!("Failed to query bangumi_info_easy: {}", e);
            return Json(GetRecorderResponse { 
                status: -2,
                local_bangumi_id: None,
                bangumi_id: Some(params.bangumi_id.unwrap()),
                recorder: None,
                date: None
            });
        }
    };

    let token = params.token.as_ref().unwrap();

    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => {
            return Json(GetRecorderResponse {
                status: -2,
                local_bangumi_id: None,
                bangumi_id: None,
                recorder: None,
                date: None
            });
        }
    };

    match sqlx::query!(
        "SELECT recorder, updated_at FROM recordings WHERE user_id = ? AND bangumi_id = ?",
        user_id,
        local_bangumi_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => {
            Json(GetRecorderResponse { 
                status: 0, 
                local_bangumi_id: Some(local_bangumi_id),
                bangumi_id: Some(params.bangumi_id.unwrap()),
                recorder: r.recorder,
                date: Some(r.updated_at.date()),
            })
        }
        Ok(None) => {
            Json(GetRecorderResponse { 
                status: 0,
                local_bangumi_id: Some(local_bangumi_id),
                bangumi_id: Some(params.bangumi_id.unwrap()),
                recorder: None,
                date: None,
            })
        }
        Err(_) => {
            Json(GetRecorderResponse {
                status: -2,
                local_bangumi_id: Some(local_bangumi_id),
                bangumi_id: Some(params.bangumi_id.unwrap()),
                recorder: None,
                date: None,
            })
        }
    }
}