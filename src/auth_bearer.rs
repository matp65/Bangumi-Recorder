use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use axum::{
    body::Body,
    extract::{Extension, Json, State},
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use bcrypt::verify as bcrypt_verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub status: i32,
    pub token: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub register_token: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub status: i32,
    pub token: Option<String>,
    pub api_token: Option<String>,
    pub message: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Claims {
    user_id: i64,
    sub: String,
    exp: usize,
    iat: usize,
}

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: i64,
}

fn build_claims(user_id: i64, username: String, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
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

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    if password_hash.starts_with("$2a$")
        || password_hash.starts_with("$2b$")
        || password_hash.starts_with("$2y$")
    {
        bcrypt_verify(password, password_hash).unwrap_or(false)
    } else if password_hash.starts_with("$argon2") {
        let parsed = PasswordHash::new(password_hash);
        if let Ok(parsed) = parsed {
            Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
        } else {
            false
        }
    } else {
        false
    }
}

pub async fn login(
    State(pool): State<MySqlPool>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {

    let (username, password) = match (payload.username, payload.password) {
        (Some(u), Some(p)) => (u, p),
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                AxumJson(LoginResponse {
                    status: -1,
                    token: None,
                    message: Some("Please enter username or password".to_string()),
                }),
            );
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
            return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(LoginResponse {
                status: -1,
                token: None,
                message: Some("Database error".to_string()),
            }));
        }
    };

    let row = match row {
        Some(row) => row,
        None => {
            return (StatusCode::UNAUTHORIZED, AxumJson(LoginResponse {
                status: -2,
                token: None,
                message: Some("Invalid username or password".to_string()),
            }));
        }
    };

    if !verify_password(&password, &row.password_hash) {
        return (StatusCode::UNAUTHORIZED, AxumJson(LoginResponse {
            status: -2,
            token: None,
            message: Some("Invalid username or password".to_string()),
        }));
    }

    let user_id = row.id as i64;
    let token = match build_claims(user_id, username, &jwt_secret) {
        Ok(token) => token,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(LoginResponse {
                status: -3,
                token: None,
                message: Some("Failed to generate token".to_string()),
            }));
        }
    };

    (
        StatusCode::OK,
        AxumJson(LoginResponse {
            status: 0,
            token: Some(token),
            message: None,
        }),
    )
}

pub fn verify_jwt(token: &str, secret: &str) -> jsonwebtoken::errors::Result<Claims> {
    let validation = Validation::default();
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation).map(|data| data.claims)
}

pub async fn jwt_auth(
    req: Request<Body>,
    next: Next,
    jwt_secret: String,
) -> Response {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer ").map(str::trim));

    let token = match token {
        Some(token) if !token.is_empty() => token,
        _ => {
            let body = AxumJson(LoginResponse {
                status: -4,
                token: None,
                message: Some("Missing or invalid Authorization header".to_string()),
            });
            return (StatusCode::UNAUTHORIZED, body).into_response();
        }
    };

    let claims = match verify_jwt(token, &jwt_secret) {
        Ok(claims) => claims,
        Err(_) => {
            let body = AxumJson(LoginResponse {
                status: -5,
                token: None,
                message: Some("Invalid or expired token".to_string()),
            });
            return (StatusCode::UNAUTHORIZED, body).into_response();
        }
    };

    let auth_user = AuthUser {
        user_id: claims.user_id
    };

    let mut req = req;
    req.extensions_mut().insert(auth_user);

    next.run(req).await
}

