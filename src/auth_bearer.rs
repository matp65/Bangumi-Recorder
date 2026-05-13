use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use axum::{
    body::Body,
    extract::{Json, State},
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use bcrypt::verify as bcrypt_verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
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
    pub password: Option<String>
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub status: i32,
    pub token: Option<String>,
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
        exp: (now + Duration::hours(24)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

fn verify_password(password: &str, password_hash: &str) -> bool {
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
    let (username, password) = match (payload.username, payload.password) {
        (Some(u), Some(p)) => (u, p),
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                AxumJson(RegisterResponse {
                    status: -1,
                    token: None,
                    message: Some("Please enter username or password".to_string()),
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
                    message: Some("Failed to hash password".to_string()),
                }),
            );
        }
    };

    let uuid = uuid::Uuid::new_v4().to_string();
    let token = match build_claims(0, username.clone(), &std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(RegisterResponse {
                    status: -1,
                    token: None,
                    message: Some("Failed to generate token".to_string()),
                }),
            );
        }
    };
    match sqlx::query!(
        "INSERT INTO users (username, password_hash, api_token_hash) VALUES (?, ?, ?)",
        username,
        password_hash,
        uuid
    )
    .execute(&pool)
    .await
    {
        Ok(_) => {
            (
                StatusCode::OK,
                AxumJson(RegisterResponse {
                    status: 0,
                    token: Some(token),
                    message: None,
                }),
            )
        }
        Err(e) => {
            log::error!("DB error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(RegisterResponse {
                    status: -1,
                    token: None,
                    message: Some("Failed to register user".to_string()),
                }),
            );
        }
    }
}

fn hash_password(
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