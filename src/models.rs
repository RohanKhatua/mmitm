use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub http_client: reqwest::Client,
    pub sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Session>>>, // Session storage
}

// Session Management Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub creator_id: String,
    pub participants: HashMap<String, Participant>, // user_id -> Participant
    pub settings: SessionSettings,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub user_id: String,
    pub name: String,
    pub location: Option<ParticipantInput>,
    pub joined_at: DateTime<Utc>,
    pub is_ready: bool, // Has confirmed their location
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub categories: Vec<String>,
    pub transport_mode: TransportMode,
    pub limit: usize,
    pub auto_refresh: bool, // Auto-generate recommendations as people join
    pub require_all_participants: bool, // Wait for everyone to be ready
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    WaitingForParticipants,
    ReadyForRecommendations,
    GeneratingRecommendations,
    RecommendationsReady,
    Expired,
}

// Session API Request/Response Models
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub creator_name: String,
    pub settings: SessionSettings,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub join_code: String, // Short code for easy sharing
    pub session: Session,
}

#[derive(Debug, Deserialize)]
pub struct JoinSessionRequest {
    pub session_id: Option<String>, // Either session_id or join_code
    pub join_code: Option<String>,
    pub participant_name: String,
}

#[derive(Debug, Serialize)]
pub struct JoinSessionResponse {
    pub user_id: String,
    pub session: Session,
}

#[derive(Debug, Deserialize)]
pub struct UpdateParticipantLocationRequest {
    pub session_id: String,
    pub user_id: String,
    pub location: ParticipantInput,
    pub is_ready: bool,
}

#[derive(Debug, Serialize)]
pub struct SessionStatusResponse {
    pub session: Session,
    pub recommendations: Option<Vec<EnhancedVenueRecommendation>>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRecommendationsRequest {
    pub session_id: String,
    pub user_id: String, // Only creator or participants can trigger
}

#[derive(Debug, Deserialize)]
pub struct RecommendationRequest {
    pub participants: Vec<ParticipantInput>, // Changed from Coordinate
    pub categories: Vec<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub transport_mode: TransportMode, // NEW
}

// NEW: Support both coordinates and addresses
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ParticipantInput {
    Coordinate(Coordinate),
    Address { address: String },
}

// NEW: Transport mode options
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransportMode {
    Drive,
    Walk,
    Transit,
    Bicycle,
}

impl Default for TransportMode {
    fn default() -> Self {
        TransportMode::Drive
    }
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize)]
pub struct VenueRecommendation {
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub rating: Option<f64>,
    pub reviews: Option<u32>,
    pub google_url: String,
    pub travel_times: Vec<u32>, // Travel times in minutes for each participant
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_level: Option<String>, // Google Places API (New) returns strings
}

#[derive(Debug, Clone)]
pub struct SearchCenter {
    pub coordinate: Coordinate,
    pub radius: f64, // in meters
}

#[derive(Debug, Clone)]
pub struct PlaceCandidate {
    pub place_id: String,
    pub name: String,
    pub coordinate: Coordinate,
    pub types: Vec<String>,
    pub rating: Option<f64>,
    pub user_ratings_total: Option<u32>,
    pub price_level: Option<String>, // Google Places API (New) returns strings
}

#[derive(Debug)]
pub struct VenueWithTravelTimes {
    pub place: PlaceCandidate,
    pub address: String,
    pub google_url: String,
    pub travel_times: Vec<u32>, // in minutes
    pub total_travel_time: u32,
    pub max_travel_time: u32,
    pub min_travel_time: u32,
    pub fairness_score: f64, // Lower is better
}

// Google Places API (New) response structures
#[derive(Debug, Deserialize)]
pub struct GooglePlacesNewResponse {
    pub places: Vec<GooglePlaceNew>,
}

#[derive(Debug, Deserialize)]
pub struct GooglePlaceNew {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: GoogleDisplayName,
    pub location: GoogleLocationNew,
    pub types: Vec<String>,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(rename = "userRatingCount", default)]
    pub user_rating_count: Option<u32>,
    #[serde(rename = "priceLevel", default)]
    pub price_level: Option<String>, // Changed from u8 to String for new API
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GoogleDisplayName {
    pub text: String,
    #[serde(rename = "languageCode", default)]
    pub language_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleLocationNew {
    pub latitude: f64,
    pub longitude: f64,
}

// Google Places Details (New) API response
#[derive(Debug, Deserialize)]
pub struct GooglePlaceDetailsNewResponse {
    #[serde(rename = "formattedAddress")]
    pub formatted_address: String,
    #[serde(rename = "googleMapsUri")]
    pub google_maps_uri: String,
}

// Legacy Google Places API response structures (kept for migration)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GooglePlacesResponse {
    pub results: Vec<GooglePlace>,
    pub status: String,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GooglePlace {
    pub place_id: String,
    pub name: String,
    pub geometry: GoogleGeometry,
    pub types: Vec<String>,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub user_ratings_total: Option<u32>,
    #[serde(default)]
    pub price_level: Option<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GoogleGeometry {
    pub location: GoogleLocation,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GoogleLocation {
    pub lat: f64,
    pub lng: f64,
}

// Google Places Details API response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GooglePlaceDetailsResponse {
    pub result: GooglePlaceDetails,
    pub status: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GooglePlaceDetails {
    pub formatted_address: String,
    pub url: String,
}

// Google Geocoding API response structures
#[derive(Debug, Deserialize)]
pub struct GoogleGeocodingResponse {
    pub results: Vec<GoogleGeocodeResult>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleGeocodeResult {
    pub formatted_address: String,
    pub geometry: GoogleGeometry,
    pub place_id: String,
}

// Enhanced venue recommendation with Google Maps URL
#[derive(Debug, Serialize)]
pub struct EnhancedVenueRecommendation {
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub rating: Option<f64>,
    pub reviews: Option<u32>,
    pub google_maps_url: String,           // Direct link to Google Maps
    pub travel_times: Vec<TravelTimeInfo>, // Per-participant travel info
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TravelTimeInfo {
    pub participant_index: usize,
    pub travel_time_minutes: u32,
    pub transport_mode: String,
    pub google_maps_directions_url: String, // Direct navigation link
}

// Google Distance Matrix API response
#[derive(Debug, Deserialize)]
pub struct GoogleDistanceMatrixResponse {
    pub rows: Vec<GoogleDistanceRow>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleDistanceRow {
    pub elements: Vec<GoogleDistanceElement>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleDistanceElement {
    pub status: String,
    #[serde(default)]
    pub duration: Option<GoogleDuration>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleDuration {
    pub value: u32, // Duration in seconds
}
