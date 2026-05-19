use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

use crate::auth_bearer::AuthUser;

#[derive(Deserialize)]
pub struct DeleteRecorderQuery {
    pub bangumi_id: Option<u32>,
}

#[derive(Serialize)]
pub struct DeleteRecorderResponse {
    pub status: i32,
    pub message: Option<String>,
}

pub async fn delete_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<DeleteRecorderQuery>,
) -> Json<DeleteRecorderResponse> {
    let bangumi_external_id = match params.bangumi_id {
        Some(id) => id,
        None => {
            return Json(DeleteRecorderResponse {
                status: -1,
                message: Some("Missing bangumi_id".to_string()),
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
            return Json(DeleteRecorderResponse {
                status: -2,
                message: Some("Bangumi not found".to_string()),
            });
        }
        Err(e) => {
            log::error!("Failed to query bangumi_info_easy: {}", e);
            return Json(DeleteRecorderResponse {
                status: -2,
                message: Some("Database error".to_string()),
            });
        }
    };

    match sqlx::query!(
        "UPDATE recordings SET is_delete = 1 WHERE user_id = ? AND bangumi_id = ? AND is_delete = 0",
        auth_user.user_id,
        local_bangumi_id
    )
    .execute(&pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Json(DeleteRecorderResponse {
                    status: -3,
                    message: Some("Recording not found".to_string()),
                })
            } else {
                Json(DeleteRecorderResponse {
                    status: 0,
                    message: Some("Deleted successfully".to_string()),
                })
            }
        }
        Err(e) => {
            log::error!("Failed to delete recording: {}", e);
            Json(DeleteRecorderResponse {
                status: -2,
                message: Some("Failed to delete recording".to_string()),
            })
        }
    }
}
