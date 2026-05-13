use sha2::{Digest, Sha256};
use sqlx::mysql::MySqlPool;

pub async fn check_api_token(
    pool: &MySqlPool,
    token: &str
) -> Option<i64> {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    let user_id = sqlx::query!("SELECT id FROM users WHERE api_token_hash = ?", token_hash)
        .fetch_optional(pool)
        .await
        .ok()?
        .map(|row| row.id as i64);
    user_id
}
