use axum::{
    Router, middleware, routing::{get, post}, response::Html,
    http::header, response::IntoResponse,
};
use rust_embed::RustEmbed;
use dotenvy::dotenv;
use sqlx::MySqlPool;
use tower_http::cors::CorsLayer;
use http::Method;

mod api;
mod auth_bearer;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Assets;

fn guess_mime(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");


    let listen = format!("{}:{}",
        std::env::var("LISTEN").unwrap_or_else(|_| "127.0.0.1".to_string()),
        std::env::var("LISTEN_PORT").unwrap_or_else(|_| "8080".to_string()));

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    async fn index() -> Html<String> {
        let html = Assets::get("index.html")
            .expect("index.html not found in embedded assets")
            .data;
        Html(String::from_utf8_lossy(&html).into_owned())
    }

    async fn serve_fallback(uri: axum::http::Uri) -> impl IntoResponse {
        let path = uri.path().trim_start_matches('/');
        if !path.is_empty() {
            if let Some(content) = Assets::get(path) {
                let mime = guess_mime(path);
                return ([(header::CONTENT_TYPE, mime)], content.data.into_owned()).into_response();
            }
        }
        let html = Assets::get("index.html")
            .expect("index.html not found in embedded assets")
            .data;
        Html(String::from_utf8_lossy(&html).into_owned()).into_response()
    }

    let api_router = Router::new()
        .route("/search/bangumi", post(api::search::search_bangumi_by_title))
        .route("/search/bangumi/id", post(api::search::search_bangumi_by_id))
        .route("/record/add", post(api::new::add_record))
        .route("/record/update", post(api::update_recorder::update_user_recorder))
        .route("/record/delete", post(api::delete_recorder::delete_recorder))
        .route("/record/get", post(api::get_recorder::get_recorder))
        .route("/record/list", get(api::list::list_recorder))
        .route("/record/detail_list", get(api::detail_list::get_detail_list))
        .route("/auth/token/regenerate", post(auth_bearer::regenerate_api_token))
        .route("/user/info", get(api::user::get_info))
        .route("/user/update", post(api::user::update_info))
        .route("/user/password", post(api::user::update_password))
        .with_state(pool.clone())
        .layer(middleware::from_fn(move |req, next| {
            let jwt_secret = jwt_secret.clone();
            async move { auth_bearer::jwt_auth(req, next, jwt_secret).await }
        }));

    let open_router = Router::new()
        .route("/new", post(api::open::new::add_record_open))
        .route("/update", post(api::open::update_recorder::update_user_recorder))
        .route("/delete", post(api::open::delete_recorder::delete_recorder))
        .route("/get", post(api::open::get_recorder::get_recorder))
        .route("/list", get(api::open::list::list_recorder))
        .route("/detail_list", get(api::open::detail_list::get_detail_list))
        .route("/user/info", get(api::open::user::get_info))
        .with_state(pool.clone());

    let app = Router::new()
        .route("/", get(index))
        .route("/auth/login", post(auth_bearer::login))
        .route("/auth/register", post(auth_bearer::register))
        .route("/auth/config", get(auth_bearer::get_config))
        .nest("/api/v1", api_router)
        .nest("/api/v1/open", open_router)
        .fallback(get(serve_fallback))
        .with_state(pool)
        .layer(
            CorsLayer::permissive()
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        );

    log::info!("Listening on http://{}", listen);

    let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}
