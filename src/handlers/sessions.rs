use axum::{extract::Path, extract::State, response::Json};
use std::sync::Arc;

use crate::{
    error::AppError,
    models::{
        AppState, CreateSessionRequest, CreateSessionResponse, GenerateRecommendationsRequest,
        JoinSessionRequest, JoinSessionResponse, SessionStatusResponse,
        UpdateParticipantLocationRequest,
    },
    services::session::SessionService,
};

/// POST /sessions - Create a new session
pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, AppError> {
    tracing::info!("Creating new session: {}", request.name);

    let response = SessionService::create_session(&state, request).await?;

    tracing::info!(
        "Created session {} with join code {}",
        response.session_id,
        response.join_code
    );

    Ok(Json(response))
}

/// POST /sessions/join - Join an existing session
pub async fn join_session(
    State(state): State<Arc<AppState>>,
    Json(request): Json<JoinSessionRequest>,
) -> Result<Json<JoinSessionResponse>, AppError> {
    tracing::info!("User {} joining session", request.participant_name);

    let response = SessionService::join_session(&state, request).await?;

    tracing::info!(
        "User {} joined session {}",
        response.user_id,
        response.session.id
    );

    Ok(Json(response))
}

/// PUT /sessions/{session_id}/participants/{user_id}/location - Update participant location
pub async fn update_participant_location(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateParticipantLocationRequest>,
) -> Result<Json<SessionStatusResponse>, AppError> {
    tracing::info!(
        "Updating location for user {} in session {}",
        request.user_id,
        request.session_id
    );

    let response = SessionService::update_participant_location(&state, request).await?;

    Ok(Json(response))
}

/// POST /sessions/{session_id}/recommendations - Generate recommendations for session
pub async fn generate_session_recommendations(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenerateRecommendationsRequest>,
) -> Result<Json<SessionStatusResponse>, AppError> {
    let session_id = request.session_id.clone();
    tracing::info!("Generating recommendations for session {}", session_id);

    let response = SessionService::generate_recommendations(&state, request).await?;

    tracing::info!("Generated recommendations for session {}", session_id);

    Ok(Json(response))
}

/// GET /sessions/{session_id} - Get session status and current state
pub async fn get_session_status(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionStatusResponse>, AppError> {
    tracing::debug!("Getting status for session {}", session_id);

    let response = SessionService::get_session_status(&state, &session_id).await?;

    Ok(Json(response))
}

/// POST /sessions/cleanup - Admin endpoint to clean up expired sessions
pub async fn cleanup_expired_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!("Cleaning up expired sessions");

    SessionService::cleanup_expired_sessions(&state).await;

    Ok(Json(serde_json::json!({
        "message": "Expired sessions cleaned up",
        "timestamp": chrono::Utc::now()
    })))
}

/// GET /sessions/{session_id}/health - Health check for session (polling endpoint)
pub async fn session_health_check(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| AppError::BadRequest("Session not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "session_id": session.id,
        "status": session.status,
        "participant_count": session.participants.len(),
        "ready_count": session.participants.values().filter(|p| p.is_ready).count(),
        "updated_at": session.updated_at
    })))
}
