use axum::{
    Json,
    extract::{Extension, State},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{MySql, QueryBuilder, mysql::MySqlPool};

use crate::api::imdb::{IMDB_SOURCE, normalize_imdb_id};
use crate::api::logs::{LogTarget, write_recording_log};
use crate::auth_bearer::AuthUser;

#[derive(Deserialize)]
pub struct UpdateRecorderQuery {
    pub bangumi_id: Option<i32>,
    pub other_id: Option<u32>,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub imdb_id: Option<String>,
    pub recorder: Option<String>,
    pub user_status: Option<i32>,
    pub other_title: Option<String>,
    pub other_description: Option<String>,
    pub other_cover: Option<String>,
    pub other_max_number: Option<i32>,
    pub other_status: Option<i32>,
}

#[derive(Serialize)]
pub struct UpdateRecorderResponse {
    pub status: i32,
    pub message: Option<String>,
}

enum UpdateTarget {
    Bangumi(u32),
    Imdb(u32),
    Other(u32),
}

fn log_target(target: &UpdateTarget) -> LogTarget {
    match target {
        UpdateTarget::Bangumi(id) => LogTarget::Bangumi(*id),
        UpdateTarget::Imdb(id) => LogTarget::Imdb(*id),
        UpdateTarget::Other(id) => LogTarget::Other(*id),
    }
}

fn has_other_metadata(params: &UpdateRecorderQuery) -> bool {
    params.other_title.is_some()
        || params.other_description.is_some()
        || params.other_cover.is_some()
        || params.other_max_number.is_some()
        || params.other_status.is_some()
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
    let target_count = [
        params.bangumi_id.is_some(),
        normalized_imdb_id.is_some(),
        params.other_id.is_some(),
    ]
    .into_iter()
    .filter(|v| *v)
    .count();

    if target_count != 1
        || (params.recorder.is_none()
            && params.user_status.is_none()
            && !has_other_metadata(params))
    {
        return Err(UpdateRecorderResponse {
            status: -1,
            message: Some("Missing required parameters".to_string()),
        });
    }

    if let Some(other_id) = params.other_id {
        return Ok(UpdateTarget::Other(other_id));
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
        UpdateTarget::Other(other_id) => {
            sqlx::query_scalar!(
                "SELECT id FROM recordings WHERE user_id = ? AND other_id = ? AND is_delete = 0",
                auth_user.user_id,
                other_id
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

    let old_recording = match sqlx::query!(
        "SELECT recorder, status FROM recordings WHERE id = ?",
        recording
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            log::error!("Failed to query recording state: {}", e);
            return Json(UpdateRecorderResponse {
                status: -2,
                message: Some("Database error".to_string()),
            });
        }
    };

    if params.recorder.is_some() || params.user_status.is_some() {
        let mut qb = QueryBuilder::<MySql>::new("UPDATE recordings SET ");
        {
            let mut separated = qb.separated(", ");
            if let Some(recorder) = params.recorder.as_ref() {
                separated
                    .push("recorder = ")
                    .push_bind_unseparated(recorder);
            }
            if let Some(status_val) = params.user_status {
                separated
                    .push("status = ")
                    .push_bind_unseparated(status_val);
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
    }

    if let UpdateTarget::Other(other_id) = &target
        && has_other_metadata(&params)
    {
        let old_other = match sqlx::query!(
            "SELECT name, description, cover_url, max_number, status FROM other_recorders WHERE id = ?",
            other_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(row) => row,
            Err(e) => {
                log::error!("Failed to query custom item state: {}", e);
                return Json(UpdateRecorderResponse {
                    status: -2,
                    message: Some("Database error".to_string()),
                });
            }
        };

        let owner = sqlx::query_scalar!(
            "SELECT add_user FROM other_recorders WHERE id = ?",
            other_id
        )
        .fetch_optional(&pool)
        .await;

        match owner {
            Ok(Some(Some(add_user))) if add_user as i64 == auth_user.user_id => {}
            Ok(Some(_)) => {
                return Json(UpdateRecorderResponse {
                    status: -4,
                    message: Some("Custom item is not editable by this user".to_string()),
                });
            }
            Ok(None) => {
                return Json(UpdateRecorderResponse {
                    status: -3,
                    message: Some("Custom item not found".to_string()),
                });
            }
            Err(e) => {
                log::error!("Failed to query custom item owner: {}", e);
                return Json(UpdateRecorderResponse {
                    status: -2,
                    message: Some("Database error".to_string()),
                });
            }
        }

        let mut qb = QueryBuilder::<MySql>::new("UPDATE other_recorders SET ");
        {
            let mut separated = qb.separated(", ");
            if let Some(title) = params.other_title.as_ref() {
                separated.push("name = ").push_bind_unseparated(title);
            }
            if let Some(description) = params.other_description.as_ref() {
                separated
                    .push("description = ")
                    .push_bind_unseparated(description);
            }
            if let Some(cover) = params.other_cover.as_ref() {
                separated.push("cover_url = ").push_bind_unseparated(cover);
            }
            if let Some(max_number) = params.other_max_number {
                separated
                    .push("max_number = ")
                    .push_bind_unseparated(max_number);
            }
            if let Some(status) = params.other_status {
                separated.push("status = ").push_bind_unseparated(status);
            }
        }
        qb.push(" WHERE id = ").push_bind(other_id);

        if let Err(e) = qb.build().execute(&pool).await {
            log::warn!("Failed to update custom item {}: {:?}", other_id, e);
            return Json(UpdateRecorderResponse {
                status: -2,
                message: Some("Database error".to_string()),
            });
        }

        if let Some(old_other) = old_other {
            let mut changes = Vec::new();
            if let Some(title) = params.other_title.as_ref()
                && old_other.name.as_deref() != Some(title.as_str())
            {
                changes.push(json!({ "field": "name", "old": old_other.name, "new": title }));
            }
            if let Some(description) = params.other_description.as_ref()
                && old_other.description.as_deref() != Some(description.as_str())
            {
                changes.push(json!({ "field": "description", "old": old_other.description, "new": description }));
            }
            if let Some(cover) = params.other_cover.as_ref()
                && old_other.cover_url.as_deref() != Some(cover.as_str())
            {
                changes.push(
                    json!({ "field": "cover_url", "old": old_other.cover_url, "new": cover }),
                );
            }
            if let Some(max_number) = params.other_max_number
                && old_other.max_number != Some(max_number)
            {
                changes.push(json!({ "field": "max_number", "old": old_other.max_number, "new": max_number }));
            }
            if let Some(status) = params.other_status
                && old_other.status != Some(status as i8)
            {
                changes.push(json!({ "field": "status", "old": old_other.status, "new": status }));
            }
            if !changes.is_empty() {
                write_recording_log(
                    &pool,
                    recording,
                    Some(auth_user.user_id),
                    log_target(&target),
                    "other_metadata_changed",
                    None,
                    None,
                    None,
                    Some(json!({ "changes": changes })),
                )
                .await;
            }
        }
    }

    if let Some(recorder) = params.recorder.as_ref()
        && old_recording.recorder.as_deref() != Some(recorder.as_str())
    {
        write_recording_log(
            &pool,
            recording,
            Some(auth_user.user_id),
            log_target(&target),
            "recorder_changed",
            Some("recorder"),
            old_recording.recorder.map(|v| json!(v)),
            Some(json!(recorder)),
            None,
        )
        .await
    }

    if let Some(user_status) = params.user_status
        && old_recording.status != user_status as i8
    {
        write_recording_log(
            &pool,
            recording,
            Some(auth_user.user_id),
            log_target(&target),
            "status_changed",
            Some("status"),
            Some(json!(old_recording.status)),
            Some(json!(user_status)),
            None,
        )
        .await
    }

    Json(UpdateRecorderResponse {
        status: 0,
        message: Some("Updated successfully".to_string()),
    })
}
