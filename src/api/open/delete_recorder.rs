use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

use super::api_token::check_api_token;

#[derive(Deserialize)]
pub struct DeleteRecorderQuery {
    pub bangumi_id: Option<u32>,
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct DeleteRecorderResponse {
    pub status: i32,
    pub message: Option<String>,
}

pub async fn delete_recorder(
    State(pool): State<MySqlPool>,
    Query(params): Query<DeleteRecorderQuery>,
) -> Result<Json<DeleteRecorderResponse>, StatusCode> {
    let bangumi_external_id = match params.bangumi_id {
        Some(id) => id,
        None => {
            return Ok(Json(DeleteRecorderResponse {
                status: -1,
                message: Some("Missing bangumi_id".to_string()),
            }));
        }
    };

    if params.token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = params.token.as_ref().unwrap();
    let user_id = match check_api_token(&pool, token).await {
        Some(id) => id,
        None => {
            return Err(StatusCode::UNAUTHORIZED);
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
            return Ok(Json(DeleteRecorderResponse {
                status: -2,
                message: Some("Bangumi not found".to_string()),
            }));
        }
        Err(e) => {
            log::error!("Failed to query bangumi_info_easy: {}", e);
            return Ok(Json(DeleteRecorderResponse {
                status: -2,
                message: Some("Database error".to_string()),
            }));
        }
    };

    match sqlx::query!(
        "UPDATE recordings SET is_delete = 1 WHERE user_id = ? AND bangumi_id = ? AND is_delete = 0",
        user_id,
        local_bangumi_id
    )
    .execute(&pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Ok(Json(DeleteRecorderResponse {
                    status: -3,
                    message: Some("Recording not found".to_string()),
                }))
            } else {
                Ok(Json(DeleteRecorderResponse {
                    status: 0,
                    message: Some("Deleted successfully".to_string()),
                }))
            }
        }
        Err(e) => {
            log::error!("Failed to delete recording: {}", e);
            Ok(Json(DeleteRecorderResponse {
                status: -2,
                message: Some("Failed to delete recording".to_string()),
            }))
        }
    }
}
