use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

use crate::api::open::api_token::require_api_token;
use crate::api::open::new::add_record_open as v1_add_record_open;
use crate::api::open::update_recorder::update_user_recorder as v1_update_recorder;
use crate::api::open::delete_recorder::delete_recorder as v1_delete_recorder;
use crate::api::open::get_recorder::get_recorder as v1_get_recorder;
use crate::api::open::list::{list_recorder as v1_list_recorder, RecorderItem as OpenRecorderItem};
use crate::api::open::detail_list::{get_detail_list as v1_get_detail_list, DetailListItem as OpenDetailListItem};
use crate::api::open::new::AddRecordQuery;
use crate::api::open::update_recorder::UpdateRecorderQuery;
use crate::api::open::delete_recorder::DeleteRecorderQuery;
use crate::api::open::get_recorder::GetRecorderQuery;
use crate::api::open::list::ListRecorderQuery;
use crate::api::open::detail_list::DetailListQuery;
use crate::api::v2::record::{AddRecordData, GetRecordData};
use crate::api::v2::response::{success, success_empty, not_found, internal_error, unauthorized, bad_request, ApiResponse};

#[derive(Deserialize)]
pub struct OpenTokenQuery {
    pub token: Option<String>,
}

pub async fn add_record(
    State(pool): State<MySqlPool>,
    query: Query<AddRecordQuery>,
) -> (StatusCode, Json<ApiResponse<AddRecordData>>) {
    if query.token.is_none() {
        return unauthorized("Missing API token");
    }

    match v1_add_record_open(State(pool.clone()), query).await {
        Ok(json_resp) => {
            let inner = json_resp.0;
            match inner.status {
                0 => success(AddRecordData {
                    local_bangumi_id: inner.local_bangumi_id,
                    other_id: inner.other_id,
                    local_other_id: inner.local_other_id,
                    bangumi_id: inner.bangumi_id,
                    recorder: inner.recorder,
                    date: inner.date,
                }),
                -2 => not_found("Bangumi not found"),
                -3 => conflict("Record already exists"),
                _ => bad_request("Invalid parameters"),
            }
        }
        Err(StatusCode::UNAUTHORIZED) => unauthorized("Invalid API token"),
        Err(_) => internal_error("Internal server error"),
    }
}

fn conflict<T: Serialize>(msg: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::CONFLICT, Json(crate::api::v2::response::ApiResponse {
        status: -1,
        data: None,
        message: Some(msg.to_string()),
    }))
}

pub async fn list_recorder(
    State(pool): State<MySqlPool>,
    query: Query<ListRecorderQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<OpenRecorderItem>>>) {
    if query.token.is_none() {
        return unauthorized("Missing API token");
    }

    match v1_list_recorder(State(pool.clone()), query).await {
        Ok(json_resp) => {
            let inner = json_resp.0;
            if inner.status == 0 {
                success(inner.data.unwrap_or_default())
            } else {
                internal_error("Failed to list records")
            }
        }
        Err(StatusCode::UNAUTHORIZED) => unauthorized("Invalid API token"),
        Err(_) => internal_error("Internal server error"),
    }
}

pub async fn get_detail_list(
    State(pool): State<MySqlPool>,
    query: Query<DetailListQuery>,
) -> (StatusCode, Json<ApiResponse<Vec<OpenDetailListItem>>>) {
    if query.token.is_none() {
        return unauthorized("Missing API token");
    }

    match v1_get_detail_list(State(pool.clone()), query).await {
        Ok(json_resp) => {
            let inner = json_resp.0;
            if inner.status == 0 {
                success(inner.data.unwrap_or_default())
            } else {
                internal_error("Failed to get detail list")
            }
        }
        Err(StatusCode::UNAUTHORIZED) => unauthorized("Invalid API token"),
        Err(_) => internal_error("Internal server error"),
    }
}

// --- RESTful path-param wrappers ---

pub async fn get_record_by_bangumi(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Query(token_q): Query<OpenTokenQuery>,
) -> (StatusCode, Json<ApiResponse<GetRecordData>>) {
    let _user_id = match require_api_token(&pool, token_q.token.as_deref()).await {
        Ok(uid) => uid,
        Err(_) => return unauthorized("Invalid API token"),
    };

    match v1_get_recorder(
        State(pool.clone()),
        Query(GetRecorderQuery {
            bangumi_id: Some(id),
            local_bangumi_id: None,
            other_id: None,
            local_other_id: None,
            token: token_q.token,
        }),
    )
    .await
    {
        Ok(json_resp) => {
            let inner = json_resp.0;
            if inner.status == 0 {
                success(GetRecordData {
                    local_bangumi_id: inner.local_bangumi_id,
                    other_id: inner.other_id,
                    local_other_id: inner.local_other_id,
                    bangumi_id: inner.bangumi_id,
                    recorder: inner.recorder,
                    user_status: inner.user_status,
                    is_delete: inner.is_delete,
                    date: inner.date,
                })
            } else {
                not_found("Record not found")
            }
        }
        Err(StatusCode::UNAUTHORIZED) => unauthorized("Invalid API token"),
        Err(_) => internal_error("Internal server error"),
    }
}

