use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};

use sqlx::mysql::MySqlPool;

use crate::auth_bearer::AuthUser;

#[derive(Deserialize)]
pub struct UpdateRecorderQuery {
    pub bangumi_id: Option<i32>,
    pub recorder: Option<String>,
    pub user_status: Option<i32>,
}

#[derive(Serialize)]
pub struct UpdateRecorderResponse {
    pub status: i32,
    pub message: Option<String>
}

pub async fn update_user_recorder (
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<UpdateRecorderQuery>,
) -> Json<UpdateRecorderResponse> {

    if params.bangumi_id.is_none() || (params.recorder.is_none() && params.user_status.is_none()) {
        return Json(UpdateRecorderResponse {
            status: -1,
            message: Some("Missing required parameters".to_string())
        });
    }

    let bangumi_id = params.bangumi_id.unwrap();

    let temp_local_bangumi_id = sqlx::query!(
        "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
        bangumi_id
    )
    .fetch_optional(&pool)
    .await;

    let local_bangumi_id = match temp_local_bangumi_id {
        Ok(Some(record)) => record.id,
        Ok(None) => {
            log::error!("Bangumi with external_id {} not found", bangumi_id);
            return Json(UpdateRecorderResponse {
                status: -2,
                message: Some("Bangumi not found".to_string())
            });
        }
        Err(e) => {
            log::error!("Failed to query bangumi_info_easy: {}", e);
            return Json(UpdateRecorderResponse {
                status: -2,
                message: Some("Database error".to_string())
            });
        }
    };

    let recording = match sqlx::query!(
        "SELECT id FROM recordings WHERE user_id = ? AND bangumi_id = ? AND is_delete = 0",
        auth_user.user_id,
        local_bangumi_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Json(UpdateRecorderResponse {
                status: -3,
                message: Some("Recording not found".to_string())
            });
        }
        Err(e) => {
            log::error!("Failed to query recording: {}", e);
            return Json(UpdateRecorderResponse {
                status: -2,
                message: Some("Database error".to_string())
            });
        }
    };

    if let Some(recorder) = params.recorder.as_ref() {
        match sqlx::query!(
            "UPDATE recordings SET recorder = ? WHERE id = ?",
            recorder,
            recording.id
        )
        .execute(&pool)
        .await
        {
            Ok(_) => {
                let _ = sqlx::query!(
                    "INSERT INTO recording_logs (recording_id, user_id, bangumi_id, recorder) VALUES (?, ?, ?, ?)",
                    recording.id,
                    auth_user.user_id,
                    local_bangumi_id,
                    recorder
                )
                .execute(&pool)
                .await;
            }
            Err(e) => {
                log::warn!("Failed to update recorder for bangumi_id {}: {:?}", bangumi_id, e);
                return Json(UpdateRecorderResponse {
                    status: -2,
                    message: Some("Database error".to_string())
                });
            }
        }
    }

    if let Some(status_val) = params.user_status {
        match sqlx::query!(
            "UPDATE recordings SET status = ? WHERE id = ?",
            status_val,
            recording.id
        )
        .execute(&pool)
        .await
        {
            Ok(_) => {}
            Err(e) => {
                log::warn!("Failed to update status for bangumi_id {}: {:?}", bangumi_id, e);
                return Json(UpdateRecorderResponse {
                    status: -2,
                    message: Some("Database error".to_string())
                });
            }
        }
    }

    Json(UpdateRecorderResponse {
        status: 0,
        message: Some("Updated successfully".to_string())
    })
}
