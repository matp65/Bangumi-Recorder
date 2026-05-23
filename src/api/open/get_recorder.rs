use axum::{
    Json, extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;

use super::api_token::check_api_token;

#[derive(Deserialize)]
pub struct GetRecorderQuery {
    pub bangumi_id: Option<u32>,
    pub local_bangumi_id: Option<u32>,
    pub other_id: Option<u32>,
    pub local_other_id: Option<u32>,
    pub token: Option<String>
}

#[derive(Serialize)]
pub struct GetRecorderResponse {
    pub status: i32,
    pub local_bangumi_id: Option<u32>,
    pub other_id: Option<u32>,
    pub local_other_id: Option<u32>,
    pub bangumi_id: Option<u32>,
    pub recorder: Option<String>,
    pub user_status: Option<i8>,
    pub is_delete: Option<bool>,
    pub date: Option<NaiveDate>
}

pub async fn get_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<GetRecorderQuery>
) -> Result<Json<GetRecorderResponse>, StatusCode> {

    if params.token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = params.token.as_ref().unwrap();

    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    if params.bangumi_id.is_none() && params.local_bangumi_id.is_none() && params.other_id.is_none() && params.local_other_id.is_none() {
        return Ok(Json(GetRecorderResponse {
            status: -2,
            local_bangumi_id: None,
            other_id: None,
            local_other_id: None,
            bangumi_id: None,
            recorder: None,
            user_status: None,
            is_delete: None,
            date: None,
        }));
    }

    if let Some(other_id) = params.other_id {
        match sqlx::query!(
            "SELECT id, bangumi_id, other_id, recorder, status, is_delete, updated_at FROM recordings WHERE user_id = ? AND other_id = ? AND is_delete = 0",
            user_id,
            other_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(r)) => {
                return Ok(Json(GetRecorderResponse {
                    status: 0,
                    local_bangumi_id: r.bangumi_id,
                    other_id: r.other_id,
                    local_other_id: Some(r.id),
                    bangumi_id: None,
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    is_delete: Some(r.is_delete != 0),
                    date: Some(r.updated_at.date()),
                }));
            }
            Ok(None) => {
                return Ok(Json(GetRecorderResponse {
                    status: 0,
                    local_bangumi_id: None,
                    other_id: Some(other_id),
                    local_other_id: None,
                    bangumi_id: None,
                    recorder: None,
                    user_status: None,
                    is_delete: None,
                    date: None,
                }));
            }
            Err(_) => {
                return Ok(Json(GetRecorderResponse {
                    status: -2,
                    local_bangumi_id: None,
                    other_id: Some(other_id),
                    local_other_id: None,
                    bangumi_id: None,
                    recorder: None,
                    user_status: None,
                    is_delete: None,
                    date: None,
                }));
            }
        }
    }

    if let Some(local_other_id) = params.local_other_id {
        match sqlx::query!(
            "SELECT id, bangumi_id, other_id, recorder, status, is_delete, updated_at FROM recordings WHERE user_id = ? AND id = ? AND is_delete = 0",
            user_id,
            local_other_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(r)) => {
                return Ok(Json(GetRecorderResponse {
                    status: 0,
                    local_bangumi_id: r.bangumi_id,
                    other_id: r.other_id,
                    local_other_id: Some(r.id),
                    bangumi_id: None,
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    is_delete: Some(r.is_delete != 0),
                    date: Some(r.updated_at.date()),
                }));
            }
            Ok(None) => {
                return Ok(Json(GetRecorderResponse {
                    status: 0,
                    local_bangumi_id: None,
                    other_id: None,
                    local_other_id: Some(local_other_id),
                    bangumi_id: None,
                    recorder: None,
                    user_status: None,
                    is_delete: None,
                    date: None,
                }));
            }
            Err(_) => {
                return Ok(Json(GetRecorderResponse {
                    status: -2,
                    local_bangumi_id: None,
                    other_id: None,
                    local_other_id: Some(local_other_id),
                    bangumi_id: None,
                    recorder: None,
                    user_status: None,
                    is_delete: None,
                    date: None,
                }));
            }
        }
    }

    let bangumi_external_id = match params.bangumi_id {
        Some(id) => id,
        None => {
            let local_bangumi_id = match params.local_bangumi_id {
                Some(id) => id,
                None => {
                    return Ok(Json(GetRecorderResponse {
                        status: -2,
                        local_bangumi_id: None,
                        other_id: None,
                        local_other_id: None,
                        bangumi_id: None,
                        recorder: None,
                        user_status: None,
                        is_delete: None,
                        date: None,
                    }));
                }
            };

            match sqlx::query!(
                "SELECT recorder, status, is_delete, updated_at FROM recordings WHERE user_id = ? AND bangumi_id = ? AND is_delete = 0",
                user_id,
                local_bangumi_id
            )
            .fetch_optional(&pool)
            .await
            {
                Ok(Some(r)) => {
                    let external_id = sqlx::query!(
                        "SELECT external_id FROM bangumi_info_easy WHERE id = ?",
                        local_bangumi_id
                    )
                    .fetch_optional(&pool)
                    .await;
                    let bangumi_id = match external_id {
                        Ok(Some(ext)) => {
                            ext.external_id.parse::<u32>().ok()
                        }
                        _ => None,
                    };
                    return Ok(Json(GetRecorderResponse {
                        status: 0,
                        local_bangumi_id: Some(local_bangumi_id),
                        other_id: None,
                        local_other_id: None,
                        bangumi_id,
                        recorder: r.recorder,
                        user_status: Some(r.status),
                        is_delete: Some(r.is_delete != 0),
                        date: Some(r.updated_at.date()),
                    }));
                }
                Ok(None) => {
                    return Ok(Json(GetRecorderResponse {
                        status: 0,
                        local_bangumi_id: Some(local_bangumi_id),
                        other_id: None,
                        local_other_id: None,
                        bangumi_id: None,
                        recorder: None,
                        user_status: None,
                        is_delete: None,
                        date: None,
                    }));
                }
                Err(_) => {
                    return Ok(Json(GetRecorderResponse {
                        status: -2,
                        local_bangumi_id: Some(local_bangumi_id),
                        other_id: None,
                        local_other_id: None,
                        bangumi_id: None,
                        recorder: None,
                        user_status: None,
                        is_delete: None,
                        date: None,
                    }));
                }
            }
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
            return Ok(Json(GetRecorderResponse { 
                status: -2,
                local_bangumi_id: None,
                other_id: None,
                local_other_id: None,
                bangumi_id: Some(bangumi_external_id),
                recorder: None,
                user_status: None,
                is_delete: None,
                date: None
            }));
        }
        Err(e) => {
            log::error!("Failed to query bangumi_info_easy: {}", e);
            return Ok(Json(GetRecorderResponse { 
                status: -2,
                local_bangumi_id: None,
                other_id: None,
                local_other_id: None,
                bangumi_id: Some(bangumi_external_id),
                recorder: None,
                user_status: None,
                is_delete: None,
                date: None
            }));
        }
    };

    match sqlx::query!(
        "SELECT recorder, status, is_delete, updated_at FROM recordings WHERE user_id = ? AND bangumi_id = ? AND is_delete = 0",
        user_id,
        local_bangumi_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => {
            Ok(Json(GetRecorderResponse { 
                status: 0, 
                local_bangumi_id: Some(local_bangumi_id),
                other_id: None,
                local_other_id: None,
                bangumi_id: Some(bangumi_external_id),
                recorder: r.recorder,
                user_status: Some(r.status),
                is_delete: Some(r.is_delete != 0),
                date: Some(r.updated_at.date()),
            }))
        }
        Ok(None) => {
            Ok(Json(GetRecorderResponse { 
                status: 0,
                local_bangumi_id: Some(local_bangumi_id),
                other_id: None,
                local_other_id: None,
                bangumi_id: Some(bangumi_external_id),
                recorder: None,
                user_status: None,
                is_delete: None,
                date: None,
            }))
        }
        Err(_) => {
            Ok(Json(GetRecorderResponse {
                status: -2,
                local_bangumi_id: Some(local_bangumi_id),
                other_id: None,
                local_other_id: None,
                bangumi_id: Some(bangumi_external_id),
                recorder: None,
                user_status: None,
                is_delete: None,
                date: None,
            }))
        }
    }
}
