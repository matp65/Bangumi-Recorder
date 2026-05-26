use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use sqlx::QueryBuilder;
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
    const MAX_SYNC_RECORDS: usize = 10_000;
    if body.records.len() > MAX_SYNC_RECORDS {
        log::warn!(
            "User {} attempted to sync {} records (limit: {})",
            user_id, body.records.len(), MAX_SYNC_RECORDS
        );
        return Err(());
    }

    let mut client_bangumi_ids = HashSet::with_capacity(body.records.len());
    for rec in &body.records {
        client_bangumi_ids.insert(rec.bangumi_id.clone());
    }

    let mut external_to_easy: HashMap<String, u32> =
        HashMap::with_capacity(body.records.len());
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

    // Step 2: Collect all records to upsert (only those with resolved bangumi IDs)
    struct PendingWrite {
        easy_id: u32,
        recorder: Option<String>,
        status: i8,
        updated_at: NaiveDateTime,
    }

    let mut to_write: Vec<PendingWrite> = Vec::with_capacity(external_to_easy.len());
    for client_rec in &body.records {
        let easy_id = match external_to_easy.get(&client_rec.bangumi_id) {
            Some(&id) => id,
            None => continue,
        };
        let status = client_rec.user_status.map(|s| s as i8).unwrap_or(0);
        let client_ts =
            client_rec.updated_at.unwrap_or_else(|| chrono::Utc::now().naive_utc());
        to_write.push(PendingWrite {
            easy_id,
            recorder: client_rec.recorder.clone(),
            status,
            updated_at: client_ts,
        });
    }

    // Step 3: Single batch upsert (INSERT + UPDATE) with DB-level conflict resolution
    let mut tx = pool.begin().await.map_err(|e| {
        log::error!("tx begin error: {:?}", e);
    })?;

    if !to_write.is_empty() {
        let mut qb = QueryBuilder::new(
            "INSERT INTO recordings (user_id, bangumi_id, recorder, status, updated_at, created_at) ",
        );
        qb.push_values(to_write.iter(), |mut b, item| {
            b.push_bind(user_id)
             .push_bind(item.easy_id)
             .push_bind(&item.recorder)
             .push_bind(item.status)
             .push_bind(item.updated_at)
             .push_bind(item.updated_at);
        });
        qb.push(
            " ON DUPLICATE KEY UPDATE \
             recorder = IF(VALUES(updated_at) > updated_at, VALUES(recorder), recorder), \
             status = IF(VALUES(updated_at) > updated_at, VALUES(status), status), \
             updated_at = IF(VALUES(updated_at) > updated_at, VALUES(updated_at), updated_at)",
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
