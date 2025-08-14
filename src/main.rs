use axum::{
    response::Json,
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod models;
mod services;

use config::Config;
use error::AppError;
use handlers::{
    recommendations::get_recommendations,
    sessions::{
        cleanup_expired_sessions, create_session, generate_session_recommendations,
        get_session_status, join_session, session_health_check, update_participant_location,
    },
};
use models::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mmitm=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    // Create HTTP client
    let http_client = reqwest::Client::new();

    // Create application state
    let state = Arc::new(AppState {
        config,
        http_client,
        sessions: Default::default(),
    });

    // Build the application router
    let app = Router::new()
        // Original recommendations endpoint
        .route("/recommendations", post(get_recommendations))
        // Session management endpoints
        .route("/sessions", post(create_session))
        .route("/sessions/join", post(join_session))
        .route(
            "/sessions/:session_id/participants/:user_id/location",
            put(update_participant_location),
        )
        .route(
            "/sessions/:session_id/recommendations",
            post(generate_session_recommendations),
        )
        .route("/sessions/:session_id", get(get_session_status))
        .route("/sessions/:session_id/health", get(session_health_check))
        // Admin endpoint
        .route("/sessions/cleanup", post(cleanup_expired_sessions))
        // Health check
        .route("/health", get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state.clone());

    // Start the server
    let port = state.config.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    tracing::info!("Server starting on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "mmitm",
        "version": env!("CARGO_PKG_VERSION")
    })))
}
