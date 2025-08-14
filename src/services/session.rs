use chrono::{Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        AppState, CreateSessionRequest, CreateSessionResponse,
        JoinSessionRequest, JoinSessionResponse,
        UpdateParticipantLocationRequest, GenerateRecommendationsRequest,
        SessionStatusResponse,
        Session, Participant, SessionStatus, ParticipantInput,
        TransportMode, EnhancedVenueRecommendation,
    },
    services::{
        geometry::GeometryService,
        google_maps::GoogleMapsService,
        geocoding::GeocodingService,
        ranking::RankingService,
    },
};

pub struct SessionService;

impl SessionService {
    /// Create a new session
    pub async fn create_session(
        state: &AppState,
        request: CreateSessionRequest,
    ) -> Result<CreateSessionResponse, AppError> {
        let session_id = Uuid::new_v4().to_string();
        let creator_id = Uuid::new_v4().to_string();
        let join_code = Self::generate_join_code();
        
        let creator = Participant {
            user_id: creator_id.clone(),
            name: request.creator_name,
            location: None,
            joined_at: Utc::now(),
            is_ready: false,
        };

        let mut participants = HashMap::new();
        participants.insert(creator_id.clone(), creator);

        let session = Session {
            id: session_id.clone(),
            name: request.name,
            creator_id: creator_id.clone(),
            participants,
            settings: request.settings,
            status: SessionStatus::WaitingForParticipants,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24), // 24-hour expiry
        };

