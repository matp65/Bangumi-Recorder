use std::collections::{HashMap, HashSet};

use axum::{
    Json,
    extract::{Extension, Query, State},
    http::StatusCode,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::QueryBuilder;
use sqlx::mysql::MySqlPool;

use super::response::{ApiResponse, bad_request, internal_error, success};
use crate::api::logs::{LogTarget, write_recording_log};
use crate::api::new::{AddRecordQuery, add_record};
use crate::auth_bearer::AuthUser;

#[derive(Debug, PartialEq, Eq)]
enum PendingSyncAction {
    Upsert {
        easy_id: u32,
        recorder: Option<String>,
        status: i8,
        updated_at: NaiveDateTime,
    },
    CreateById {
        bangumi_id: u32,
        recorder: Option<String>,
        status: i8,
        updated_at: NaiveDateTime,
    },
}

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

struct ExistingRecordingState {
    id: u32,
    recorder: Option<String>,
    status: i8,
    updated_at: NaiveDateTime,
}

fn build_pending_sync_actions(
    records: &[SyncRequestRecord],
    external_to_easy: &HashMap<String, u32>,
) -> Vec<PendingSyncAction> {
    let mut actions = Vec::with_capacity(records.len());
    for client_rec in records {
        if let Some(&easy_id) = external_to_easy.get(&client_rec.bangumi_id) {
            let status = client_rec.user_status.map(|s| s as i8).unwrap_or(0);
            let client_ts = client_rec
                .updated_at
                .unwrap_or_else(|| chrono::Utc::now().naive_utc());
            actions.push(PendingSyncAction::Upsert {
                easy_id,
                recorder: client_rec.recorder.clone(),
                status,
                updated_at: client_ts,
            });
            continue;
        }

        if let Ok(bangumi_id) = client_rec.bangumi_id.parse::<u32>() {
            let status = client_rec.user_status.map(|s| s as i8).unwrap_or(0);
            let client_ts = client_rec
                .updated_at
                .unwrap_or_else(|| chrono::Utc::now().naive_utc());
            actions.push(PendingSyncAction::CreateById {
                bangumi_id,
                recorder: client_rec.recorder.clone(),
                status,
                updated_at: client_ts,
            });
        }
    }
    actions
}

async fn do_sync(
    pool: &MySqlPool,
    user_id: i64,
    body: SyncRequestBody,
) -> Result<SyncResponseData, ()> {
    const MAX_SYNC_RECORDS: usize = 10_000;
    if body.records.len() > MAX_SYNC_RECORDS {
        log::warn!(
            "User {} attempted to sync {} records (limit: {})",
            user_id,
            body.records.len(),
            MAX_SYNC_RECORDS
        );
        return Err(());
    }

    let mut client_bangumi_ids = HashSet::with_capacity(body.records.len());
    for rec in &body.records {
        client_bangumi_ids.insert(rec.bangumi_id.clone());
    }

    let mut external_to_easy: HashMap<String, u32> = HashMap::with_capacity(body.records.len());
    if !body.records.is_empty() {
        let mut qb = QueryBuilder::new(
            "SELECT id, external_id FROM bangumi_info_easy WHERE external_id IN (",
        );
        let mut sep = qb.separated(", ");
        for rec in &body.records {
            sep.push_bind(rec.bangumi_id.as_str());
        }
        sep.push_unseparated(")");
        let rows: Vec<(u32, String)> = qb
            .build_query_as()
            .fetch_all(pool)
            .await
            .map_err(|e| log::error!("batch resolve bangumi error: {:?}", e))?;
        for (id, external_id) in rows {
            external_to_easy.insert(external_id, id);
        }
    }

    // Step 2: Build pending actions for each record, including auto-creation for unresolved IDs
    let pending_actions = build_pending_sync_actions(&body.records, &external_to_easy);
    let mut to_write: Vec<(u32, Option<String>, i8, NaiveDateTime)> =
        Vec::with_capacity(pending_actions.len());
    for action in &pending_actions {
        match action {
            PendingSyncAction::Upsert {
                easy_id,
                recorder,
                status,
                updated_at,
            } => to_write.push((*easy_id, recorder.clone(), *status, *updated_at)),
            PendingSyncAction::CreateById {
                bangumi_id,
                recorder,
                status,
                updated_at,
            } => {
                let response = add_record(
                    State(pool.clone()),
                    Extension(AuthUser { user_id }),
                    Json(AddRecordQuery {
                        bangumi_id: Some(*bangumi_id),
                        source: Some("bangumi".to_string()),
                        external_id: None,
                        imdb_id: None,
                        use_api: None,
                        other_id: None,
                        other_title: None,
                        other_description: None,
                        other_cover: None,
                        other_max_number: None,
                        other_status: None,
                        user_status: Some(*status as i32),
                        recorder: recorder.clone(),
                    }),
                )
                .await
                .0;

                if let Some(easy_id) = response.local_bangumi_id {
                    let client_ts = *updated_at;
                    to_write.push((easy_id, recorder.clone(), *status, client_ts));
                }
            }
        }
    }

    // Step 3: Single batch upsert (INSERT + UPDATE) with DB-level conflict resolution
    let mut tx = pool.begin().await.map_err(|e| {
        log::error!("tx begin error: {:?}", e);
    })?;

    let mut old_states: HashMap<u32, ExistingRecordingState> = HashMap::new();
    if !to_write.is_empty() {
        let mut qb = QueryBuilder::new(
            "SELECT id, bangumi_id, recorder, status, updated_at FROM recordings WHERE user_id = ",
        );
        qb.push_bind(user_id).push(" AND bangumi_id IN (");
        let mut sep = qb.separated(", ");
        for (easy_id, _, _, _) in &to_write {
            sep.push_bind(*easy_id);
        }
        sep.push_unseparated(")");

        let rows: Vec<(u32, u32, Option<String>, i8, NaiveDateTime)> = qb
            .build_query_as()
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| log::error!("existing recording state query error: {:?}", e))?;
        for (id, easy_id, recorder, status, updated_at) in rows {
            old_states.insert(
                easy_id,
                ExistingRecordingState {
                    id,
                    recorder,
                    status,
                    updated_at,
                },
            );
        }
    }

    if !to_write.is_empty() {
        let mut qb = QueryBuilder::new(
            "INSERT INTO recordings (user_id, bangumi_id, recorder, status, updated_at, created_at) ",
        );
        qb.push_values(to_write.iter(), |mut b, item| {
            let (easy_id, recorder, status, updated_at) = item;
            b.push_bind(user_id)
                .push_bind(*easy_id)
                .push_bind(recorder)
                .push_bind(*status)
                .push_bind(*updated_at)
                .push_bind(*updated_at);
        });
        qb.push(
            " ON DUPLICATE KEY UPDATE \
             recorder = IF(VALUES(updated_at) > updated_at AND NOT (recorder <=> VALUES(recorder) AND status = VALUES(status)), VALUES(recorder), recorder), \
             status = IF(VALUES(updated_at) > updated_at AND NOT (recorder <=> VALUES(recorder) AND status = VALUES(status)), VALUES(status), status), \
             updated_at = IF(VALUES(updated_at) > updated_at AND NOT (recorder <=> VALUES(recorder) AND status = VALUES(status)), VALUES(updated_at), updated_at)",
        );
        qb.build().execute(&mut *tx).await.map_err(|e| {
            log::error!("batch upsert error: {:?}", e);
        })?;
    }

    // Step 4: Query authoritative server state (post-write)
    let server_rows = sqlx::query!(
        r#"
        SELECT b.external_id AS bangumi_id, r.recorder, r.status, r.updated_at, r.is_delete
        FROM recordings r
        JOIN bangumi_info_easy b ON r.bangumi_id = b.id
        WHERE r.user_id = ? AND r.bangumi_id IS NOT NULL
        "#,
        user_id
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| log::error!("server state query error: {:?}", e))?;

    tx.commit().await.map_err(|e| {
        log::error!("tx commit error: {:?}", e);
    })?;

    for (easy_id, recorder, status, updated_at) in &to_write {
        let Some(old) = old_states.get(easy_id) else {
            continue;
        };
        if *updated_at <= old.updated_at {
            continue;
        }
        if old.recorder != *recorder {
            write_recording_log(
                pool,
                old.id,
                Some(user_id),
                LogTarget::Bangumi(*easy_id),
                "recorder_changed",
                Some("recorder"),
                old.recorder.as_ref().map(|v| json!(v)),
                recorder.as_ref().map(|v| json!(v)),
                Some(json!({ "source": "sync" })),
            )
            .await;
        }
        if old.status != *status {
            write_recording_log(
                pool,
                old.id,
                Some(user_id),
                LogTarget::Bangumi(*easy_id),
                "status_changed",
                Some("status"),
                Some(json!(old.status)),
                Some(json!(status)),
                Some(json!({ "source": "sync" })),
            )
            .await;
        }
    }

    let mut records: Vec<SyncResponseRecord> = Vec::with_capacity(server_rows.len());
    let mut deleted: Vec<String> = Vec::new();

    for r in server_rows {
        let bangumi_id = r.bangumi_id;
        if client_bangumi_ids.contains(&bangumi_id) {
            records.push(SyncResponseRecord {
                bangumi_id,
                recorder: r.recorder,
                user_status: Some(r.status),
                updated_at: r.updated_at,
            });
        } else if r.is_delete != 0 {
            deleted.push(bangumi_id);
        } else {
            records.push(SyncResponseRecord {
                bangumi_id,
                recorder: r.recorder,
                user_status: Some(r.status),
                updated_at: r.updated_at,
            });
        }
    }

    Ok(SyncResponseData { records, deleted })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_pending_sync_actions_creates_missing_numeric_ids() {
        let records = vec![
            SyncRequestRecord {
                bangumi_id: "1001".to_string(),
                recorder: Some("tv".to_string()),
                user_status: Some(2),
                updated_at: None,
            },
            SyncRequestRecord {
                bangumi_id: "2002".to_string(),
                recorder: None,
                user_status: None,
                updated_at: None,
            },
        ];
        let mut external_to_easy = HashMap::new();
        external_to_easy.insert("2002".to_string(), 42);

        let actions = build_pending_sync_actions(&records, &external_to_easy);

        assert_eq!(actions.len(), 2);
        assert!(matches!(
            actions[0],
            PendingSyncAction::CreateById {
                bangumi_id: 1001,
                ..
            }
        ));
        assert!(matches!(
            actions[1],
            PendingSyncAction::Upsert { easy_id: 42, .. }
        ));
    }
}
