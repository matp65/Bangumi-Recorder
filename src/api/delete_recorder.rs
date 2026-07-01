use axum::{
    Json,
    extract::{Extension, State},
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

use crate::api::imdb::{IMDB_SOURCE, normalize_imdb_id};
use crate::auth_bearer::AuthUser;

#[derive(Deserialize)]
pub struct DeleteRecorderQuery {
    pub bangumi_id: Option<u32>,
    pub source: Option<String>,
    pub external_id: Option<String>,
    pub imdb_id: Option<String>,
    pub other_id: Option<u32>,
    pub local_other_id: Option<u32>,
    pub hard_delete: Option<bool>,
}

#[derive(Serialize)]
pub struct DeleteRecorderResponse {
    pub status: i32,
    pub message: Option<String>,
}

fn response(status: i32, message: &str) -> Json<DeleteRecorderResponse> {
    Json(DeleteRecorderResponse {
        status,
        message: Some(message.to_string()),
    })
}

pub async fn delete_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<DeleteRecorderQuery>,
) -> Json<DeleteRecorderResponse> {
    let hard_delete = params.hard_delete.unwrap_or(false);

    if let Some(local_other_id) = params.local_other_id {
        let result = if hard_delete {
            sqlx::query!(
                "DELETE FROM recordings WHERE user_id = ? AND id = ?",
                auth_user.user_id,
                local_other_id
            )
            .execute(&pool)
            .await
        } else {
            sqlx::query!(
                "UPDATE recordings SET is_delete = 1 WHERE user_id = ? AND id = ? AND is_delete = 0",
                auth_user.user_id,
                local_other_id
            )
            .execute(&pool)
            .await
        };
        return delete_by_sql_result(result, hard_delete);
    }

    if let Some(other_id) = params.other_id {
        let result = if hard_delete {
            sqlx::query!(
                "DELETE FROM recordings WHERE user_id = ? AND other_id = ?",
                auth_user.user_id,
                other_id
            )
            .execute(&pool)
            .await
        } else {
            sqlx::query!(
                "UPDATE recordings SET is_delete = 1 WHERE user_id = ? AND other_id = ? AND is_delete = 0",
                auth_user.user_id,
                other_id
            )
            .execute(&pool)
            .await
        };
        return delete_by_sql_result(result, hard_delete);
    }

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

    if target_count != 1 {
        return response(-1, "Missing media id");
    }

    if let Some(bangumi_id) = params.bangumi_id {
        let local_id = match sqlx::query!(
            "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
            bangumi_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(record)) => record.id,
            Ok(None) => return response(-2, "Bangumi not found"),
            Err(e) => {
                log::error!("Failed to query bangumi_info_easy: {}", e);
                return response(-2, "Database error");
            }
        };

        let result = if hard_delete {
            sqlx::query!(
                "DELETE FROM recordings WHERE user_id = ? AND bangumi_id = ?",
                auth_user.user_id,
                local_id
            )
            .execute(&pool)
            .await
        } else {
            sqlx::query!(
                "UPDATE recordings SET is_delete = 1 WHERE user_id = ? AND bangumi_id = ? AND is_delete = 0",
                auth_user.user_id,
                local_id
            )
            .execute(&pool)
            .await
        };
        return delete_by_sql_result(result, hard_delete);
    }

    let imdb_id = match normalized_imdb_id {
        Some(id) => id,
        None => return response(-1, "Invalid IMDb id"),
    };

    let local_id = match sqlx::query!(
        "SELECT id FROM external_media WHERE source = ? AND external_id = ?",
        IMDB_SOURCE,
        imdb_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(record)) => record.id,
        Ok(None) => return response(-2, "IMDb title not found"),
        Err(e) => {
            log::error!("Failed to query external_media: {}", e);
            return response(-2, "Database error");
        }
    };

    let result = if hard_delete {
        sqlx::query!(
            "DELETE FROM recordings WHERE user_id = ? AND external_media_id = ?",
            auth_user.user_id,
            local_id
        )
        .execute(&pool)
        .await
    } else {
        sqlx::query!(
            "UPDATE recordings SET is_delete = 1 WHERE user_id = ? AND external_media_id = ? AND is_delete = 0",
            auth_user.user_id,
            local_id
        )
        .execute(&pool)
        .await
    };
    delete_by_sql_result(result, hard_delete)
}

fn delete_by_sql_result(
    result: Result<sqlx::mysql::MySqlQueryResult, sqlx::Error>,
    hard_delete: bool,
) -> Json<DeleteRecorderResponse> {
    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                response(-3, "Recording not found")
            } else {
                response(
                    0,
                    if hard_delete {
                        "Hard deleted successfully"
                    } else {
                        "Deleted successfully"
                    },
                )
            }
        }
        Err(e) => {
            log::error!("Failed to delete recording: {}", e);
            response(-2, "Failed to delete recording")
        }
    }
}
