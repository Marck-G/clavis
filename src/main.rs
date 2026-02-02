use axum::{
    Router,
    response::Json,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod ldap;

use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // load dotenv if is not production env found
    if std::env::var("PRODUCTION").is_err() {
        dotenv::dotenv().ok();
    }

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Session store (MemoryStore for Phase 1/2, later maybe Redis or similar if needed)
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_same_site(tower_sessions::cookie::SameSite::Lax) // Strict might be too much for dev if ports differ
        .with_expiry(Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::minutes(10),
        )); // 10 minutes

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/change-password", post(auth::change_password))
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .fallback_service(ServeDir::new("static"));

    // Run it
    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:4000".to_string());
    let addr: SocketAddr = bind_address.parse().expect("Invalid BIND_ADDRESS");

    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "version": "0.1.0" }))
}