        // Store session
        let mut sessions = state.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());
        
        // Also store by join code for easy lookup
        sessions.insert(join_code.clone(), session.clone());

        tracing::info!("Created session {} with join code {}", session_id, join_code);

        Ok(CreateSessionResponse {
            session_id,
            join_code,
            session,
        })
    }

    /// Join an existing session
    pub async fn join_session(
        state: &AppState,
        request: JoinSessionRequest,
    ) -> Result<JoinSessionResponse, AppError> {
        let session_id = if let Some(id) = request.session_id {
            id
        } else if let Some(code) = request.join_code {
            code
        } else {
            return Err(AppError::BadRequest("Either session_id or join_code is required".to_string()));
        };

        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id)
            .ok_or_else(|| AppError::BadRequest("Session not found".to_string()))?;

        // Check if session is expired
        if Utc::now() > session.expires_at {
            session.status = SessionStatus::Expired;
            return Err(AppError::BadRequest("Session has expired".to_string()));
        }

        let user_id = Uuid::new_v4().to_string();
        let participant = Participant {
            user_id: user_id.clone(),
            name: request.participant_name,
            location: None,
            joined_at: Utc::now(),
            is_ready: false,
        };

        session.participants.insert(user_id.clone(), participant);
        session.updated_at = Utc::now();

        tracing::info!("User {} joined session {}", user_id, session_id);

        Ok(JoinSessionResponse {
            user_id,
            session: session.clone(),
        })
    }

    /// Update participant location
    pub async fn update_participant_location(
        state: &AppState,
        request: UpdateParticipantLocationRequest,
    ) -> Result<SessionStatusResponse, AppError> {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&request.session_id)
            .ok_or_else(|| AppError::BadRequest("Session not found".to_string()))?;

        let participant = session.participants.get_mut(&request.user_id)
            .ok_or_else(|| AppError::BadRequest("Participant not found in session".to_string()))?;

        participant.location = Some(request.location);
        participant.is_ready = request.is_ready;
        session.updated_at = Utc::now();

        // Check if we should auto-generate recommendations
        let should_generate = Self::should_auto_generate_recommendations(session);
        
        if should_generate {
            session.status = SessionStatus::ReadyForRecommendations;
        }

        tracing::info!("Updated location for user {} in session {}", request.user_id, request.session_id);

        Ok(SessionStatusResponse {
            session: session.clone(),
            recommendations: None,
        })
    }

    /// Generate recommendations for a session
    pub async fn generate_recommendations(
        state: &AppState,
        request: GenerateRecommendationsRequest,
    ) -> Result<SessionStatusResponse, AppError> {
        // Get session
        let session = {
            let sessions = state.sessions.read().await;
            sessions.get(&request.session_id)
                .ok_or_else(|| AppError::BadRequest("Session not found".to_string()))?
                .clone()
        };

        // Verify user is in session
        if !session.participants.contains_key(&request.user_id) {
            return Err(AppError::BadRequest("User not found in session".to_string()));
        }

        // Update session status
        {
            let mut sessions = state.sessions.write().await;
            if let Some(session) = sessions.get_mut(&request.session_id) {
                session.status = SessionStatus::GeneratingRecommendations;
                session.updated_at = Utc::now();
            }
        }

        // Collect participant locations
        let participant_inputs: Vec<ParticipantInput> = session.participants
            .values()
            .filter_map(|p| p.location.as_ref())
            .cloned()
            .collect();

        if participant_inputs.len() < 2 {
            return Err(AppError::BadRequest("At least 2 participants with locations are required".to_string()));
        }

        // Generate recommendations using existing logic
        let recommendations = Self::generate_recommendations_internal(
            state,
            participant_inputs,
            session.settings.categories.clone(),
            session.settings.transport_mode.clone(),
            session.settings.limit,
        ).await?;

        // Update session with results
        {
            let mut sessions = state.sessions.write().await;
            if let Some(session) = sessions.get_mut(&request.session_id) {
                session.status = SessionStatus::RecommendationsReady;
                session.updated_at = Utc::now();
            }
        }

        let updated_session = {
            let sessions = state.sessions.read().await;
            sessions.get(&request.session_id).unwrap().clone()
        };

        tracing::info!("Generated {} recommendations for session {}", recommendations.len(), request.session_id);

        Ok(SessionStatusResponse {
            session: updated_session,
            recommendations: Some(recommendations),
        })
    }

    /// Get session status
    pub async fn get_session_status(
        state: &AppState,
        session_id: &str,
    ) -> Result<SessionStatusResponse, AppError> {
        let sessions = state.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| AppError::BadRequest("Session not found".to_string()))?;

        Ok(SessionStatusResponse {
            session: session.clone(),
            recommendations: None, // Could store recommendations in session if needed
        })
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(state: &AppState) {
        let mut sessions = state.sessions.write().await;
        let now = Utc::now();
        
        sessions.retain(|_, session| {
            if now > session.expires_at {
                tracing::info!("Removing expired session {}", session.id);
                false
            } else {
                true
            }
        });
    }

    // Helper methods
    fn generate_join_code() -> String {
        // Generate a 6-character alphanumeric code
        use rand::Rng;
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        
        (0..6)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect()
    }

    fn should_auto_generate_recommendations(session: &Session) -> bool {
        if !session.settings.auto_refresh {
            return false;
        }

        let ready_count = session.participants.values()
            .filter(|p| p.is_ready && p.location.is_some())
            .count();

        if session.settings.require_all_participants {
            ready_count == session.participants.len()
        } else {
            ready_count >= 2
        }
    }

    async fn generate_recommendations_internal(
        state: &AppState,
        participant_inputs: Vec<ParticipantInput>,
        categories: Vec<String>,
        transport_mode: crate::models::TransportMode,
        limit: usize,
    ) -> Result<Vec<EnhancedVenueRecommendation>, AppError> {
        // Use existing recommendation logic
        let google_maps_service = GoogleMapsService::new(
            state.http_client.clone(),
            state.config.google_maps_api_key.clone(),
        );
        let geocoding_service = GeocodingService::new(state.config.google_maps_api_key.clone());

        // Resolve participant locations
        let participant_coordinates = geocoding_service
            .resolve_participants(&participant_inputs)
            .await
            .map_err(|e| AppError::ExternalApi(format!("Geocoding failed: {}", e)))?;

        // Calculate search center
        let search_center = GeometryService::calculate_search_center(&participant_coordinates)?;

        // Search for places
        let places = google_maps_service
            .search_places(&search_center, &categories)
            .await?;

        if places.is_empty() {
            return Ok(vec![]);
        }

        // Get travel times
        let venues_with_times = google_maps_service
            .get_travel_times(&participant_coordinates, &places, &transport_mode)
            .await?;

        // Rank venues
        let ranked_venues = RankingService::rank_venues(
            venues_with_times, 
            participant_coordinates.len() == 2
        );

        // Convert to enhanced response format
        let transport_mode_str = match transport_mode {
            crate::models::TransportMode::Drive => "drive",
            crate::models::TransportMode::Walk => "walk",
            crate::models::TransportMode::Transit => "transit",
            crate::models::TransportMode::Bicycle => "bicycle",
        };

        let recommendations: Vec<EnhancedVenueRecommendation> = ranked_venues
            .into_iter()
            .take(limit)
            .map(|venue| {
                let travel_times_info = venue.travel_times
                    .iter()
                    .enumerate()
                    .map(|(index, &time)| crate::models::TravelTimeInfo {
                        participant_index: index,
                        travel_time_minutes: time,
                        transport_mode: transport_mode_str.to_string(),
                        google_maps_directions_url: geocoding_service.generate_directions_url(
                            &participant_coordinates[index],
                            &venue.place.coordinate,
                            transport_mode_str,
                        ),
                    })
                    .collect();

                EnhancedVenueRecommendation {
                    name: venue.place.name,
                    address: venue.address,
                    lat: venue.place.coordinate.lat,
                    lng: venue.place.coordinate.lng,
                    rating: venue.place.rating,
                    reviews: venue.place.user_ratings_total,
                    google_maps_url: geocoding_service.generate_venue_url(
                        &venue.place.coordinate,
                        Some(&venue.place.place_id)
                    ),
                    travel_times: travel_times_info,
                    category: venue
                        .place
                        .types
                        .first()
                        .unwrap_or(&"unknown".to_string())
                        .clone(),
                    price_level: venue.place.price_level,
                }
            })
            .collect();

        Ok(recommendations)
    }
}
