use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDateTime;

use crate::auth_bearer::AuthUser;
use crate::api::new::AddRecordQuery;
use crate::api::update_recorder::UpdateRecorderQuery;
use crate::api::delete_recorder::DeleteRecorderQuery;
use crate::api::get_recorder::GetRecorderQuery;
use crate::api::list::{RecorderItem, list_recorder as v1_list_recorder};
use crate::api::detail_list::{DetailListItem, get_detail_list as v1_get_detail_list};
use crate::api::new::add_record as v1_add_record;
use crate::api::update_recorder::update_user_recorder as v1_update_recorder;
use crate::api::delete_recorder::delete_recorder as v1_delete_recorder;
use crate::api::get_recorder::get_recorder as v1_get_recorder;
use super::response::{success, success_empty, bad_request, not_found, conflict, internal_error, ApiResponse};

#[derive(Serialize)]
pub struct AddRecordData {
    pub local_bangumi_id: Option<u32>,
    pub other_id: Option<u32>,
    pub local_other_id: Option<u32>,
    pub bangumi_id: Option<u32>,
    pub recorder: Option<String>,
    pub date: Option<NaiveDateTime>,
}

#[derive(Serialize)]
pub struct GetRecordData {
    pub local_bangumi_id: Option<u32>,
    pub other_id: Option<u32>,
    pub local_other_id: Option<u32>,
    pub bangumi_id: Option<u32>,
    pub recorder: Option<String>,
    pub user_status: Option<i8>,
    pub is_delete: Option<bool>,
    pub date: Option<NaiveDateTime>,
}

pub async fn add_record(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<AddRecordQuery>,
) -> (StatusCode, Json<ApiResponse<AddRecordData>>) {
    let v1_resp = v1_add_record(State(pool.clone()), Extension(auth_user), Json(params)).await;
    let inner = v1_resp.0;
    match inner.status {
        0 => success(AddRecordData {
            local_bangumi_id: inner.local_bangumi_id,
            other_id: inner.other_id,
            local_other_id: inner.local_other_id,
            bangumi_id: inner.bangumi_id,
            recorder: inner.recorder,
            date: inner.date.map(|d| d.and_hms_opt(0, 0, 0).unwrap()),
        }),
        -1 => bad_request("Missing or invalid parameters"),
        -2 => not_found("Bangumi not found"),
        -3 => conflict("Record already exists"),
        _ => internal_error("Failed to add record"),
    }
}

pub async fn update_user_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<UpdateRecorderQuery>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let v1_resp = v1_update_recorder(State(pool.clone()), Extension(auth_user), Json(params)).await;
    let inner = v1_resp.0;
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

pub async fn delete_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<DeleteRecorderQuery>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let v1_resp = v1_delete_recorder(State(pool.clone()), Extension(auth_user), Json(params)).await;
    let inner = v1_resp.0;
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

pub async fn get_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(params): Json<GetRecorderQuery>,
) -> (StatusCode, Json<ApiResponse<GetRecordData>>) {
    let v1_resp = v1_get_recorder(State(pool.clone()), Extension(auth_user), Json(params)).await;
    let inner = v1_resp.0;
    if inner.status == 0 {
        success(GetRecordData {
            local_bangumi_id: inner.local_bangumi_id,
            other_id: inner.other_id,
            local_other_id: inner.local_other_id,
            bangumi_id: inner.bangumi_id,
            recorder: inner.recorder,
            user_status: inner.user_status,
            is_delete: inner.is_delete,
            date: inner.date.map(|d| d.and_hms_opt(0, 0, 0).unwrap()),
        })
    } else {
        not_found("Record not found")
    }
}

pub async fn list_recorder(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<RecorderItem>>>) {
    let v1_resp = v1_list_recorder(State(pool.clone()), Extension(auth_user)).await;
    let inner = v1_resp.0;
    if inner.status == 0 {
        success(inner.data.unwrap_or_default())
    } else {
        internal_error("Failed to list records")
    }
}

pub async fn get_detail_list(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<DetailListItem>>>) {
    let v1_resp = v1_get_detail_list(State(pool.clone()), Extension(auth_user)).await;
    let inner = v1_resp.0;
    if inner.status == 0 {
        success(inner.data.unwrap_or_default())
    } else {
        internal_error("Failed to get detail list")
    }
}

// --- RESTful path-param wrappers ---

#[derive(Deserialize)]
pub struct UpdateRecordBody {
    pub recorder: Option<String>,
    pub user_status: Option<i32>,
}

pub async fn get_record_by_bangumi(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<u32>,
) -> (StatusCode, Json<ApiResponse<GetRecordData>>) {
    get_recorder(
        State(pool),
        Extension(auth_user),
        Json(GetRecorderQuery {
            bangumi_id: Some(id),
            local_bangumi_id: None,
            other_id: None,
            local_other_id: None,
        }),
    ).await
}

pub async fn update_record_by_bangumi(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<u32>,
    Json(body): Json<UpdateRecordBody>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    update_user_recorder(
        State(pool),
        Extension(auth_user),
        Json(UpdateRecorderQuery {
            bangumi_id: Some(id as i32),
            recorder: body.recorder,
            user_status: body.user_status,
        }),
    ).await
}

pub async fn delete_record_by_bangumi(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<u32>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    delete_recorder(
        State(pool),
        Extension(auth_user),
        Json(DeleteRecorderQuery {
            bangumi_id: Some(id),
            other_id: None,
            local_other_id: None,
        }),
    ).await
}

pub async fn get_record_by_custom(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<u32>,
) -> (StatusCode, Json<ApiResponse<GetRecordData>>) {
    get_recorder(
        State(pool),
        Extension(auth_user),
        Json(GetRecorderQuery {
            bangumi_id: None,
            local_bangumi_id: None,
            other_id: Some(id),
            local_other_id: None,
        }),
    ).await
}

pub async fn delete_record_by_custom(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<u32>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    delete_recorder(
        State(pool),
        Extension(auth_user),
        Json(DeleteRecorderQuery {
            bangumi_id: None,
            other_id: Some(id),
            local_other_id: None,
        }),
    ).await
}
