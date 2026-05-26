use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::auth_bearer::{
    LoginRequest, RegisterRequest,
    verify_password, hash_password, hash_api_token,
};
use super::response::{success, conflict, internal_error, unauthorized, forbidden, ApiResponse};

#[derive(Serialize)]
pub struct LoginData {
    pub token: String,
}

#[derive(Serialize)]
pub struct RegisterData {
    pub token: String,
    pub api_token: String,
}

#[derive(Serialize)]
pub struct ConfigData {
    pub allow_register: bool,
    pub register_need_token: bool,
}

fn build_claims(user_id: i64, username: String, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        user_id: i64,
        sub: String,
        exp: usize,
        iat: usize,
    }

    let now = Utc::now();
    let claims = Claims {
        user_id,
        sub: username,
        iat: now.timestamp() as usize,
        exp: (now + Duration::days(7)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub async fn login(
    State(pool): State<MySqlPool>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<ApiResponse<LoginData>>) {
    let (username, password) = match (payload.username, payload.password) {
        (Some(u), Some(p)) => (u, p),
        _ => {
            return unauthorized("Please enter username or password");
        }
    };

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let row = match sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = ?",
        username
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return internal_error("Database error");
        }
    };

    let row = match row {
        Some(row) => row,
        None => {
            return unauthorized("Invalid username or password");
        }
    };

    if !verify_password(&password, &row.password_hash) {
        return unauthorized("Invalid username or password");
    }

    let user_id = row.id as i64;
    let token = match build_claims(user_id, username, &jwt_secret) {
        Ok(token) => token,
        Err(_) => {
            return internal_error("Failed to generate token");
        }
    };

    success(LoginData { token })
}

pub async fn register(
    State(pool): State<MySqlPool>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<RegisterData>>) {
    let allow_register = std::env::var("ALLOW_REGISTER")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase();

    if allow_register == "false" {
        return forbidden("Registration is disabled");
    }

    let register_need_token = std::env::var("REGISTER_NEED_TOKEN")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

    if register_need_token {
        let token = std::env::var("REGISTER_TOKEN").unwrap_or_default();
        if token.is_empty() {
            return internal_error("Registration token is not configured");
        }
    }

    let config_register_token = std::env::var("REGISTER_TOKEN").unwrap_or_default();

    if register_need_token && payload.register_token.as_deref() != Some(config_register_token.as_str()) {
        return unauthorized("Invalid registration token");
    }

    let (username, password) = match (payload.username, payload.password) {
        (Some(u), Some(p)) => (u, p),
        _ => {
            return unauthorized("Please enter username or password");
        }
    };

    match sqlx::query!("SELECT id FROM users WHERE username = ?", username)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(_)) => {
            return conflict("Username already exists");
        }
        Ok(None) => {}
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Database error");
        }
    };

    let password_hash = match hash_password(password) {
        Ok(hash) => hash,
        Err(_) => {
            return internal_error("Failed to hash password");
        }
    };

    let user_uuid = uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string();

    let raw_api_token = uuid::Uuid::new_v4().to_string();
    let api_token_hash = hash_api_token(&raw_api_token);

    let insert_result = sqlx::query!(
        "INSERT INTO users (username, password_hash, api_token_hash, uuid) VALUES (?, ?, ?, ?)",
        username,
        password_hash,
        api_token_hash,
        user_uuid
    )
    .execute(&pool)
    .await;

    let user_id = match insert_result {
        Ok(res) => res.last_insert_id() as i64,
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return internal_error("Failed to register user");
        }
    };

    // Create default API token with all permissions
    let _ = sqlx::query(
        "INSERT INTO api_tokens (user_id, name, token_hash, permissions) VALUES (?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind("Default Token")
    .bind(&api_token_hash)
    .bind(u64::MAX as i64)
    .execute(&pool)
    .await;

    let token = match build_claims(user_id, username, &std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")) {
        Ok(token) => token,
        Err(_) => {
            return internal_error("Failed to generate token");
        }
    };

    success(RegisterData {
        token,
        api_token: raw_api_token,
    })
}

pub async fn get_config() -> (StatusCode, Json<ApiResponse<ConfigData>>) {
    let allow_register = std::env::var("ALLOW_REGISTER")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase();

    let register_need_token = std::env::var("REGISTER_NEED_TOKEN")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase();

    success(ConfigData {
        allow_register: allow_register != "false",
        register_need_token: register_need_token != "false",
    })
}
