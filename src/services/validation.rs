use std::collections::HashSet;

/// Valid Google Places API (New) place types
/// Reference: https://developers.google.com/maps/documentation/places/web-service/supported_types
pub struct PlaceTypeValidator;

impl PlaceTypeValidator {
    /// Validate that all provided place types are supported by Google Places API (New)
    pub fn validate_place_types(types: &[String]) -> Result<(), String> {
        let valid_types = Self::get_valid_place_types();
        let invalid_types: Vec<&String> = types
            .iter()
            .filter(|t| !valid_types.contains(t.as_str()))
            .collect();

        if !invalid_types.is_empty() {
            return Err(format!(
                "Invalid place types: {}. See Google Places API documentation for valid types.",
                invalid_types.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }

        Ok(())
    }

    /// Get the set of valid place types for Google Places API (New)
    /// This is a subset of the most commonly used Table A types
    fn get_valid_place_types() -> HashSet<&'static str> {
        [
            // Food & Drink
            "restaurant", "cafe", "bar", "bakery", "meal_takeaway", "meal_delivery",
            "food", "pizza_restaurant", "fast_food_restaurant",
            
            // Entertainment
            "movie_theater", "museum", "art_gallery", "night_club", "bowling_alley",
            "casino", "tourist_attraction",
            
            // Recreation
            "park", "zoo", "stadium", "gym", "spa", "aquarium", "amusement_park",
            
            // Shopping
            "shopping_mall", "store", "book_store", "clothing_store", "department_store",
            "electronics_store", "furniture_store", "grocery_store", "hardware_store",
            "jewelry_store", "liquor_store", "pet_store", "pharmacy", "shoe_store",
            "convenience_store",
            
            // Services
            "bank", "atm", "hospital", "dentist", "doctor", "veterinary_care",
            "hair_care", "beauty_salon", "laundry", "car_wash", "gas_station",
            
            // Transportation
            "bus_station", "subway_station", "train_station", "airport", "taxi_stand",
            "parking",
            
            // Accommodation
            "lodging", "hotel", "rv_park", "campground",
            
            // Public Services
            "library", "post_office", "police", "fire_station", "city_hall", "courthouse",
            "embassy", "local_government_office",
            
            // Education
            "school", "university", "primary_school", "secondary_school",
            
            // Places of Worship
            "place_of_worship", "church", "mosque", "synagogue", "temple",
            
            // Others
            "real_estate_agency", "travel_agency", "insurance_agency", "lawyer",
            "accounting", "dentist", "physiotherapist", "roofing_contractor",
            "electrician", "plumber", "locksmith", "moving_company"
        ].into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_place_types() {
        let types = vec!["restaurant".to_string(), "cafe".to_string()];
        assert!(PlaceTypeValidator::validate_place_types(&types).is_ok());
    }

    #[test]
    fn test_invalid_place_types() {
        let types = vec!["invalid_type".to_string(), "another_invalid".to_string()];
        assert!(PlaceTypeValidator::validate_place_types(&types).is_err());
    }

    #[test]
    fn test_mixed_valid_invalid_types() {
        let types = vec!["restaurant".to_string(), "invalid_type".to_string()];
        let result = PlaceTypeValidator::validate_place_types(&types);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid_type"));
    }
}
