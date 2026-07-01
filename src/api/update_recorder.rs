use axum::{
    Json,
    extract::{Extension, State},
};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, QueryBuilder, mysql::MySqlPool};

use crate::api::imdb::{IMDB_SOURCE, normalize_imdb_id};
use crate::auth_bearer::AuthUser;

#[derive(Deserialize)]
pub struct UpdateRecorderQuery {
    pub bangumi_id: Option<i32>,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub imdb_id: Option<String>,
    pub recorder: Option<String>,
    pub user_status: Option<i32>,
}

#[derive(Serialize)]
pub struct UpdateRecorderResponse {
    pub status: i32,
    pub message: Option<String>,
}

enum UpdateTarget {
    Bangumi(u32),
    Imdb(u32),
}

async fn resolve_target(
    pool: &MySqlPool,
    params: &UpdateRecorderQuery,
) -> Result<UpdateTarget, UpdateRecorderResponse> {
    let normalized_imdb_id = params
        .imdb_id
        .as_deref()
        .or_else(|| {
            if params
                .source
                .as_deref()
                .unwrap_or_default()
                .eq_ignore_ascii_case(IMDB_SOURCE)
            {
                params.external_id.as_deref()
            } else {
                None
            }
        })
        .and_then(normalize_imdb_id);
    let target_count = [params.bangumi_id.is_some(), normalized_imdb_id.is_some()]
        .into_iter()
        .filter(|v| *v)
        .count();

    if target_count != 1 || (params.recorder.is_none() && params.user_status.is_none()) {
        return Err(UpdateRecorderResponse {
            status: -1,
            message: Some("Missing required parameters".to_string()),
        });
    }

    if let Some(bangumi_id) = params.bangumi_id {
        let row = sqlx::query!(
            "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
            bangumi_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to query bangumi_info_easy: {}", e);
            UpdateRecorderResponse {
                status: -2,
                message: Some("Database error".to_string()),
            }
        })?;

        return row
            .map(|r| UpdateTarget::Bangumi(r.id))
            .ok_or_else(|| UpdateRecorderResponse {
                status: -2,
                message: Some("Bangumi not found".to_string()),
            });
    }

    let imdb_id = normalized_imdb_id.unwrap();
    let row = sqlx::query!(
        "SELECT id FROM external_media WHERE source = ? AND external_id = ?",
        IMDB_SOURCE,
        imdb_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        log::error!("Failed to query external_media: {}", e);
        UpdateRecorderResponse {
            status: -2,
            message: Some("Database error".to_string()),
        }
    })?;

    row.map(|r| UpdateTarget::Imdb(r.id))
        .ok_or_else(|| UpdateRecorderResponse {
            status: -2,
            message: Some("IMDb title not found".to_string()),
        })
}

pub async fn update_user_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<UpdateRecorderQuery>,
) -> Json<UpdateRecorderResponse> {
    let target = match resolve_target(&pool, &params).await {
        Ok(target) => target,
        Err(response) => return Json(response),
    };

    let recording = match &target {
        UpdateTarget::Bangumi(local_id) => {
            sqlx::query_scalar!(
                "SELECT id FROM recordings WHERE user_id = ? AND bangumi_id = ? AND is_delete = 0",
                auth_user.user_id,
                local_id
            )
            .fetch_optional(&pool)
            .await
        }
        UpdateTarget::Imdb(local_id) => {
            sqlx::query_scalar!(
                "SELECT id FROM recordings WHERE user_id = ? AND external_media_id = ? AND is_delete = 0",
                auth_user.user_id,
                local_id
            )
            .fetch_optional(&pool)
            .await
        }
    };

    let recording = match recording {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Json(UpdateRecorderResponse {
                status: -3,
                message: Some("Recording not found".to_string()),
            });
        }
        Err(e) => {
            log::error!("Failed to query recording: {}", e);
            return Json(UpdateRecorderResponse {
                status: -2,
                message: Some("Database error".to_string()),
            });
        }
    };

    let mut qb = QueryBuilder::<MySql>::new("UPDATE recordings SET ");
    {
        let mut separated = qb.separated(", ");
        if let Some(recorder) = params.recorder.as_ref() {
            separated.push("recorder = ").push_bind(recorder);
        }
        if let Some(status_val) = params.user_status {
            separated.push("status = ").push_bind(status_val);
        }
        separated.push("updated_at = CURRENT_TIMESTAMP");
    }
    qb.push(" WHERE id = ").push_bind(recording);

    if let Err(e) = qb.build().execute(&pool).await {
        log::warn!("Failed to update recording {}: {:?}", recording, e);
        return Json(UpdateRecorderResponse {
            status: -2,
            message: Some("Database error".to_string()),
        });
    }

    if let (UpdateTarget::Bangumi(local_id), Some(recorder)) = (&target, params.recorder.as_ref())
        && let Err(e) = sqlx::query!(
            "INSERT INTO recording_logs (recording_id, user_id, bangumi_id, recorder) VALUES (?, ?, ?, ?)",
            recording,
            auth_user.user_id,
            local_id,
            recorder
        )
        .execute(&pool)
        .await
    {
        log::warn!("Failed to write recording log: {:?}", e);
    }

    Json(UpdateRecorderResponse {
        status: 0,
        message: Some("Updated successfully".to_string()),
    })
}