#[derive(Deserialize)]
pub struct OpenUpdateBody {
    pub recorder: Option<String>,
    pub user_status: Option<i32>,
}

pub async fn update_record_by_bangumi(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Query(token_q): Query<OpenTokenQuery>,
    Json(body): Json<OpenUpdateBody>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let _user_id = match require_api_token(&pool, token_q.token.as_deref()).await {
        Ok(uid) => uid,
        Err(_) => return unauthorized("Invalid API token"),
    };

    match v1_update_recorder(
        State(pool.clone()),
        Query(UpdateRecorderQuery {
            bangumi_id: Some(id as i32),
            recorder: body.recorder,
            user_status: body.user_status,
            token: token_q.token,
        }),
    )
    .await
    {
        Ok(json_resp) => {
            let inner = json_resp.0;
            if inner.status == 0 {
                success_empty()
            } else {
                let msg = inner.message.as_deref().unwrap_or("Update failed");
                if inner.status == -1 {
                    bad_request(msg)
                } else if inner.status == -3 {
                    not_found(msg)
                } else {
                    internal_error(msg)
                }
            }
        }
        Err(StatusCode::UNAUTHORIZED) => unauthorized("Invalid API token"),
        Err(_) => internal_error("Internal server error"),
    }
}

pub async fn delete_record_by_bangumi(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Query(token_q): Query<OpenTokenQuery>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let _user_id = match require_api_token(&pool, token_q.token.as_deref()).await {
        Ok(uid) => uid,
        Err(_) => return unauthorized("Invalid API token"),
    };

    match v1_delete_recorder(
        State(pool.clone()),
        Query(DeleteRecorderQuery {
            bangumi_id: Some(id),
            other_id: None,
            local_other_id: None,
            token: token_q.token,
        }),
    )
    .await
    {
        Ok(json_resp) => {
            let inner = json_resp.0;
            if inner.status == 0 {
                success_empty()
            } else {
                let msg = inner.message.as_deref().unwrap_or("Delete failed");
                if inner.status == -1 {
                    bad_request(msg)
                } else if inner.status == -3 {
                    not_found(msg)
                } else {
                    internal_error(msg)
                }
            }
        }
        Err(StatusCode::UNAUTHORIZED) => unauthorized("Invalid API token"),
        Err(_) => internal_error("Internal server error"),
    }
}

pub async fn get_record_by_custom(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Query(token_q): Query<OpenTokenQuery>,
) -> (StatusCode, Json<ApiResponse<GetRecordData>>) {
    let _user_id = match require_api_token(&pool, token_q.token.as_deref()).await {
        Ok(uid) => uid,
        Err(_) => return unauthorized("Invalid API token"),
    };

    match v1_get_recorder(
        State(pool.clone()),
        Query(GetRecorderQuery {
            bangumi_id: None,
            local_bangumi_id: None,
            other_id: Some(id),
            local_other_id: None,
            token: token_q.token,
        }),
    )
    .await
    {
        Ok(json_resp) => {
            let inner = json_resp.0;
            if inner.status == 0 {
                success(GetRecordData {
                    local_bangumi_id: inner.local_bangumi_id,
                    other_id: inner.other_id,
                    local_other_id: inner.local_other_id,
                    bangumi_id: inner.bangumi_id,
                    recorder: inner.recorder,
                    user_status: inner.user_status,
                    is_delete: inner.is_delete,
                    date: inner.date,
                })
            } else {
                not_found("Record not found")
            }
        }
        Err(StatusCode::UNAUTHORIZED) => unauthorized("Invalid API token"),
        Err(_) => internal_error("Internal server error"),
    }
}

pub async fn delete_record_by_custom(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Query(token_q): Query<OpenTokenQuery>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let _user_id = match require_api_token(&pool, token_q.token.as_deref()).await {
        Ok(uid) => uid,
        Err(_) => return unauthorized("Invalid API token"),
    };

    match v1_delete_recorder(
        State(pool.clone()),
        Query(DeleteRecorderQuery {
            bangumi_id: None,
            other_id: Some(id),
            local_other_id: None,
            token: token_q.token,
        }),
    )
    .await
    {
        Ok(json_resp) => {
            let inner = json_resp.0;
            if inner.status == 0 {
                success_empty()
            } else {
                let msg = inner.message.as_deref().unwrap_or("Delete failed");
                if inner.status == -1 {
                    bad_request(msg)
                } else if inner.status == -3 {
                    not_found(msg)
                } else {
                    internal_error(msg)
                }
            }
        }
        Err(StatusCode::UNAUTHORIZED) => unauthorized("Invalid API token"),
        Err(_) => internal_error("Internal server error"),
    }
}
