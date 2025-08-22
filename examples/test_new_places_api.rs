use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment
    dotenvy::dotenv().ok();

    let api_key = std::env::var("GOOGLE_MAPS_API_KEY")
        .expect("GOOGLE_MAPS_API_KEY environment variable is required");

    println!("Testing Google Places API (New) with Nearby Search...");

    // Test the new Nearby Search API
    let client = reqwest::Client::new();
    let url = "https://places.googleapis.com/v1/places:searchNearby";

    let request_body = json!({
        "includedTypes": ["restaurant", "cafe"],
        "maxResultCount": 5,
        "locationRestriction": {
            "circle": {
                "center": {
                    "latitude": 37.7749,
                    "longitude": -122.4194
                },
                "radius": 1000.0
            }
        },
        "rankPreference": "POPULARITY"
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Goog-Api-Key", &api_key)
        .header("X-Goog-FieldMask", "places.id,places.displayName,places.location,places.types,places.rating,places.userRatingCount,places.priceLevel")
        .json(&request_body)
        .send()
        .await?;

    println!("Response status: {}", response.status());

    if response.status().is_success() {
        let text = response.text().await?;
        println!("Response body: {}", text);
    } else {
        let error_text = response.text().await?;
        println!("Error response: {}", error_text);
    }

    Ok(())
}
