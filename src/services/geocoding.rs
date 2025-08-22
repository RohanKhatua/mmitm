use crate::models::{Coordinate, GoogleGeocodingResponse, ParticipantInput};
use reqwest::Client;
use std::error::Error;

pub struct GeocodingService {
    client: Client,
    api_key: String,
}

impl GeocodingService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Convert participant inputs (addresses or coordinates) to coordinates
    pub async fn resolve_participants(
        &self,
        participants: &[ParticipantInput],
    ) -> Result<Vec<Coordinate>, Box<dyn Error + Send + Sync>> {
        let mut resolved_coordinates = Vec::new();

        for participant in participants {
            let coordinate = match participant {
                ParticipantInput::Coordinate(coord) => coord.clone(),
                ParticipantInput::Address { address } => {
                    tracing::debug!("Geocoding address: {}", address);
                    self.geocode_address(address).await?
                }
            };
            resolved_coordinates.push(coordinate);
        }

        tracing::info!(
            "Resolved {} participant locations",
            resolved_coordinates.len()
        );
        Ok(resolved_coordinates)
    }

    /// Geocode a single address to coordinates
    async fn geocode_address(
        &self,
        address: &str,
    ) -> Result<Coordinate, Box<dyn Error + Send + Sync>> {
        let url = "https://maps.googleapis.com/maps/api/geocode/json";

        let response = self
            .client
            .get(url)
            .query(&[("address", address), ("key", &self.api_key)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Geocoding API request failed: {}", response.status()).into());
        }

        let geocoding_response: GoogleGeocodingResponse = response.json().await?;

        if geocoding_response.status != "OK" {
            return Err(format!(
                "Geocoding failed with status: {}",
                geocoding_response.status
            )
            .into());
        }

        if geocoding_response.results.is_empty() {
            return Err(format!("No results found for address: {}", address).into());
        }

        let result = &geocoding_response.results[0];
        let location = &result.geometry.location;

        tracing::debug!(
            "Geocoded '{}' to {}, {}",
            address,
            location.lat,
            location.lng
        );

        Ok(Coordinate {
            lat: location.lat,
            lng: location.lng,
        })
    }

    /// Generate Google Maps directions URL
    pub fn generate_directions_url(
        &self,
        origin: &Coordinate,
        destination: &Coordinate,
        transport_mode: &str,
    ) -> String {
        let mode = match transport_mode.to_lowercase().as_str() {
            "walk" => "walking",
            "transit" => "transit",
            "bicycle" => "bicycling",
            _ => "driving",
        };

        format!(
            "https://www.google.com/maps/dir/{},{}/{},{}/@{},{},15z/data=!3m1!4b1!4m2!4m1!3e{}",
            origin.lat,
            origin.lng,
            destination.lat,
            destination.lng,
            destination.lat,
            destination.lng,
            match mode {
                "walking" => "2",
                "transit" => "3",
                "bicycling" => "1",
                _ => "0", // driving
            }
        )
    }

    /// Generate Google Maps venue URL
    pub fn generate_venue_url(&self, coordinate: &Coordinate, place_id: Option<&str>) -> String {
        if let Some(place_id) = place_id {
            format!("https://www.google.com/maps/place/?q=place_id:{}", place_id)
        } else {
            format!(
                "https://www.google.com/maps/search/?api=1&query={},{}",
                coordinate.lat, coordinate.lng
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_directions_url() {
        let service = GeocodingService::new("test_key".to_string());
        let origin = Coordinate {
            lat: 37.7749,
            lng: -122.4194,
        };
        let destination = Coordinate {
            lat: 37.7849,
            lng: -122.4094,
        };

        let url = service.generate_directions_url(&origin, &destination, "drive");
        assert!(url.contains("google.com/maps/dir"));
        assert!(url.contains("37.7749"));
    }

    #[test]
    fn test_generate_venue_url() {
        let service = GeocodingService::new("test_key".to_string());
        let coordinate = Coordinate {
            lat: 37.7749,
            lng: -122.4194,
        };

        let url = service.generate_venue_url(&coordinate, Some("test_place_id"));
        assert!(url.contains("place_id:test_place_id"));

        let url_no_place_id = service.generate_venue_url(&coordinate, None);
        assert!(url_no_place_id.contains("37.7749"));
    }
}
