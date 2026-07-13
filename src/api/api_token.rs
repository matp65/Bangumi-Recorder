use axum::http::{HeaderMap, StatusCode, header::AUTHORIZATION};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::mysql::MySqlPool;

// Permission bit definitions
pub const PERM_READ: u64 = 1 << 0;
pub const PERM_WRITE: u64 = 1 << 1;
pub const PERM_VIEW_INFO: u64 = 1 << 2;
pub const PERM_MODIFY_INFO: u64 = 1 << 3;
pub const PERM_ADD_RECORD: u64 = 1 << 4;
pub const PERM_DELETE_RECORD: u64 = 1 << 5;
pub const PERM_MODIFY_RECORD: u64 = 1 << 6;
pub const PERM_CHANGE_STATUS: u64 = 1 << 7;
pub const PERM_READ_LOGS: u64 = 1 << 8;
pub const PERM_ALL: u64 = u64::MAX;
const LEGACY_ALL_COMBINED: u64 = PERM_READ
    | PERM_WRITE
    | PERM_VIEW_INFO
    | PERM_MODIFY_INFO
    | PERM_ADD_RECORD
    | PERM_DELETE_RECORD
    | PERM_MODIFY_RECORD
    | PERM_CHANGE_STATUS;

// Combined value of all individual permissions (without PERM_ALL).
// Used by the frontend to compute "Allow All" without JS 32-bit overflow.
pub const ALL_COMBINED: u64 = PERM_READ
    | PERM_WRITE
    | PERM_VIEW_INFO
    | PERM_MODIFY_INFO
    | PERM_ADD_RECORD
    | PERM_DELETE_RECORD
    | PERM_MODIFY_RECORD
    | PERM_CHANGE_STATUS
    | PERM_READ_LOGS;

pub static PERM_LABELS: &[(&str, u64, &str)] = &[
    ("Read-only", PERM_READ, "View record list and details"),
    (
        "Read-Write",
        PERM_WRITE,
        "Add, modify, delete records and change status",
    ),
    (
        "View Personal Info",
        PERM_VIEW_INFO,
        "View nickname, avatar, etc.",
    ),
    (
        "Modify Personal Info",
        PERM_MODIFY_INFO,
        "Modify nickname, avatar, etc.",
    ),
    ("Add Record", PERM_ADD_RECORD, "Add new tracking records"),
    (
        "Delete Record",
        PERM_DELETE_RECORD,
        "Delete tracking records",
    ),
    (
        "Modify Record",
        PERM_MODIFY_RECORD,
        "Modify tracking progress",
    ),
    (
        "Change Status",
        PERM_CHANGE_STATUS,
        "Change tracking status",
    ),
    ("Read Logs", PERM_READ_LOGS, "View user operation logs"),
];

pub fn has_perm(perms: u64, required: &[u64]) -> bool {
    if perms & PERM_ALL == PERM_ALL {
        return true;
    }
    if perms & ALL_COMBINED == ALL_COMBINED {
        return true;
    }
    if perms & LEGACY_ALL_COMBINED == LEGACY_ALL_COMBINED {
        return true;
    }
    required.iter().any(|&f| perms & f != 0)
}

pub fn has_all_perms(perms: u64, required: &[u64]) -> bool {
    if perms & PERM_ALL == PERM_ALL
        || perms & ALL_COMBINED == ALL_COMBINED
        || perms & LEGACY_ALL_COMBINED == LEGACY_ALL_COMBINED
    {
        return true;
    }
    required.iter().all(|&permission| perms & permission != 0)
}

