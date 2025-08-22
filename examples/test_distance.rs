use mmitm::models::Coordinate;
use mmitm::services::geometry::GeometryService;

fn main() {
    let coord1 = Coordinate {
        lat: 12.9716,
        lng: 77.5946,
    }; // Bangalore
    let coord2 = Coordinate {
        lat: 12.2958,
        lng: 76.6394,
    }; // Mysore

    let distance = GeometryService::haversine_distance(&coord1, &coord2);
    println!("Distance between Bangalore and Mysore: {} meters", distance);
    println!("Distance in kilometers: {} km", distance / 1000.0);
}
