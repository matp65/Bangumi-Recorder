use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;
use super::api_token::check_api_token;
use crate::api::search::{IDSearchQuery, search_bangumi_by_id};

#[derive(Debug, Deserialize, Serialize)]
pub struct AddRecordResponse {
    pub status: i32,
    pub local_bangumi_id: Option<u32>,
    pub bangumi_id: Option<u32>,
    pub recorder: Option<String>,
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct AddRecordQuery {
    pub bangumi_id: Option<u32>,
    pub user_status: Option<i32>,
    pub recorder: Option<String>,
    pub token: Option<String>,
}

pub async fn add_record_open(
    State(pool): State<MySqlPool>,
    Query(params): Query<AddRecordQuery>,
) -> Json<AddRecordResponse> {
    if params.bangumi_id.is_none() || (params.recorder.is_none() && params.user_status.is_none()) || params.token.is_none() {
        return Json(AddRecordResponse {
            status: -1,
            local_bangumi_id: None,
            bangumi_id: None,
            recorder: None,
            date: None,
        });
    }

    let bangumi_tv_id = params.bangumi_id.unwrap();
    let user_status = params.user_status.clone().unwrap_or(0);
    let recorder = params.recorder.clone().unwrap_or_default();

    let temp_local_bangumi_id = sqlx::query!(
        "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
        bangumi_tv_id
    )
    .fetch_one(&pool)
    .await;

    let bangumi_id = match temp_local_bangumi_id {
        Ok(record) => record.id,
        Err(sqlx::Error::RowNotFound) => {

            let _ = search_bangumi_by_id(
                State(pool.clone()),
                Json(IDSearchQuery { 
                    id: Some(bangumi_tv_id) 
                })
            ).await;

            match sqlx::query!(
                "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
                bangumi_tv_id
            )
            .fetch_one(&pool)
            .await            
            {
                Ok(record) => record.id,
                Err(sqlx::Error::RowNotFound) => {
                    log::error!("Bangumi with external_id {} not found after search", bangumi_tv_id);
                    return Json(AddRecordResponse {
                        status: -2,
                        local_bangumi_id: Some(bangumi_tv_id),
                        bangumi_id: None,
                        recorder: None,
                        date: None,
                    });
                }
                Err(e) => {
                    log::error!("Failed to query bangumi_info_easy after search: {}", e);
                    return Json(AddRecordResponse {
                        status: -1,
                        local_bangumi_id: None,
                        bangumi_id: None,
                        recorder: None,
                        date: None,
                    });
                }
            }

            // log::error!("Bangumi with external_id {} not found", bangumi_tv_id);
            // return Json(AddRecordResponse {
            //     status: -2,
            //     local_bangumi_id: Some(bangumi_tv_id),
            //     bangumi_id: None,
            //     date: None,
            // });
        }
        Err(e) => {
            log::error!("Failed to query bangumi_info_easy: {}", e);
            return Json(AddRecordResponse {
                status: -1,
                local_bangumi_id: None,
                bangumi_id: None,
                recorder: None,
                date: None,
            });
        }
    };

    let token = params.token.as_ref().unwrap();

    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => {
            return Json(AddRecordResponse {
                status: -2, // Invalid token
                local_bangumi_id: None,
                bangumi_id: None,
                recorder: None,
                date: None,
            });
        }
    };

    match sqlx::query!(
        "SELECT id, recorder, is_delete FROM recordings WHERE user_id = ? AND bangumi_id = ? LIMIT 1",
        user_id,
        bangumi_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(k)) => {
            if k.is_delete == 0 {
                return Json(AddRecordResponse {
                    status: -3,
                    local_bangumi_id: Some(bangumi_id),
                    bangumi_id: Some(bangumi_tv_id),
                    recorder: Some(k.recorder.unwrap_or_default()),
                    date: None,
                });
            }
            let _ = sqlx::query!(
                "UPDATE recordings SET is_delete = 0, status = ?, recorder = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                user_status,
                recorder,
                k.id
            )
            .execute(&pool)
            .await;
            return Json(AddRecordResponse {
                status: 0,
                local_bangumi_id: Some(bangumi_id),
                bangumi_id: Some(bangumi_tv_id),
                recorder: Some(recorder),
                date: Some(chrono::Utc::now().naive_utc().date()),
            });
        }
        Err(e) => {
            log::error!("Failed to check existing record: {}", e);
            return Json(AddRecordResponse {
                status: -1,
                local_bangumi_id: None,
                bangumi_id: None,
                recorder: None,
                date: None,
            });
        }
        Ok(None) => {}
    }

    match sqlx::query!(
        "INSERT INTO recordings (user_id, bangumi_id, status, recorder) VALUES (?, ?, ?, ?)",
        user_id,
        bangumi_id,
        user_status,
        recorder
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Json(AddRecordResponse {
            status: 0,
            local_bangumi_id: Some(bangumi_id),
            bangumi_id: Some(bangumi_tv_id),
            recorder: Some(recorder),
            date: Some(chrono::Utc::now().naive_utc().date()),
        }),
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("uk_recordings_user_bangumi") {
                    return Json(AddRecordResponse {
                        status: -3,
                        local_bangumi_id: Some(bangumi_id),
                        bangumi_id: Some(bangumi_tv_id),
                        recorder: Some(recorder),
                        date: None,
                    });
                }
            }
            log::error!("Failed to add record: {}", e);
            Json(AddRecordResponse {
                status: -1,
                local_bangumi_id: None,
                bangumi_id: None,
                recorder: None,
                date: None,
            })
        }
    }
}