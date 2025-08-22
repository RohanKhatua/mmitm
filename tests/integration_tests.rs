use mmitm::models::Coordinate;
use mmitm::services::geometry::GeometryService;

#[test]
fn test_two_participants_midpoint() {
    let participants = vec![
        Coordinate { lat: 0.0, lng: 0.0 },
        Coordinate { lat: 2.0, lng: 2.0 },
    ];

    let result = GeometryService::calculate_search_center(&participants).unwrap();

    // Should be midpoint
    assert_eq!(result.coordinate.lat, 1.0);
    assert_eq!(result.coordinate.lng, 1.0);
}

#[test]
fn test_three_participants_centroid() {
    let participants = vec![
        Coordinate { lat: 0.0, lng: 0.0 },
        Coordinate { lat: 3.0, lng: 0.0 },
        Coordinate { lat: 0.0, lng: 3.0 },
    ];

    let result = GeometryService::calculate_search_center(&participants).unwrap();

    // Should be centroid
    assert_eq!(result.coordinate.lat, 1.0);
    assert_eq!(result.coordinate.lng, 1.0);
}

#[test]
fn test_haversine_distance_zero() {
    let coord = Coordinate {
        lat: 12.9716,
        lng: 77.5946,
    };
    let distance = GeometryService::haversine_distance(&coord, &coord);
    assert_eq!(distance, 0.0);
}

#[test]
fn test_invalid_participants_count() {
    let participants = vec![Coordinate { lat: 0.0, lng: 0.0 }];

    let result = GeometryService::calculate_search_center(&participants);
    assert!(result.is_err());
}
