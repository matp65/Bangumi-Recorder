use sqlx::mysql::MySqlPool;

pub async fn check_api_token(
    pool: &MySqlPool,
    token: &str
) -> Option<i64> {
    let user_id = sqlx::query!("SELECT id FROM users WHERE api_token_hash = ?", token)
        .fetch_optional(pool)
        .await
        .ok()?
        .map(|row| row.id as i64);
    user_id
}
