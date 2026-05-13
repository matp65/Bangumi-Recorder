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
    pub recorder: Option<String>
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

    if params.bangumi_id.is_none() || params.recorder.is_none() {
        return Json(UpdateRecorderResponse {
            status: -1,
            message: Some("Missing required parameters".to_string())
        });
    }

    let bangumi_id = params.bangumi_id.unwrap();
    let recorder = params.recorder.as_ref().unwrap();

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

    match sqlx::query!(
        "UPDATE recordings SET recorder = ? WHERE bangumi_id = ? AND user_id = ?",
        recorder,
        local_bangumi_id,
        auth_user.user_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Json(UpdateRecorderResponse {
            status: 0,
            message: Some("Recorder updated successfully".to_string())
        }),
        Err(e) => {
            log::warn!("Failed to update recorder for bangumi_id {}: {:?}", bangumi_id, e);
            Json(UpdateRecorderResponse {
                status: -2,
                message: Some(format!("Database error"))
            })
        }
    }
}