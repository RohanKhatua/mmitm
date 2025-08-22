use reqwest::Client;
use std::collections::HashMap;

use crate::{
    error::AppError,
    models::{
        Coordinate, GoogleDistanceMatrixResponse, GooglePlaceDetailsNewResponse,
        GooglePlacesNewResponse, PlaceCandidate, SearchCenter, VenueWithTravelTimes,
    },
};

pub struct GoogleMapsService {
    client: Client,
    api_key: String,
}

impl GoogleMapsService {
    pub fn new(client: Client, api_key: String) -> Self {
        Self { client, api_key }
    }

    /// Search for places using Google Places Nearby Search (New) API
    pub async fn search_places(
        &self,
        search_center: &SearchCenter,
        categories: &[String],
    ) -> Result<Vec<PlaceCandidate>, AppError> {
        // Use the new Places API (New) with all categories in a single request
        let places = self
            .search_places_nearby_new(search_center, categories)
            .await?;
        Ok(places)
    }

    /// Search for places using Google Places Nearby Search (New) API
    async fn search_places_nearby_new(
        &self,
        search_center: &SearchCenter,
        categories: &[String],
    ) -> Result<Vec<PlaceCandidate>, AppError> {
        // Use the new Places API (New) endpoint
        let url = "https://places.googleapis.com/v1/places:searchNearby";

        // Construct request body for the new API
        let request_body = serde_json::json!({
            "includedTypes": categories,
            "maxResultCount": 20,
            "locationRestriction": {
                "circle": {
                    "center": {
                        "latitude": search_center.coordinate.lat,
                        "longitude": search_center.coordinate.lng
                    },
                    "radius": search_center.radius
                }
            },
            "rankPreference": "POPULARITY"
        });

        tracing::debug!(
            "Searching for places with types {:?} near {},{} with radius {}m",
            categories,
            search_center.coordinate.lat,
            search_center.coordinate.lng,
            search_center.radius
        );

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Goog-Api-Key", &self.api_key)
            .header("X-Goog-FieldMask", "places.id,places.displayName,places.location,places.types,places.rating,places.userRatingCount,places.priceLevel")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::GoogleApiError(format!(
                "Places API (New) request failed with status: {}",
                response.status()
            )));
        }

        let places_response: GooglePlacesNewResponse = response.json().await?;

        let candidates: Vec<PlaceCandidate> = places_response
            .places
            .into_iter()
            .map(|place| PlaceCandidate {
                place_id: place.id,
                name: place.display_name.text,
                coordinate: Coordinate {
                    lat: place.location.latitude,
                    lng: place.location.longitude,
                },
                types: place.types,
                rating: place.rating,
                user_ratings_total: place.user_rating_count,
                price_level: place.price_level,
            })
            .collect();

        tracing::debug!("Found {} places using new Places API", candidates.len());

        Ok(candidates)
    }

    /// Get travel times from all participants to all venues
    pub async fn get_travel_times(
        &self,
        participants: &[Coordinate],
        places: &[PlaceCandidate],
        transport_mode: &crate::models::TransportMode,
    ) -> Result<Vec<VenueWithTravelTimes>, AppError> {
        let mut venues_with_times = Vec::new();

        // Process places in batches to avoid API limits
        for place_batch in places.chunks(25) {
            let batch_results = self
                .get_travel_times_batch(participants, place_batch, transport_mode)
                .await?;
            venues_with_times.extend(batch_results);
        }

        Ok(venues_with_times)
    }

    /// Get travel times for a batch of places
    async fn get_travel_times_batch(
        &self,
        participants: &[Coordinate],
        places: &[PlaceCandidate],
        transport_mode: &crate::models::TransportMode,
    ) -> Result<Vec<VenueWithTravelTimes>, AppError> {
        // Get place details for addresses and Google URLs
        let place_details = self.get_place_details_batch(places).await?;

        // Get travel times using Distance Matrix API
        let travel_times = self
            .get_distance_matrix(participants, places, transport_mode)
            .await?;

        let mut venues = Vec::new();

        for (i, place) in places.iter().enumerate() {
            if let (Some(details), Some(times)) =
                (place_details.get(&place.place_id), travel_times.get(i))
            {
                if times.iter().all(|&time| time > 0) {
                    // All participants have valid travel times
                    let total_time: u32 = times.iter().sum();
                    let max_time = *times.iter().max().unwrap();
                    let min_time = *times.iter().min().unwrap();
                    let fairness_score = (max_time - min_time) as f64;

                    venues.push(VenueWithTravelTimes {
                        place: place.clone(),
                        address: details.address.clone(),
                        google_url: details.url.clone(),
                        travel_times: times.clone(),
                        total_travel_time: total_time,
                        max_travel_time: max_time,
                        min_travel_time: min_time,
                        fairness_score,
                    });
                }
            }
        }

        Ok(venues)
    }

    /// Get place details for multiple places
    async fn get_place_details_batch(
        &self,
        places: &[PlaceCandidate],
    ) -> Result<HashMap<String, PlaceDetails>, AppError> {
        let mut details_map = HashMap::new();

        // Process in smaller batches to avoid overwhelming the API
        for place_batch in places.chunks(10) {
            let batch_futures = place_batch
                .iter()
                .map(|place| self.get_place_details(&place.place_id));

            let batch_results = futures::future::join_all(batch_futures).await;

            for (place, result) in place_batch.iter().zip(batch_results) {
                if let Ok(details) = result {
                    details_map.insert(place.place_id.clone(), details);
                }
            }
        }

        Ok(details_map)
    }

    /// Get details for a single place using Place Details (New) API
    async fn get_place_details(&self, place_id: &str) -> Result<PlaceDetails, AppError> {
        // Use the new Place Details (New) API endpoint
        let url = format!("https://places.googleapis.com/v1/places/{}", place_id);

        let response = self
            .client
            .get(&url)
            .header("X-Goog-Api-Key", &self.api_key)
            .header("X-Goog-FieldMask", "formattedAddress,googleMapsUri")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::GoogleApiError(format!(
                "Place Details (New) API request failed with status: {}",
                response.status()
            )));
        }

        let details_response: GooglePlaceDetailsNewResponse = response.json().await?;

        Ok(PlaceDetails {
            address: details_response.formatted_address,
            url: details_response.google_maps_uri,
        })
    }

    /// Get travel times using Distance Matrix API
    async fn get_distance_matrix(
        &self,
        participants: &[Coordinate],
        places: &[PlaceCandidate],
        transport_mode: &crate::models::TransportMode,
    ) -> Result<Vec<Vec<u32>>, AppError> {
        let origins: Vec<String> = participants
            .iter()
            .map(|coord| format!("{},{}", coord.lat, coord.lng))
            .collect();

        let destinations: Vec<String> = places
            .iter()
            .map(|place| format!("{},{}", place.coordinate.lat, place.coordinate.lng))
            .collect();

        let url = "https://maps.googleapis.com/maps/api/distancematrix/json";

        let mode = match transport_mode {
            crate::models::TransportMode::Drive => "driving",
            crate::models::TransportMode::Walk => "walking",
            crate::models::TransportMode::Transit => "transit",
            crate::models::TransportMode::Bicycle => "bicycling",
        };

        let params = [
            ("origins", origins.join("|")),
            ("destinations", destinations.join("|")),
            ("mode", mode.to_string()),
            ("units", "metric".to_string()),
            ("key", self.api_key.clone()),
        ];

        tracing::debug!(
            "Getting travel times for {} origins to {} destinations",
            origins.len(),
            destinations.len()
        );

        let response = self.client.get(url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(AppError::GoogleApiError(format!(
                "Distance Matrix API request failed with status: {}",
                response.status()
            )));
        }

        let matrix_response: GoogleDistanceMatrixResponse = response.json().await?;

        if matrix_response.status != "OK" {
            return Err(AppError::GoogleApiError(format!(
                "Distance Matrix API error: {}",
                matrix_response.status
            )));
        }

        // Convert response to travel times matrix
        let mut travel_times = vec![vec![0u32; places.len()]; participants.len()];

        for (participant_idx, row) in matrix_response.rows.iter().enumerate() {
            for (place_idx, element) in row.elements.iter().enumerate() {
                if element.status == "OK" {
                    if let Some(duration) = &element.duration {
                        travel_times[participant_idx][place_idx] = duration.value / 60;
                        // Convert to minutes
                    }
                }
            }
        }

        // Transpose matrix to get travel times per place
        let mut transposed = vec![vec![0u32; participants.len()]; places.len()];
        for (i, row) in travel_times.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                transposed[j][i] = *value;
            }
        }

        Ok(transposed)
    }
}

#[derive(Debug, Clone)]
struct PlaceDetails {
    address: String,
    url: String,
}
