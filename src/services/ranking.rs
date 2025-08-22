use crate::models::VenueWithTravelTimes;

pub struct RankingService;

impl RankingService {
    /// Rank venues based on total travel time and fairness
    /// For two participants, penalize venues with travel time imbalance > 10 minutes
    pub fn rank_venues(
        mut venues: Vec<VenueWithTravelTimes>,
        is_two_participants: bool,
    ) -> Vec<VenueWithTravelTimes> {
        // Apply penalty for two participants with imbalance > 10 minutes
        if is_two_participants {
            for venue in &mut venues {
                let imbalance = venue.max_travel_time - venue.min_travel_time;
                if imbalance > 10 {
                    // Apply penalty by increasing fairness score
                    venue.fairness_score += (imbalance as f64) * 2.0;
                }
            }
        }

        // Sort by composite score: fairness first (lower is better), then total time
        venues.sort_by(|a, b| {
            // Primary sort: fairness score (lower is better)
            let fairness_cmp = a
                .fairness_score
                .partial_cmp(&b.fairness_score)
                .unwrap_or(std::cmp::Ordering::Equal);

            if fairness_cmp != std::cmp::Ordering::Equal {
                return fairness_cmp;
            }

            // Secondary sort: total travel time (lower is better)
            a.total_travel_time.cmp(&b.total_travel_time)
        });

        venues
    }

    /// Calculate a composite score for ranking (lower is better)
    #[allow(dead_code)]
    pub fn calculate_composite_score(venue: &VenueWithTravelTimes, fairness_weight: f64) -> f64 {
        // Normalize travel time (assume max reasonable time is 120 minutes)
        let normalized_time = (venue.total_travel_time as f64) / 120.0;

        // Normalize fairness (assume max reasonable difference is 60 minutes)
        let normalized_fairness = venue.fairness_score / 60.0;

        // Weighted combination
        (normalized_time * (1.0 - fairness_weight)) + (normalized_fairness * fairness_weight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Coordinate, PlaceCandidate};

    fn create_test_venue(name: &str, travel_times: Vec<u32>) -> VenueWithTravelTimes {
        let total_time: u32 = travel_times.iter().sum();
        let max_time = *travel_times.iter().max().unwrap();
        let min_time = *travel_times.iter().min().unwrap();
        let fairness_score = (max_time - min_time) as f64;

        VenueWithTravelTimes {
            place: PlaceCandidate {
                place_id: format!("test_{}", name),
                name: name.to_string(),
                coordinate: Coordinate { lat: 0.0, lng: 0.0 },
                types: vec!["restaurant".to_string()],
                rating: Some(4.0),
                user_ratings_total: Some(100),
                price_level: Some("2".to_string()),
            },
            address: "Test Address".to_string(),
            google_url: "https://maps.google.com/test".to_string(),
            travel_times,
            total_travel_time: total_time,
            max_travel_time: max_time,
            min_travel_time: min_time,
            fairness_score,
        }
    }

    #[test]
    fn test_rank_venues_fairness_priority() {
        let venues = vec![
            create_test_venue("unfair_fast", vec![10, 30]), // Total: 40, Unfair: 20
            create_test_venue("fair_slow", vec![25, 25]),   // Total: 50, Fair: 0
        ];

        let ranked = RankingService::rank_venues(venues, false);

        // Fair venue should rank higher despite being slower
        assert_eq!(ranked[0].place.name, "fair_slow");
        assert_eq!(ranked[1].place.name, "unfair_fast");
    }

    #[test]
    fn test_rank_venues_two_participants_penalty() {
        let venues = vec![
            create_test_venue("big_imbalance", vec![10, 25]), // Imbalance: 15 minutes (> 10)
            create_test_venue("small_imbalance", vec![15, 20]), // Imbalance: 5 minutes (< 10)
        ];

        let ranked = RankingService::rank_venues(venues, true);

        // Small imbalance should rank higher due to penalty on big imbalance
        assert_eq!(ranked[0].place.name, "small_imbalance");
        assert_eq!(ranked[1].place.name, "big_imbalance");
    }
}