pub async fn register(
    State(pool): State<MySqlPool>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Check if registration is allowed
    let allow_register = std::env::var("ALLOW_REGISTER")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase();
    
    if allow_register == "false" {
        return (
            StatusCode::FORBIDDEN,
            AxumJson(RegisterResponse {
                status: -6,
                token: None,
                api_token: None,
                message: Some("Registration is disabled".to_string()),
            }),
        );
    }

    let register_need_token = std::env::var("REGISTER_NEED_TOKEN")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

    if register_need_token {
        let token = std::env::var("REGISTER_TOKEN").unwrap_or_default();
        if token.is_empty() {
            return (
                StatusCode::UNAUTHORIZED,
                AxumJson(RegisterResponse {
                    status: -7,
                    token: None,
                    api_token: None,
                    message: Some("Registration token is not configured".to_string()),
                }),
            );
        }
    }

    let config_register_token = std::env::var("REGISTER_TOKEN")
        .unwrap_or_default();

    if register_need_token && payload.register_token.as_deref() != Some(config_register_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            AxumJson(RegisterResponse {
                status: -8,
                token: None,
                api_token: None,
                message: Some("Invalid registration token".to_string()),
            }),
        );
    }

    let (username, password) = match (payload.username, payload.password) {
        (Some(u), Some(p)) => (u, p),
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                AxumJson(RegisterResponse {
                    status: -1,
                    token: None,
                    api_token: None,
                    message: Some("Please enter username or password".to_string()),
                }),
            );
        }
    };

    match sqlx::query!("SELECT id FROM users WHERE username = ?", username)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                AxumJson(RegisterResponse {
                    status: -2,
                    token: None,
                    api_token: None,
                    message: Some("Username already exists".to_string()),
                }),
            );
        }
        Ok(None) => {}
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(RegisterResponse {
                    status: -1,
                    token: None,
                    api_token: None,
                    message: Some("Database error".to_string()),
                }),
            );
        }
    };
    
    let password_hash = match hash_password(password) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(RegisterResponse {
                    status: -1,
                    token: None,
                    api_token: None,
                    message: Some("Failed to hash password".to_string()),
                }),
            );
        }
    };

    let raw_api_token = uuid::Uuid::new_v4().to_string();
    let api_token_hash = hash_api_token(&raw_api_token);

    let insert_result = sqlx::query!(
        "INSERT INTO users (username, password_hash, api_token_hash) VALUES (?, ?, ?)",
        username,
        password_hash,
        api_token_hash
    )
    .execute(&pool)
    .await;

    let user_id = match insert_result {
        Ok(res) => res.last_insert_id() as i64,
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(RegisterResponse {
                    status: -1,
                    token: None,
                    api_token: None,
                    message: Some("Failed to register user".to_string()),
                }),
            );
        }
    };

    let token = match build_claims(user_id, username.clone(), &std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(RegisterResponse {
                    status: -1,
                    token: None,
                    api_token: None,
                    message: Some("Failed to generate token".to_string()),
                }),
            );
        }
    };

    (
        StatusCode::OK,
        AxumJson(RegisterResponse {
            status: 0,
            token: Some(token),
            api_token: Some(raw_api_token),
            message: None,
        }),
    )
}

#[derive(Serialize)]
pub struct TokenRegenerateResponse {
    pub status: i32,
    pub api_token: Option<String>,
    pub message: Option<String>,
}

fn hash_api_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn regenerate_api_token(
    State(pool): State<MySqlPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = hash_api_token(&raw_token);

    match sqlx::query!(
        "UPDATE users SET api_token_hash = ? WHERE id = ?",
        token_hash,
        auth_user.user_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            AxumJson(TokenRegenerateResponse {
                status: 0,
                api_token: Some(raw_token),
                message: None,
            }),
        ),
        Err(e) => {
            log::error!("Failed to regenerate token: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(TokenRegenerateResponse {
                    status: -1,
                    api_token: None,
                    message: Some("Failed to regenerate token".to_string()),
                }),
            )
        }
    }
}

pub fn hash_password(
    password: String
) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(
        password.as_bytes(),
        &salt,
    )?;

    Ok(password_hash.to_string())
}

#[derive(Serialize)]
pub struct ConfigResponse {
    pub allow_register: bool,
    pub register_need_token: bool,
}

pub async fn get_config() -> impl IntoResponse {
    let allow_register = std::env::var("ALLOW_REGISTER")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase();
    
    let register_need_token = std::env::var("REGISTER_NEED_TOKEN")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase();

    (
        StatusCode::OK,
        AxumJson(ConfigResponse {
            allow_register: allow_register != "false",
            register_need_token: register_need_token != "false",
        }),
    )
}