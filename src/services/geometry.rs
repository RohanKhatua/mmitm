use crate::{
    error::AppError,
    models::{Coordinate, SearchCenter},
};

pub struct GeometryService;

impl GeometryService {
    /// Calculate the search center and radius based on participant coordinates
    /// For 2 participants: midpoint with radius = min((distance / 2 + 1 km), 5 km)
    /// For 3+ participants: centroid with radius = min((max_pairwise_distance / 2 + 2 km), 50 km)
    pub fn calculate_search_center(participants: &[Coordinate]) -> Result<SearchCenter, AppError> {
        if participants.len() < 2 {
            return Err(AppError::BadRequest(
                "At least 2 participants are required".to_string(),
            ));
        }

        if participants.len() == 2 {
            let coord1 = &participants[0];
            let coord2 = &participants[1];

            // Calculate midpoint
            let midpoint = Coordinate {
                lat: (coord1.lat + coord2.lat) / 2.0,
                lng: (coord1.lng + coord2.lng) / 2.0,
            };

            // Calculate distance between the two points
            let distance = Self::haversine_distance(coord1, coord2);

            // Radius = min((distance / 2 + 1 km), 5 km)
            let radius = ((distance / 2.0) + 1000.0).min(5000.0);

            return Ok(SearchCenter {
                coordinate: midpoint,
                radius,
            });
        }

        // For 3+ participants, calculate centroid
        let centroid = Self::calculate_centroid(participants);

        // Find maximum pairwise distance
        let max_distance = Self::find_max_pairwise_distance(participants);

        // Radius = min((max_pairwise_distance / 2 + 2 km), 50 km)
        let radius = ((max_distance / 2.0) + 2000.0).min(50000.0);

        Ok(SearchCenter {
            coordinate: centroid,
            radius,
        })
    }

    /// Calculate centroid of multiple coordinates
    fn calculate_centroid(coordinates: &[Coordinate]) -> Coordinate {
        let total_lat: f64 = coordinates.iter().map(|c| c.lat).sum();
        let total_lng: f64 = coordinates.iter().map(|c| c.lng).sum();
        let count = coordinates.len() as f64;

        Coordinate {
            lat: total_lat / count,
            lng: total_lng / count,
        }
    }

    /// Find maximum pairwise distance among all coordinates
    fn find_max_pairwise_distance(coordinates: &[Coordinate]) -> f64 {
        let mut max_distance: f64 = 0.0;

        for i in 0..coordinates.len() {
            for j in (i + 1)..coordinates.len() {
                let distance = Self::haversine_distance(&coordinates[i], &coordinates[j]);
                max_distance = max_distance.max(distance);
            }
        }

        max_distance
    }

    /// Calculate haversine distance between two coordinates in meters
    pub fn haversine_distance(coord1: &Coordinate, coord2: &Coordinate) -> f64 {
        const EARTH_RADIUS: f64 = 6371000.0; // Earth's radius in meters

        let lat1_rad = coord1.lat.to_radians();
        let lat2_rad = coord2.lat.to_radians();
        let delta_lat = (coord2.lat - coord1.lat).to_radians();
        let delta_lng = (coord2.lng - coord1.lng).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let coord1 = Coordinate {
            lat: 12.9716,
            lng: 77.5946,
        }; // Bangalore
        let coord2 = Coordinate {
            lat: 12.2958,
            lng: 76.6394,
        }; // Mysore

        let distance = GeometryService::haversine_distance(&coord1, &coord2);

        // Distance between Bangalore and Mysore is approximately 128 km
        assert!((distance - 128000.0).abs() < 5000.0);
    }

    #[test]
    fn test_calculate_search_center_two_participants() {
        let participants = vec![
            Coordinate {
                lat: 12.9716,
                lng: 77.5946,
            },
            Coordinate {
                lat: 12.2958,
                lng: 76.6394,
            },
        ];

        let result = GeometryService::calculate_search_center(&participants).unwrap();

        // Should be midpoint
        assert!((result.coordinate.lat - 12.6337).abs() < 0.001);
        assert!((result.coordinate.lng - 77.117).abs() < 0.001);

        // Radius should be capped at 5km for this distance
        assert_eq!(result.radius, 5000.0);
    }

    #[test]
    fn test_calculate_centroid() {
        let coordinates = vec![
            Coordinate { lat: 0.0, lng: 0.0 },
            Coordinate { lat: 2.0, lng: 2.0 },
            Coordinate { lat: 4.0, lng: 4.0 },
        ];

        let centroid = GeometryService::calculate_centroid(&coordinates);

        assert_eq!(centroid.lat, 2.0);
        assert_eq!(centroid.lng, 2.0);
    }
}
