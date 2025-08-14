use axum::{extract::State, response::Json};
use std::sync::Arc;

use crate::{
    error::AppError,
    models::{AppState, EnhancedVenueRecommendation, RecommendationRequest, TravelTimeInfo},
    services::{
        geocoding::GeocodingService, geometry::GeometryService, google_maps::GoogleMapsService,
        ranking::RankingService, validation::PlaceTypeValidator,
    },
};

pub async fn get_recommendations(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RecommendationRequest>,
) -> Result<Json<Vec<EnhancedVenueRecommendation>>, AppError> {
    tracing::info!(
        "Processing recommendation request for {} participants with transport mode: {:?}",
        request.participants.len(),
        request.transport_mode
    );

    // Validate request
    if request.participants.len() < 2 {
        return Err(AppError::BadRequest(
            "At least 2 participants are required".to_string(),
        ));
    }

    if request.categories.is_empty() {
        return Err(AppError::BadRequest(
            "At least one category is required".to_string(),
        ));
    }

    // Validate place types
    PlaceTypeValidator::validate_place_types(&request.categories)
        .map_err(|err| AppError::BadRequest(format!("Invalid place types: {}", err)))?;

    // Initialize services
    let google_maps_service = GoogleMapsService::new(
        state.http_client.clone(),
        state.config.google_maps_api_key.clone(),
    );
    let geocoding_service = GeocodingService::new(state.config.google_maps_api_key.clone());

    // Step 1: Resolve participant inputs to coordinates
    let participant_coordinates = geocoding_service
        .resolve_participants(&request.participants)
        .await
        .map_err(|e| AppError::ExternalApi(format!("Geocoding failed: {}", e)))?;

    tracing::debug!(
        "Resolved {} participant locations",
        participant_coordinates.len()
    );

    // Step 2: Calculate geometric center and search radius
    let search_center = GeometryService::calculate_search_center(&participant_coordinates)?;

    tracing::debug!(
        "Search center: lat={}, lng={}, radius={}m",
        search_center.coordinate.lat,
        search_center.coordinate.lng,
        search_center.radius
    );

    // Step 3: Search for places
    let places = google_maps_service
        .search_places(&search_center, &request.categories)
        .await?;

    if places.is_empty() {
        tracing::info!("No places found for the given criteria");
        return Ok(Json(vec![]));
    }

    tracing::info!("Found {} candidate places", places.len());

    // Step 4: Get travel times with transport mode
    let venues_with_travel_times = google_maps_service
        .get_travel_times(&participant_coordinates, &places, &request.transport_mode)
        .await?;

    if venues_with_travel_times.is_empty() {
        tracing::info!("No venues with valid travel times");
        return Ok(Json(vec![]));
    }

    // Step 5: Rank venues
    let ranked_venues =
        RankingService::rank_venues(venues_with_travel_times, participant_coordinates.len() == 2);

    // Step 6: Convert to enhanced response format with Google Maps URLs
    let transport_mode_str = match request.transport_mode {
        crate::models::TransportMode::Drive => "drive",
        crate::models::TransportMode::Walk => "walk",
        crate::models::TransportMode::Transit => "transit",
        crate::models::TransportMode::Bicycle => "bicycle",
    };

    let recommendations: Vec<EnhancedVenueRecommendation> = ranked_venues
        .into_iter()
        .take(request.limit)
        .map(|venue| {
            // Generate travel time info with directions URLs for each participant
            let travel_times_info: Vec<TravelTimeInfo> = venue
                .travel_times
                .iter()
                .enumerate()
                .map(|(index, &time)| TravelTimeInfo {
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
                google_maps_url: geocoding_service
                    .generate_venue_url(&venue.place.coordinate, Some(&venue.place.place_id)),
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

    tracing::info!(
        "Returning {} enhanced recommendations",
        recommendations.len()
    );

    Ok(Json(recommendations))
}
