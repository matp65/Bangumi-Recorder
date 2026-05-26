use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDateTime;

use crate::auth_bearer::AuthUser;
use super::response::{success, bad_request, internal_error, ApiResponse};

#[derive(Deserialize)]
pub struct SyncRequestRecord {
    pub bangumi_id: String,
    pub recorder: Option<String>,
    pub user_status: Option<i32>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct SyncRequestBody {
    pub records: Vec<SyncRequestRecord>,
}

#[derive(Serialize, Clone)]
pub struct SyncResponseRecord {
    pub bangumi_id: String,
    pub recorder: Option<String>,
    pub user_status: Option<i8>,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct SyncResponseData {
    pub records: Vec<SyncResponseRecord>,
    pub deleted: Vec<String>,
}

#[derive(Deserialize)]
pub struct SyncSinceQuery {
    pub since: Option<NaiveDateTime>,
}

async fn do_sync(
    pool: &MySqlPool,
    user_id: i64,
    body: SyncRequestBody,
) -> Result<SyncResponseData, ()> {
    let mut server_records: Vec<SyncResponseRecord> = Vec::new();
    let mut client_bangumi_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for client_rec in &body.records {
        client_bangumi_ids.insert(client_rec.bangumi_id.clone());

        let easy_id = match sqlx::query_scalar!(
            "SELECT id FROM bangumi_info_easy WHERE external_id = ?",
            client_rec.bangumi_id
        )
        .fetch_optional(pool)
        .await
        {
            Ok(Some(id)) => id,
            Ok(None) => continue,
            Err(_) => continue,
        };

        let existing = sqlx::query!(
            r#"
            SELECT r.id, r.recorder, r.status, r.updated_at
            FROM recordings r
            WHERE r.user_id = ? AND r.bangumi_id = ?
            "#,
            user_id,
            easy_id
        )
        .fetch_optional(pool)
        .await;

        let existing = match existing {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(server_rec) = existing {
            let server_ts = server_rec.updated_at;
            let client_ts = client_rec.updated_at.unwrap_or(server_ts);

            if client_ts > server_ts {
                let _ = sqlx::query!(
                    "UPDATE recordings SET recorder = ?, status = ?, updated_at = ? WHERE id = ?",
                    client_rec.recorder,
                    client_rec.user_status.map(|s| s as i8),
                    client_ts,
                    server_rec.id
                )
                .execute(pool)
                .await;
            }

            let final_rec = sqlx::query!(
                r#"
                SELECT r.recorder, r.status, r.updated_at
                FROM recordings r
                WHERE r.id = ?
                "#,
                server_rec.id
            )
            .fetch_optional(pool)
            .await;

            if let Ok(Some(r)) = final_rec {
                server_records.push(SyncResponseRecord {
                    bangumi_id: client_rec.bangumi_id.clone(),
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    updated_at: r.updated_at,
                });
            }
        } else {
            let user_status = client_rec.user_status.map(|s| s as i8).unwrap_or(0);
            let now = client_rec.updated_at.unwrap_or_else(|| chrono::Utc::now().naive_utc());

            let _ = sqlx::query!(
                r#"
                INSERT INTO recordings (user_id, bangumi_id, recorder, status, updated_at, created_at)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
                user_id,
                easy_id,
                client_rec.recorder,
                user_status,
                now,
                now
            )
            .execute(pool)
            .await;

            server_records.push(SyncResponseRecord {
                bangumi_id: client_rec.bangumi_id.clone(),
                recorder: client_rec.recorder.clone(),
                user_status: Some(user_status),
                updated_at: now,
            });
        }
    }

    let mut deleted: Vec<String> = Vec::new();

    let server_all = sqlx::query!(
        r#"
        SELECT b.external_id AS bangumi_id, r.is_delete
        FROM recordings r
        JOIN bangumi_info_easy b ON r.bangumi_id = b.id
        WHERE r.user_id = ? AND r.bangumi_id IS NOT NULL
        "#,
        user_id
    )
    .fetch_all(pool)
    .await;

    if let Ok(rows) = server_all {
        for r in rows {
            let bid = r.bangumi_id;
            if !client_bangumi_ids.contains(&bid) {
                if r.is_delete != 0 {
                    deleted.push(bid.clone());
                } else {
                    let rec = sqlx::query!(
                        r#"
                        SELECT r.recorder, r.status, r.updated_at
                        FROM recordings r
                        JOIN bangumi_info_easy b ON r.bangumi_id = b.id
                        WHERE b.external_id = ? AND r.user_id = ?
                        "#,
                        bid,
                        user_id
                    )
                    .fetch_optional(pool)
                    .await;

                    if let Ok(Some(rr)) = rec {
                        server_records.push(SyncResponseRecord {
                            bangumi_id: bid.clone(),
                            recorder: rr.recorder,
                            user_status: Some(rr.status),
                            updated_at: rr.updated_at,
                        });
                    }
                }
            }
        }
    }

    Ok(SyncResponseData {
        records: server_records,
        deleted,
    })
}

async fn do_incremental_sync(
    pool: &MySqlPool,
    user_id: i64,
    since: NaiveDateTime,
) -> Result<Vec<SyncResponseRecord>, ()> {
    let rows = sqlx::query!(
        r#"
        SELECT b.external_id AS bangumi_id, r.recorder, r.status, r.updated_at
        FROM recordings r
        JOIN bangumi_info_easy b ON r.bangumi_id = b.id
        WHERE r.user_id = ? AND r.bangumi_id IS NOT NULL AND r.updated_at > ?
        "#,
        user_id,
        since
    )
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let records: Vec<SyncResponseRecord> = rows
                .into_iter()
                .map(|r| SyncResponseRecord {
                    bangumi_id: r.bangumi_id,
                    recorder: r.recorder,
                    user_status: Some(r.status),
                    updated_at: r.updated_at,
                })
                .collect();
            Ok(records)
        }
        Err(e) => {
            log::error!("DB error: {:?}", e);
            Err(())
        }
    }
}

pub async fn sync_records(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<SyncRequestBody>,
) -> (StatusCode, Json<ApiResponse<SyncResponseData>>) {
    match do_sync(&pool, auth_user.user_id, body).await {
        Ok(data) => success(data),
        Err(_) => internal_error("Sync failed"),
    }
}

pub async fn incremental_sync(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<SyncSinceQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<SyncResponseRecord>>>) {
    let since = match query.since {
        Some(s) => s,
        None => return bad_request("Missing 'since' query parameter"),
    };

    match do_incremental_sync(&pool, auth_user.user_id, since).await {
        Ok(records) => success(records),
        Err(_) => internal_error("Database error"),
    }
}

pub async fn do_sync_records(
    pool: &MySqlPool,
    user_id: i64,
    body: SyncRequestBody,
) -> (StatusCode, Json<ApiResponse<SyncResponseData>>) {
    match do_sync(pool, user_id, body).await {
        Ok(data) => success(data),
        Err(_) => internal_error("Sync failed"),
    }
}

pub async fn do_incremental_sync_records(
    pool: &MySqlPool,
    user_id: i64,
    since: NaiveDateTime,
) -> (StatusCode, Json<ApiResponse<Vec<SyncResponseRecord>>>) {
    match do_incremental_sync(pool, user_id, since).await {
        Ok(records) => success(records),
        Err(_) => internal_error("Database error"),
    }
}