pub fn api_token_from_request<'a>(
    headers: &'a HeaderMap,
    query_token: Option<&'a str>,
) -> Option<&'a str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            let (scheme, token) = value.split_once(' ')?;
            scheme
                .eq_ignore_ascii_case("Bearer")
                .then_some(token.trim())
        })
        .filter(|value| !value.is_empty())
        .or(query_token)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenInfo {
    pub user_id: i64,
    pub permissions: u64,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct ApiTokenRow {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub token_hash: String,
    pub permissions: u64,
    pub is_active: bool,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn check_api_token(pool: &MySqlPool, token: &str) -> Option<TokenInfo> {
    let token_hash = hash_token(token);

    let row = sqlx::query(
        "SELECT user_id, permissions FROM api_tokens WHERE token_hash = ? AND is_active = 1",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return None,
        Err(e) => {
            log::error!("Failed to check API token: {:?}", e);
            return None;
        }
    };

    let user_id: u32 = match row.try_get("user_id") {
        Ok(v) => v,
        Err(_) => return None,
    };
    let permissions: u64 = match row.try_get("permissions") {
        Ok(v) => v,
        Err(_) => return None,
    };

    // Update last_used_at
    let _ = sqlx::query("UPDATE api_tokens SET last_used_at = NOW() WHERE token_hash = ?")
        .bind(&token_hash)
        .execute(pool)
        .await;

    Some(TokenInfo {
        user_id: user_id as i64,
        permissions,
    })
}

pub async fn require_api_token(
    pool: &MySqlPool,
    token: Option<&str>,
) -> Result<TokenInfo, StatusCode> {
    let token = token.ok_or(StatusCode::UNAUTHORIZED)?;
    check_api_token(pool, token)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)
}

pub async fn require_token_with_perm(
    pool: &MySqlPool,
    token: Option<&str>,
    required: &[u64],
) -> Result<TokenInfo, StatusCode> {
    let info = require_api_token(pool, token).await?;
    if !has_perm(info.permissions, required) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(info)
}

pub async fn require_token_with_all_perms(
    pool: &MySqlPool,
    token: Option<&str>,
    required: &[u64],
) -> Result<TokenInfo, StatusCode> {
    let info = require_api_token(pool, token).await?;
    if !has_all_perms(info.permissions, required) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_perm_accepts_specific_required_permission() {
        assert!(has_perm(PERM_READ_LOGS, &[PERM_READ_LOGS]));
        assert!(!has_perm(PERM_READ, &[PERM_READ_LOGS]));
    }

    #[test]
    fn has_all_perms_requires_every_requested_permission() {
        assert!(has_all_perms(
            PERM_READ | PERM_WRITE,
            &[PERM_READ, PERM_WRITE]
        ));
        assert!(!has_all_perms(PERM_READ, &[PERM_READ, PERM_WRITE]));
    }

    #[test]
    fn has_perm_accepts_current_all_combined() {
        assert!(has_perm(ALL_COMBINED, &[PERM_READ_LOGS]));
    }

    #[test]
    fn has_perm_accepts_legacy_all_combined_for_compatibility() {
        let legacy_all = PERM_READ
            | PERM_WRITE
            | PERM_VIEW_INFO
            | PERM_MODIFY_INFO
            | PERM_ADD_RECORD
            | PERM_DELETE_RECORD
            | PERM_MODIFY_RECORD
            | PERM_CHANGE_STATUS;

        assert!(has_perm(legacy_all, &[PERM_READ_LOGS]));
    }

    #[test]
    fn hash_token_is_stable_sha256_hex() {
        assert_eq!(
            hash_token("token"),
            "3c469e9d6c5875d37a43f353d4f88e61fcf812c66eee3457465a40b0da4153e0"
        );
    }

    #[test]
    fn authorization_header_is_preferred_over_legacy_query_token() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, "Bearer header-token".parse().unwrap());
        assert_eq!(
            api_token_from_request(&headers, Some("query-token")),
            Some("header-token")
        );
    }

    #[test]
    fn authorization_scheme_is_case_insensitive() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, "bearer header-token".parse().unwrap());
        assert_eq!(api_token_from_request(&headers, None), Some("header-token"));
    }

    #[test]
    fn legacy_query_token_remains_supported() {
        assert_eq!(
            api_token_from_request(&HeaderMap::new(), Some("query-token")),
            Some("query-token")
        );
    }

    #[test]
    fn invalid_authorization_header_falls_back_to_legacy_query_token() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, "Basic credentials".parse().unwrap());
        assert_eq!(
            api_token_from_request(&headers, Some("query-token")),
            Some("query-token")
        );
    }
}
