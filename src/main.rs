use axum::{
    Router, middleware, routing::{get, post}, response::Html
};
use tower_http::services::ServeDir;
use dotenvy::dotenv;
use sqlx::MySqlPool;
use tower_http::cors::CorsLayer;
use http::Method;

mod api;
mod auth_bearer;

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
        let html = std::fs::read_to_string("frontend/dist/index.html")
            .unwrap_or_else(|_| {
                std::fs::read_to_string("frontend/index.html")
                    .expect("failed to read index.html")
            });

        Html(html)
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
        .with_state(pool.clone());

    let app = Router::new()
        .route("/", get(index))
        .route("/auth/login", post(auth_bearer::login))
        .route("/auth/register", post(auth_bearer::register))
        .nest("/api/v1", api_router)
        .nest("/api/v1/open", open_router)
        .nest_service("/assets", ServeDir::new("frontend/dist/assets"))
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