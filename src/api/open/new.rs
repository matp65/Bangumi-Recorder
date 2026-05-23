use axum::{
    extract::{Query, State},
    http::StatusCode,
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
    pub other_id: Option<u32>,
    pub local_other_id: Option<u32>,
    pub bangumi_id: Option<u32>,
    pub recorder: Option<String>,
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct AddRecordQuery {
    pub bangumi_id: Option<u32>,
    pub other_id: Option<u32>,
    pub other_title: Option<String>,
    pub other_description: Option<String>,
    pub other_cover: Option<String>,
    pub other_max_number: Option<i32>,
    pub other_status: Option<i32>,
    pub user_status: Option<i32>,
    pub recorder: Option<String>,
    pub token: Option<String>,
}

pub async fn add_record_open(
    State(pool): State<MySqlPool>,
    Query(params): Query<AddRecordQuery>,
) -> Result<Json<AddRecordResponse>, StatusCode> {
    if params.token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if params.bangumi_id.is_none() && params.other_id.is_none() {
        return Ok(Json(AddRecordResponse {
            status: -1,
            local_bangumi_id: None,
            other_id: None,
            local_other_id: None,
            bangumi_id: None,
            recorder: None,
            date: None,
        }));
    }

    if params.bangumi_id.is_some() && params.other_id.is_some() {
        return Ok(Json(AddRecordResponse {
            status: -1,
            local_bangumi_id: None,
            other_id: None,
            local_other_id: None,
            bangumi_id: None,
            recorder: None,
            date: None,
        }));
    }
    
    let add_record_type: String = if params.bangumi_id.is_some() {
        "is_bangumi".to_string()
    } else if params.other_id.is_some() {
        "is_other".to_string()
    } else {
        return Ok(Json(AddRecordResponse {
            status: -1,
            local_bangumi_id: None,
            other_id: None,
            local_other_id: None,
            bangumi_id: None,
            recorder: None,
            date: None
            }));
    };

    let token = params.token.as_ref().unwrap();

    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let bangumi_tv_id = params.bangumi_id.as_ref().unwrap_or_else(|| &0);
    let user_status = params.user_status.as_ref().unwrap_or_else(|| &0);
    let recorder = params.recorder.clone().unwrap_or_default();

    if add_record_type == "is_bangumi" {
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
                        id: Some(*bangumi_tv_id) 
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
                        return Ok(Json(AddRecordResponse {
                            status: -2,
                            local_bangumi_id: Some(*bangumi_tv_id),
                            other_id: None,
                            local_other_id: None,
                            bangumi_id: None,
                            recorder: None,
                            date: None,
                        }));
                    }
                    Err(e) => {
                        log::error!("Failed to query bangumi_info_easy after search: {}", e);
                        return Ok(Json(AddRecordResponse {
                            status: -1,
                            local_bangumi_id: None,
                            other_id: None,
                            local_other_id: None,
                            bangumi_id: None,
                            recorder: None,
                            date: None,
                        }));
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
                return Ok(Json(AddRecordResponse {
                    status: -1,
                    local_bangumi_id: None,
other_id: None,
                                local_other_id: None,
                                bangumi_id: None,
                    recorder: None,
                    date: None,
                }));
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
                    return Ok(Json(AddRecordResponse {
                        status: -3,
                        local_bangumi_id: Some(bangumi_id),
other_id: None,
                                local_other_id: None,
                                bangumi_id: Some(*bangumi_tv_id),
                        recorder: Some(k.recorder.unwrap_or_default()),
                        date: None,
                    }));
                }
                let _ = sqlx::query!(
                    "UPDATE recordings SET is_delete = 0, status = ?, recorder = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                    user_status,
                    recorder,
                    k.id
                )
                .execute(&pool)
                .await;
                return Ok(Json(AddRecordResponse {
                    status: 0,
                    local_bangumi_id: Some(bangumi_id),
other_id: None,
                                    local_other_id: None,
                                    bangumi_id: Some(*bangumi_tv_id),
                    recorder: Some(recorder),
                    date: Some(chrono::Utc::now().naive_utc().date()),
                }));
            }
            Err(e) => {
                log::error!("Failed to check existing record: {}", e);
                return Ok(Json(AddRecordResponse {
                    status: -1,
                    local_bangumi_id: None,
other_id: None,
                                local_other_id: None,
                                bangumi_id: None,
                    recorder: None,
                    date: None,
                }));
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
            Ok(_) => Ok(Json(AddRecordResponse {
                status: 0,
                local_bangumi_id: Some(bangumi_id),
                other_id: None,
                local_other_id: None,
                bangumi_id: Some(*bangumi_tv_id),
                recorder: Some(recorder),
                date: Some(chrono::Utc::now().naive_utc().date()),
            })),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.constraint() == Some("uk_recordings_user_bangumi") {
                        return Ok(Json(AddRecordResponse {
                            status: -3,
                            local_bangumi_id: Some(bangumi_id),
                            other_id: None,
                            local_other_id: None,
                            bangumi_id: Some(*bangumi_tv_id),
                            recorder: Some(recorder),
                            date: None,
                        }));
                    }
                }
                log::error!("Failed to add record: {}", e);
                Ok(Json(AddRecordResponse {
                    status: -1,
                    local_bangumi_id: None,
                    other_id: None,
                    local_other_id: None,
                    bangumi_id: None,
                    recorder: None,
                    date: None,
                }))
            }
        }
    } else if add_record_type == "is_other" {
        let other_recorder_id: u32;

        if let Some(eid) = params.other_id {
            other_recorder_id = eid;
        } else {
            let insert_result = if params.other_title.is_some() {
                sqlx::query!(
                    "INSERT INTO other_recorders (name, description, cover_url, max_number, status, add_user) VALUES (?, ?, ?, ?, ?, ?)",
                    params.other_title,
                    params.other_description,
                    params.other_cover,
                    params.other_max_number,
                    params.other_status,
                    user_id
                )
                .execute(&pool)
                .await
            } else {
                sqlx::query!(
                    "INSERT INTO other_recorders (add_user) VALUES (?)",
                    user_id
                )
                .execute(&pool)
                .await
            };

            other_recorder_id = match insert_result {
                Ok(result) => result.last_insert_id() as u32,
                Err(e) => {
                    log::error!("Failed to add other record: {}", e);
                    return Ok(Json(AddRecordResponse {
                        status: -1,
                        local_bangumi_id: None,
                        other_id: None,
                        local_other_id: None,
                        bangumi_id: None,
                        recorder: None,
                        date: None,
                    }));
                }
            };
        }

        match sqlx::query!(
            "SELECT id, recorder, is_delete FROM recordings WHERE user_id = ? AND other_id = ? AND is_delete = 0 LIMIT 1",
            user_id,
            other_recorder_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(k)) => {
                if k.is_delete == 0 {
                    return Ok(Json(AddRecordResponse {
                        status: -3,
                        local_bangumi_id: None,
                        other_id: Some(other_recorder_id),
                        local_other_id: Some(k.id),
                        bangumi_id: None,
                        recorder: Some(k.recorder.unwrap_or_default()),
                        date: None,
                    }));
                }
                let _ = sqlx::query!(
                    "UPDATE recordings SET is_delete = 0, status = ?, recorder = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                    user_status,
                    recorder,
                    k.id
                )
                .execute(&pool)
                .await;
                return Ok(Json(AddRecordResponse {
                    status: 0,
                    local_bangumi_id: None,
                    other_id: Some(other_recorder_id),
                    local_other_id: Some(k.id),
                    bangumi_id: None,
                    recorder: Some(recorder),
                    date: Some(chrono::Utc::now().naive_utc().date()),
                }));
            }
            Err(e) => {
                log::error!("Failed to check existing other record: {}", e);
                return Ok(Json(AddRecordResponse {
                    status: -1,
                    local_bangumi_id: None,
                    other_id: None,
                    local_other_id: None,
                    bangumi_id: None,
                    recorder: None,
                    date: None,
                }));
            }
            Ok(None) => {}
        }

        match sqlx::query!(
            "INSERT INTO recordings (user_id, other_id, status, recorder) VALUES (?, ?, ?, ?)",
            user_id,
            other_recorder_id,
            user_status,
            recorder
        )
        .execute(&pool)
        .await
        {
            Ok(result) => {
                let local_other_id = result.last_insert_id() as u32;
                Ok(Json(AddRecordResponse {
                    status: 0,
                    local_bangumi_id: None,
                    other_id: Some(other_recorder_id),
                    local_other_id: Some(local_other_id),
                    bangumi_id: None,
                    recorder: Some(recorder),
                    date: Some(chrono::Utc::now().naive_utc().date()),
                }))
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.constraint() == Some("uk_recordings_user_other") {
                        let dup = sqlx::query!(
                            "SELECT id, recorder FROM recordings WHERE user_id = ? AND other_id = ? AND is_delete = 0",
                            user_id,
                            other_recorder_id
                        )
                        .fetch_one(&pool)
                        .await;
                        if let Ok(d) = dup {
                            return Ok(Json(AddRecordResponse {
                                status: -3,
                                local_bangumi_id: None,
                                other_id: Some(other_recorder_id),
                                local_other_id: Some(d.id),
                                bangumi_id: None,
                                recorder: d.recorder,
                                date: None,
                            }));
                        }
                    }
                }
                log::error!("Failed to add recording for other record: {}", e);
                Ok(Json(AddRecordResponse {
                    status: -1,
                    local_bangumi_id: None,
                    other_id: None,
                    local_other_id: None,
                    bangumi_id: None,
                    recorder: None,
                    date: None,
                }))
            }
        }
    } else {
        Ok(Json(AddRecordResponse {
            status: -1,
            local_bangumi_id: None,
            other_id: None,
            local_other_id: None,
            bangumi_id: None,
            recorder: None,
            date: None,
        }))
    }
}